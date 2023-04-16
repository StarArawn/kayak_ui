use std::sync::{Arc, RwLock};

use bevy::{
    ecs::{event::ManualEventReader, system::CommandQueue},
    prelude::*,
    utils::{HashMap, HashSet},
    window::PrimaryWindow,
};
use morphorm::Hierarchy;

use crate::{
    calculate_nodes::{calculate_layout, calculate_nodes},
    children::KChildren,
    clone_component::{clone_state, clone_system, EntityCloneSystems, PreviousWidget},
    context_entities::ContextEntities,
    cursor::PointerEvents,
    event_dispatcher::EventDispatcher,
    focus_tree::FocusTree,
    input::query_world,
    layout::{LayoutCache, Rect},
    layout_dispatcher::LayoutEventDispatcher,
    node::{DirtyNode, Node, WrappedIndex},
    prelude::KayakWidgetContext,
    render_primitive::RenderPrimitive,
    styles::{
        ComputedStyles, Corner, Edge, KCursorIcon, KPositionType, KStyle, LayoutType,
        RenderCommand, StyleProp, Units,
    },
    tree::{Change, Tree},
    widget_state::WidgetState,
    Focusable, KayakUIPlugin, WindowSize,
};

/// A tag component representing when a widget has been mounted(added to the tree).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Mounted;

const UPDATE_DEPTH: u32 = 0;

type WidgetSystems = HashMap<
    String,
    (
        Box<dyn System<In = (KayakWidgetContext, Entity, Entity), Out = bool>>,
        Box<dyn System<In = (KayakWidgetContext, Entity), Out = bool>>,
    ),
>;

///
/// Kayak Context
///
/// This bevy resource keeps track of all of the necessary UI state. This includes the widgets, tree, input, layout, and other important data.
/// The Context provides some connivent helper functions for creating and using widgets, state, and context.
///
/// Usage:
/// ```rust
/// use bevy::prelude::*;
/// use kayak_ui::prelude::{widgets::*, *};
///
/// // Bevy setup function
/// fn setup(mut commands: Commands) {
///     let mut widget_context = Context::new();
///     let app_entity = commands.spawn(KayakAppBundle {
///         ..Default::default()
///     }).id();
///     // Stores the kayak app widget in the widget context's tree.
///     widget_context.add_widget(None, app_entity);
///     commands.spawn((widget_context, EventDispatcher::default()));
/// }
///
/// fn main() {
///     App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugin(ContextPlugin)
///     .add_plugin(KayakWidgets)
///     .add_startup_system(setup);
/// }
/// ```
#[derive(Component)]
pub struct KayakRootContext {
    pub tree: Arc<RwLock<Tree>>,
    pub(crate) layout_cache: Arc<RwLock<LayoutCache>>,
    pub(crate) focus_tree: Arc<RwLock<FocusTree>>,
    systems: WidgetSystems,
    pub(crate) current_z: f32,
    pub(crate) context_entities: ContextEntities,
    pub(crate) current_cursor: CursorIcon,
    pub(crate) clone_systems: Arc<RwLock<EntityCloneSystems>>,
    pub(crate) cloned_widget_entities: Arc<RwLock<HashMap<Entity, Entity>>>,
    pub(crate) widget_state: WidgetState,
    pub(crate) order_tree: Arc<RwLock<Tree>>,
    pub(crate) index: Arc<RwLock<HashMap<Entity, usize>>>,
    /// Unique id's store entity id's related to a key rather than the child tree.
    /// This lets users get a unique entity. The first Entity is the parent widget.
    /// The 2nd hashmap is a list of keys and their entities.
    pub(crate) unique_ids: Arc<RwLock<HashMap<Entity, HashMap<String, Entity>>>>,
    /// Maps keyed entities to spawn parents. We can't use the tree in this case.
    pub(crate) unique_ids_parents: Arc<RwLock<HashMap<Entity, Entity>>>,
    pub(crate) uninitilized_systems: HashSet<String>,
    pub camera_entity: Entity,
}

impl Default for KayakRootContext {
    fn default() -> Self {
        Self::new(Entity::from_raw(0))
    }
}

impl KayakRootContext {
    /// Creates a new widget context.
    pub fn new(camera_entity: Entity) -> Self {
        Self {
            tree: Arc::new(RwLock::new(Tree::default())),
            layout_cache: Arc::new(RwLock::new(LayoutCache::default())),
            focus_tree: Default::default(),
            systems: HashMap::default(),
            current_z: 0.0,
            context_entities: ContextEntities::new(),
            current_cursor: CursorIcon::Default,
            clone_systems: Default::default(),
            cloned_widget_entities: Default::default(),
            widget_state: Default::default(),
            index: Default::default(),
            order_tree: Default::default(),
            unique_ids: Default::default(),
            unique_ids_parents: Default::default(),
            uninitilized_systems: Default::default(),
            camera_entity,
        }
    }

    /// Adds a kayak plugin and runs the build function on the context.
    pub fn add_plugin(&mut self, plugin: impl KayakUIPlugin) {
        plugin.build(self)
    }

    /// Retreives the current entity that has focus or None if nothing is focused.
    pub fn get_current_focus(&self) -> Option<Entity> {
        if let Ok(tree) = self.focus_tree.try_read() {
            return tree.current().and_then(|a| Some(a.0));
        }
        None
    }

    /// Get's the layout for th given widget index.
    pub(crate) fn get_layout(&self, id: &WrappedIndex) -> Option<Rect> {
        if let Ok(cache) = self.layout_cache.try_read() {
            cache.rect.get(id).cloned()
        } else {
            None
        }
    }

    pub(crate) fn get_geometry_changed(&self, id: &WrappedIndex) -> bool {
        if let Ok(cache) = self.layout_cache.try_read() {
            if let Some(geometry_changed) = cache.geometry_changed.get(id) {
                !geometry_changed.is_empty()
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Adds a new set of systems for a widget type.
    /// Update systems are ran every frame and return true or false depending on if the widget has "changed".
    /// Render systems are ran only if the widget has changed and are meant to re-render children and handle
    /// tree changes.
    pub fn add_widget_system<Params, Params2>(
        &mut self,
        type_name: impl Into<String>,
        update: impl IntoSystem<(KayakWidgetContext, Entity, Entity), bool, Params>,
        render: impl IntoSystem<(KayakWidgetContext, Entity), bool, Params2>,
    ) {
        let type_name = type_name.into();
        let update_system = Box::new(IntoSystem::into_system(update));
        let render_system = Box::new(IntoSystem::into_system(render));
        self.systems
            .insert(type_name.clone(), (update_system, render_system));
        self.uninitilized_systems.insert(type_name);
    }

    /// Let's the widget context know what data types are used for a given widget.
    /// This is useful as it allows Kayak to keep track of previous values for diffing.
    /// When the default update widget system is called it checks the props and state of
    /// the current widget with it's values from the previous frame.
    /// This allows Kayak to diff data. Alternatively a custom widget update system can
    /// be used and listen for events, resources, or any other bevy ECS data.
    pub fn add_widget_data<
        Props: Component + Clone + PartialEq,
        State: Component + Clone + PartialEq,
    >(
        &mut self,
    ) {
        if let Ok(mut clone_systems) = self.clone_systems.try_write() {
            clone_systems
                .0
                .push((clone_system::<Props>, clone_state::<State>));
        }
    }

    /// Adds a widget to the tree.
    /// Widgets are created using entities and components.
    /// Once created their id's need to be added to the widget tree
    /// so that the correct ordering is preserved for updates and rendering.
    pub fn add_widget(&mut self, parent: Option<Entity>, entity: Entity) {
        if let Ok(mut tree) = self.tree.write() {
            tree.add(WrappedIndex(entity), parent.map(WrappedIndex));
            if let Ok(mut cache) = self.layout_cache.try_write() {
                cache.add(WrappedIndex(entity));
            }
        }
    }

    /// Creates a new context using the context entity for the given type_id + parent id.
    /// Context can be considered state that changes across multiple components.
    /// Alternatively you can use bevy's resources.
    pub fn set_context_entity<T: Default + 'static>(
        &self,
        parent_id: Option<Entity>,
        context_entity: Entity,
    ) {
        if let Some(parent_id) = parent_id {
            self.context_entities
                .add_context_entity::<T>(parent_id, context_entity);
        }
    }

    /// Returns a new/existing widget entity.
    /// Because a re-render can potentially spawn new entities it's advised to use this
    /// to avoid creating a new entity.
    ///
    /// Usage:
    /// ```rust
    /// fn setup() {
    ///     let mut widget_context = WidgetContext::new();
    ///     // Root tree node, no parent node.
    ///     let root_entity =  widget_context.spawn_widget(&mut commands, None);
    ///     commands.entity(root_entity).insert(KayakAppBundle::default());
    ///     widget_context.add_widget(None, root_entity);
    /// }
    ///```
    pub fn spawn_widget(
        &self,
        commands: &mut Commands,
        key: Option<&'static str>,
        parent_id: Option<Entity>,
    ) -> Entity {
        let mut entity = None;
        if let Some(parent_entity) = parent_id {
            if let Some(key) = key.map(|key| key.to_string()) {
                if let Ok(unique_ids) = self.unique_ids.try_read() {
                    if let Some(key_hashmap) = unique_ids.get(&parent_entity) {
                        entity = key_hashmap.get(&key).cloned();

                        if let Some(child) = entity {
                            if let Some(mut entity_commands) = commands.get_entity(child) {
                                entity_commands.despawn();
                            }
                            entity =
                                Some(commands.get_or_spawn(child).set_parent(parent_entity).id());
                            log::trace!(
                                "Reusing keyed widget entity {:?} with parent: {:?}!",
                                child.index(),
                                parent_id.unwrap().index()
                            );
                        }
                    } else {
                        log::trace!("couldn't find key entity on parent!");
                    }
                } else {
                    panic!("Couldn't get unique id lock!");
                }
            } else {
                let children = self.get_children_ordered(parent_entity);
                // We need to increment the index count even if we are using the unique id key.
                let index = self.get_and_add_index(parent_entity);
                let child = children.get(index).cloned();

                if let Some(child) = child {
                    log::trace!(
                        "Reusing widget entity {:?} with parent: {:?}!",
                        child.index(),
                        parent_id.unwrap().index()
                    );
                    if let Some(mut entity_commands) = commands.get_entity(child) {
                        entity_commands.despawn();
                    }
                    entity = Some(commands.get_or_spawn(child).id());
                }
            }
        }

        // If we have no entity spawn it!
        if entity.is_none() {
            entity = Some(commands.spawn_empty().id());
            log::trace!(
                "Spawning new widget with entity {:?}!",
                entity.unwrap().index()
            );

            // Note: The root widget cannot have a key for now..
            if let Some(parent_entity) = parent_id {
                commands.entity(entity.unwrap()).set_parent(parent_entity);

                if let Some(key) = key.map(|key| key.to_string()) {
                    if let Ok(mut unique_ids) = self.unique_ids.try_write() {
                        if let Some(key_hashmap) = unique_ids.get_mut(&parent_entity) {
                            key_hashmap.insert(key, entity.unwrap());
                            if let Ok(mut unique_ids_parents) = self.unique_ids_parents.try_write()
                            {
                                unique_ids_parents.insert(entity.unwrap(), parent_entity);
                            }
                        } else {
                            let mut key_hashmap = HashMap::new();
                            key_hashmap.insert(key, entity.unwrap());
                            unique_ids.insert(parent_entity, key_hashmap);
                            if let Ok(mut unique_ids_parents) = self.unique_ids_parents.try_write()
                            {
                                unique_ids_parents.insert(entity.unwrap(), parent_entity);
                            }
                        }
                    }
                } else {
                    // We need to add it to the ordered tree
                    if let Ok(mut tree) = self.order_tree.try_write() {
                        tree.add(WrappedIndex(entity.unwrap()), parent_id.map(WrappedIndex))
                    }
                }
            }
        }
        entity.unwrap()
    }

    fn get_children_ordered(&self, entity: Entity) -> Vec<Entity> {
        let mut children = vec![];
        if let Ok(tree) = self.order_tree.read() {
            let iterator = tree.child_iter(WrappedIndex(entity));

            children = iterator.map(|index| index.0).collect::<Vec<_>>();
        }

        children
    }

    fn get_and_add_index(&self, parent: Entity) -> usize {
        if let Ok(mut hash_map) = self.index.try_write() {
            if hash_map.contains_key(&parent) {
                let index = hash_map.get_mut(&parent).unwrap();
                let current_index = *index;
                *index += 1;
                return current_index;
            } else {
                hash_map.insert(parent, 1);
                return 0;
            }
        }

        0
    }

    /// Generates a flat list of widget render commands sorted by tree order.
    /// There is no need to call this unless you are implementing your own custom renderer.
    pub fn build_render_primitives(
        &self,
        nodes: &Query<&crate::node::Node>,
        widget_names: &Query<&WidgetName>,
    ) -> Vec<RenderPrimitive> {
        let node_tree = self.tree.try_read();
        if node_tree.is_err() {
            return vec![];
        }

        let node_tree = node_tree.unwrap();

        if node_tree.root_node.is_none() {
            return vec![];
        }

        let render_primitives = if let Ok(mut layout_cache) = self.layout_cache.try_write() {
            recurse_node_tree_to_build_primitives(
                &node_tree,
                &mut layout_cache,
                nodes,
                widget_names,
                node_tree.root_node.unwrap(),
                0.0,
                RenderPrimitive::Empty,
            )
        } else {
            vec![]
        };
        // render_primitives.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // render_primitives.iter().enumerate().for_each(|(index, p)| {
        //     log::info!("Name: {:?}, Z: {:?}", p.to_string(), index);
        // });

        // dbg!(&render_primitives
        //     .iter()
        //     .map(|a| (a.1.to_string(), a.0))
        //     .collect::<Vec<_>>());

        render_primitives.into_iter().collect()
    }
}

fn recurse_node_tree_to_build_primitives(
    node_tree: &Tree,
    layout_cache: &mut LayoutCache,
    nodes: &Query<&crate::node::Node>,
    widget_names: &Query<&WidgetName>,
    current_node: WrappedIndex,
    main_z_index: f32,
    mut prev_clip: RenderPrimitive,
) -> Vec<RenderPrimitive> {
    let mut render_primitives = Vec::new();
    if let Ok(node) = nodes.get(current_node.0) {
        let mut render_primitive = node.primitive.clone();
        let mut new_z_index = main_z_index;

        let layout = if let Some(layout) = layout_cache.rect.get_mut(&current_node) {
            log::trace!(
                "z_index is {} and node.z is {} for: {}-{}",
                new_z_index,
                node.z,
                widget_names.get(current_node.0).unwrap().0,
                current_node.0.index(),
            );

            new_z_index += if node.z <= 0.0 { 0.0 } else { node.z };

            layout.z_index = new_z_index;
            render_primitive.set_layout(*layout);
            *layout
        } else {
            log::warn!(
                "No layout for node: {}-{}",
                widget_names.get(current_node.0).unwrap().0,
                current_node.0.index()
            );
            Rect::default()
        };

        match &render_primitive {
            RenderPrimitive::Text {
                content, layout, ..
            } => {
                log::trace!(
                    "Text node: {}-{} is equal to: {}, {:?}",
                    widget_names.get(current_node.0).unwrap().0,
                    current_node.0.index(),
                    content,
                    layout,
                );
            }
            RenderPrimitive::Clip { layout } => {
                log::trace!(
                    "Clip node: {}-{} is equal to: {:?}",
                    widget_names.get(current_node.0).unwrap().0,
                    current_node.0.index(),
                    layout,
                );
            }
            RenderPrimitive::Empty => {
                log::trace!(
                    "Empty node: {}-{} is equal to: {:?}",
                    widget_names.get(current_node.0).unwrap().0,
                    current_node.0.index(),
                    layout
                );
            }
            _ => {}
        }

        render_primitives.push(render_primitive.clone());

        let new_prev_clip = if matches!(render_primitive, RenderPrimitive::Clip { .. }) {
            render_primitive.clone()
        } else {
            prev_clip
        };

        prev_clip = new_prev_clip.clone();
        if node_tree.children.contains_key(&current_node) {
            let z = 1.0f32;
            let mut children_primitives = Vec::new();
            for child in node_tree.children.get(&current_node).unwrap() {
                // main_z_index += 1.0;
                let mut children_p = recurse_node_tree_to_build_primitives(
                    node_tree,
                    layout_cache,
                    nodes,
                    widget_names,
                    *child,
                    main_z_index + if node.z < 0.0 { 0.0 } else { node.z } + z,
                    new_prev_clip.clone(),
                );

                // Between each child node we need to reset the clip.
                if matches!(prev_clip, RenderPrimitive::Clip { .. }) {
                    children_p.push(prev_clip.clone());
                }

                if let Ok(node) = nodes.get(child.0) {
                    let zz = if node.z < 0.0 { z } else { z + node.z };
                    children_primitives.push((zz, children_p));
                }
            }

            // Sort and add
            children_primitives.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for cp in children_primitives.drain(..) {
                render_primitives.extend(cp.1);
            }
        } else {
            log::trace!(
                "No children for node: {}-{}",
                widget_names.get(current_node.0).unwrap().0,
                current_node.0.index()
            );
        }
    } else {
        log::error!(
            "No render node: {}-{} > {}-{}",
            node_tree
                .get_parent(current_node)
                .map(|v| v.0.index() as i32)
                .unwrap_or(-1),
            widget_names
                .get(
                    node_tree
                        .get_parent(current_node)
                        .map(|v| v.0)
                        .unwrap_or(Entity::from_raw(0))
                )
                .map(|v| v.0.clone())
                .unwrap_or_else(|_| "None".into()),
            widget_names
                .get(current_node.0)
                .map(|v| v.0.clone())
                .unwrap_or_else(|_| "None".into()),
            current_node.0.index()
        );
    }

    render_primitives
}

fn update_widgets_sys(world: &mut World) {
    let mut context_data = Vec::new();

    query_world::<Query<(Entity, &mut KayakRootContext)>, _, _>(
        |mut query| {
            for (entity, mut kayak_root_context) in query.iter_mut() {
                context_data.push((entity, std::mem::take(&mut *kayak_root_context)));
            }
        },
        world,
    );

    for (entity, mut context) in context_data.drain(..) {
        for system_id in context.uninitilized_systems.drain() {
            if let Some(system) = context.systems.get_mut(&system_id) {
                system.0.initialize(world);
                system.1.initialize(world);
            }
        }

        let tree_iterator = if let Ok(tree) = context.tree.read() {
            tree.down_iter().collect::<Vec<_>>()
        } else {
            panic!("Failed to acquire read lock.");
        };

        // let change_tick = world.increment_change_tick();

        let old_focus = if let Ok(mut focus_tree) = context.focus_tree.try_write() {
            let current = focus_tree.current();
            focus_tree.clear();
            if let Ok(tree) = context.tree.read() {
                if let Some(root_node) = tree.root_node {
                    focus_tree.add(root_node, &tree);
                }
            }
            current
        } else {
            None
        };

        let mut new_ticks = HashMap::new();

        // dbg!("Updating widgets!");
        update_widgets(
            context.camera_entity,
            world,
            &context.tree,
            &context.layout_cache,
            &mut context.systems,
            tree_iterator,
            &context.context_entities,
            &context.focus_tree,
            &context.clone_systems,
            &context.cloned_widget_entities,
            &context.widget_state,
            &mut new_ticks,
            &context.order_tree,
            &context.index,
            &context.unique_ids,
            &context.unique_ids_parents,
        );

        if let Some(old_focus) = old_focus {
            if let Ok(mut focus_tree) = context.focus_tree.try_write() {
                if focus_tree.contains(old_focus) {
                    focus_tree.focus(old_focus);
                }
            }
        }

        // dbg!("Finished updating widgets!");
        let tick = world.read_change_tick();

        for (key, system) in context.systems.iter_mut() {
            if let Some(new_tick) = new_ticks.get(key) {
                system.0.set_last_change_tick(*new_tick);
                system.1.set_last_change_tick(*new_tick);
            } else {
                system.0.set_last_change_tick(tick);
                system.1.set_last_change_tick(tick);
            }
            // system.apply_buffers(world);
        }

        // Clear out indices
        if let Ok(mut indices) = context.index.try_write() {
            // for (entity, value) in indices.iter_mut() {
            //     if tree.root_node.unwrap().0.id() != entity.id() {
            //         *value = 0;
            //     }
            // }
            indices.clear();
        }

        world.entity_mut(entity).insert(context);
    }
}

fn update_widgets(
    camera_entity: Entity,
    world: &mut World,
    tree: &Arc<RwLock<Tree>>,
    layout_cache: &Arc<RwLock<LayoutCache>>,
    systems: &mut WidgetSystems,
    widgets: Vec<WrappedIndex>,
    context_entities: &ContextEntities,
    focus_tree: &Arc<RwLock<FocusTree>>,
    clone_systems: &Arc<RwLock<EntityCloneSystems>>,
    cloned_widget_entities: &Arc<RwLock<HashMap<Entity, Entity>>>,
    widget_state: &WidgetState,
    new_ticks: &mut HashMap<String, u32>,
    order_tree: &Arc<RwLock<Tree>>,
    index: &Arc<RwLock<HashMap<Entity, usize>>>,
    unique_ids: &Arc<RwLock<HashMap<Entity, HashMap<String, Entity>>>>,
    unique_ids_parents: &Arc<RwLock<HashMap<Entity, Entity>>>,
) {
    for entity in widgets.iter() {
        // A small hack to add parents to widgets
        // let mut command_queue = CommandQueue::default();
        // {
        //     let mut commands = Commands::new(&mut command_queue, &world);
        //     if let Some(mut entity_commands) = commands.get_entity(entity.0) {
        //         entity_commands.set_parent(camera_entity);
        //     }
        // }
        // command_queue.apply(world);

        if let Some(entity_ref) = world.get_entity(entity.0) {
            if let Some(widget_type) = entity_ref.get::<WidgetName>() {
                let widget_context = KayakWidgetContext::new(
                    tree.clone(),
                    context_entities.clone(),
                    layout_cache.clone(),
                    widget_state.clone(),
                    order_tree.clone(),
                    index.clone(),
                    Some(camera_entity),
                    unique_ids.clone(),
                    unique_ids_parents.clone(),
                );
                widget_context.copy_from_point(tree, *entity);
                let children_before = widget_context.get_children(entity.0);
                // let widget_name = widget_type.0.clone();
                let (widget_context, should_update_children) = update_widget(
                    systems,
                    tree,
                    world,
                    *entity,
                    widget_type.0.clone(),
                    widget_context,
                    children_before,
                    clone_systems,
                    cloned_widget_entities,
                    widget_state,
                    new_ticks,
                    order_tree,
                    unique_ids,
                    unique_ids_parents,
                );

                if should_update_children {
                    if let Ok(mut tree) = tree.write() {
                        let mut _had_removal = false;
                        let diff = tree.diff_children(&widget_context, *entity, UPDATE_DEPTH);
                        for (_index, child, _parent, changes) in diff.changes.iter() {
                            if changes
                                .iter()
                                .any(|change| matches!(change, Change::Inserted))
                            {
                                if let Ok(mut cache) = layout_cache.try_write() {
                                    cache.add(*child);
                                }
                            }

                            if changes
                                .iter()
                                .any(|change| matches!(change, Change::Deleted))
                            {
                                _had_removal = true;
                            }
                        }

                        // if _had_removal {
                        //     tree.dump();
                        //     dbg!(&diff);
                        // }

                        tree.merge(&widget_context, *entity, diff, UPDATE_DEPTH);

                        // if _had_removal {
                        //     tree.dump();
                        // }

                        for child in widget_context.child_iter(*entity) {
                            if let Some(mut entity_commands) = world.get_entity_mut(child.0) {
                                entity_commands.insert(DirtyNode);
                            }
                        }
                    }
                }

                // if should_update_children {
                let children = if let Ok(tree) = tree.read() {
                    tree.child_iter(*entity).collect::<Vec<_>>()
                } else {
                    vec![]
                };

                // dbg!((entity, &children));
                update_widgets(
                    camera_entity,
                    world,
                    tree,
                    layout_cache,
                    systems,
                    children,
                    context_entities,
                    focus_tree,
                    clone_systems,
                    cloned_widget_entities,
                    widget_state,
                    new_ticks,
                    order_tree,
                    index,
                    unique_ids,
                    unique_ids_parents,
                );
                // }
            }
        } else {
            // In this case the entity we are trying to process no longer exists.
            // The approach taken here removes said entities from the tree.
            let mut despawn_list = Vec::default();
            if let Ok(mut tree) = tree.write() {
                for child in tree.down_iter_at(*entity, true) {
                    despawn_list.push(child.0);
                    if let Ok(mut order_tree) = order_tree.try_write() {
                        // had_removal = true;
                        log::trace!(
                            "Removing entity! {:?} inside of: {:?}",
                            child.0.index(),
                            entity.0.index()
                        );
                        order_tree.remove(child);
                    }
                }

                for entity in despawn_list.drain(..) {
                    tree.remove(WrappedIndex(entity));
                    if let Some(entity_mut) = world.get_entity_mut(entity) {
                        entity_mut.despawn();
                    }
                }
            }
        }

        if let Some(entity_ref) = world.get_entity(entity.0) {
            if entity_ref.contains::<Focusable>() {
                if let Ok(tree) = tree.try_read() {
                    if let Ok(mut focus_tree) = focus_tree.try_write() {
                        focus_tree.add(*entity, &tree);
                    }
                }
            }
        }
    }
}

fn update_widget(
    systems: &mut WidgetSystems,
    tree: &Arc<RwLock<Tree>>,
    world: &mut World,
    entity: WrappedIndex,
    widget_type: String,
    widget_context: KayakWidgetContext,
    previous_children: Vec<Entity>,
    clone_systems: &Arc<RwLock<EntityCloneSystems>>,
    cloned_widget_entities: &Arc<RwLock<HashMap<Entity, Entity>>>,
    widget_state: &WidgetState,
    new_ticks: &mut HashMap<String, u32>,
    order_tree: &Arc<RwLock<Tree>>,
    unique_ids: &Arc<RwLock<HashMap<Entity, HashMap<String, Entity>>>>,
    unique_ids_parents: &Arc<RwLock<HashMap<Entity, Entity>>>,
) -> (Tree, bool) {
    // Check if we should update this widget
    let should_rerender = {
        let old_props_entity =
            if let Ok(mut cloned_widget_entities) = cloned_widget_entities.try_write() {
                if let Some(entity) = cloned_widget_entities.get(&entity.0).cloned() {
                    if let Some(possible_entity) = world.get_entity(entity) {
                        let target = possible_entity.id();
                        cloned_widget_entities.insert(entity, target);
                        target
                    } else {
                        let target = world.spawn_empty().insert(PreviousWidget).id();
                        cloned_widget_entities.insert(entity, target);
                        target
                    }
                } else {
                    let target = world.spawn_empty().insert(PreviousWidget).id();
                    cloned_widget_entities.insert(entity.0, target);
                    target
                }
            } else {
                panic!("Couldn't get write lock!")
            };

        let widget_update_system = &mut systems
            .get_mut(&widget_type)
            .expect(&format!(
                "Wasn't able to find render/update systems for widget: {}!",
                widget_type
            ))
            .0;
        let old_tick = widget_update_system.get_last_change_tick();
        let should_rerender =
            widget_update_system.run((widget_context.clone(), entity.0, old_props_entity), world);
        let new_tick = widget_update_system.get_last_change_tick();
        new_ticks.insert(widget_type.clone(), new_tick);
        widget_update_system.set_last_change_tick(old_tick);
        widget_update_system.apply_buffers(world);

        if should_rerender {
            if let Ok(cloned_widget_entities) = cloned_widget_entities.try_read() {
                if let Some(target_entity) = cloned_widget_entities.get(&entity.0) {
                    if let Ok(clone_systems) = clone_systems.try_read() {
                        for s in clone_systems.0.iter() {
                            s.0(world, *target_entity, entity.0);
                            s.1(world, *target_entity, entity.0, widget_state);
                            if let Some(styles) = world.entity(entity.0).get::<KStyle>().cloned() {
                                if let Some(mut entity) = world.get_entity_mut(*target_entity) {
                                    entity.insert(styles);
                                }
                            }
                            if let Some(styles) =
                                world.entity(entity.0).get::<ComputedStyles>().cloned()
                            {
                                if let Some(mut entity) = world.get_entity_mut(*target_entity) {
                                    entity.insert(styles);
                                }
                            }
                            if let Some(children) =
                                world.entity(entity.0).get::<KChildren>().cloned()
                            {
                                if let Some(mut entity) = world.get_entity_mut(*target_entity) {
                                    entity.insert(children);
                                }
                            }

                            if let Some(widget_name) =
                                world.entity(entity.0).get::<WidgetName>().cloned()
                            {
                                if let Some(mut entity) = world.get_entity_mut(*target_entity) {
                                    entity.insert(widget_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        should_rerender
    };

    if !should_rerender {
        return (widget_context.take(), false);
    }

    let should_update_children;
    if let Ok(tree) = tree.try_read() {
        log::trace!(
            "Re-rendering: {:?} {:?}, parent: {:?}",
            &widget_type,
            entity.0.index(),
            tree.parent(entity)
                .unwrap_or(WrappedIndex(Entity::from_raw(99999)))
                .0
                .index()
        );
    }
    {
        // Before rendering widget we need to advance the indices correctly..
        if let Some(children) = world.get::<KChildren>(entity.0) {
            let child_count = children.len();
            if let Ok(mut indices) = widget_context.index.try_write() {
                indices.insert(entity.0, 0);
                log::trace!(
                    "Advancing children for: {:?} by: {:?}",
                    entity.0.index(),
                    child_count
                );
            }
        }

        // Remove children from previous render.
        widget_context.remove_children(previous_children);
        let widget_render_system = &mut systems.get_mut(&widget_type).unwrap().1;
        let old_tick = widget_render_system.get_last_change_tick();
        should_update_children =
            widget_render_system.run((widget_context.clone(), entity.0), world);
        let new_tick = widget_render_system.get_last_change_tick();
        new_ticks.insert(widget_type.clone(), new_tick);
        widget_render_system.set_last_change_tick(old_tick);
        widget_render_system.apply_buffers(world);

        if let Ok(mut indices) = widget_context.index.try_write() {
            indices.insert(entity.0, 0);
        }
    }
    let widget_context = widget_context.take();
    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, world);

    commands.entity(entity.0).remove::<Mounted>();

    let diff = if let Ok(tree) = tree.read() {
        tree.diff_children(&widget_context, entity, UPDATE_DEPTH)
    } else {
        panic!("Failed to acquire read lock.");
    };

    // log::info!("Entity: {:?}, Diff: {:?}", entity.0, &diff);

    // Always mark widget dirty if it's re-rendered.
    // Mark node as needing a recalculation of rendering/layout.
    commands.entity(entity.0).insert(DirtyNode);

    command_queue.apply(world);

    // Children of this node need to be despawned.
    let mut despawn_list = Vec::default();

    for (_index, changed_entity, parent, changes) in diff.changes.iter() {
        if changes.iter().any(|change| *change == Change::Inserted) {
            if let Some(mut entity_commands) = world.get_entity_mut(changed_entity.0) {
                entity_commands.insert(Mounted);
                entity_commands.set_parent(parent.0);
            }
            world.entity_mut(parent.0).add_child(changed_entity.0);
        } else if changes
            .iter()
            .any(|change| matches!(change, Change::Deleted))
        {
            if let Ok(tree) = tree.try_read() {
                for child in tree.down_iter_at(*changed_entity, true) {
                    // Due to a bug in bevy we need to remove the parent manually otherwise we'll panic later.
                    world.entity_mut(child.0).remove_parent();

                    // let children = if let Some(parent) = tree.get_parent(child) {
                    //     world.entity(parent.0).get::<KChildren>()
                    // } else {
                    //     None
                    // };

                    let parent = tree.parent(child).unwrap();

                    // if let Some(children) = children {
                    // for child in children.iter() {
                    //     despawn_list.push((parent.0, *child));
                    // }
                    // } //else {

                    despawn_list.push((parent.0, child.0));

                    // }
                    if let Ok(mut order_tree) = order_tree.try_write() {
                        order_tree.remove(child);
                    }
                }
            }
        }
    }

    for (parent, entity) in despawn_list.drain(..) {
        // Clear out keyed entity.
        if let (Ok(mut unique_ids), Ok(mut unique_ids_parents)) =
            (unique_ids.try_write(), unique_ids_parents.try_write())
        {
            if let Some(parent) = unique_ids_parents.get(&entity) {
                if let Some(keyed_hashmap) = unique_ids.get_mut(&parent) {
                    let possible_key = keyed_hashmap
                        .iter()
                        .find(|(_, keyed_entity)| **keyed_entity == entity)
                        .map(|(key, _)| key.clone());
                    if let Some(key) = possible_key {
                        keyed_hashmap.remove(&key);
                        unique_ids_parents.remove(&entity);
                        log::trace!("Removing key {key}, for entity: {:?}", entity);
                    }
                }
            }
        }

        // Remove state entity
        if let Some(state_entity) = widget_state.remove(entity) {
            if let Some(entity_mut) = world.get_entity_mut(state_entity) {
                entity_mut.despawn_recursive();
            }
        }

        // Remove widget entity
        if let Some(entity_mut) = world.get_entity_mut(entity) {
            log::trace!(
                "Removing entity! {:?} - {:?} with parent {:?}",
                entity.index(),
                entity_mut.get::<WidgetName>(),
                parent.index(),
                // entity.index()
            );
            entity_mut.despawn();

            // Also remove all cloned widget entities
            if let Ok(cloned_widget_entities) = cloned_widget_entities.try_read() {
                if let Some(entity) = cloned_widget_entities.get(&entity) {
                    world.despawn(*entity);
                }
            }
        }
    }

    // if should_update_children {
    if let Ok(cloned_widget_entities) = cloned_widget_entities.try_read() {
        if let Some(target_entity) = cloned_widget_entities.get(&entity.0) {
            if let Some(styles) = world.entity(entity.0).get::<KStyle>().cloned() {
                if let Some(mut entity) = world.get_entity_mut(*target_entity) {
                    entity.insert(styles);
                }
            }
            if let Some(styles) = world.entity(entity.0).get::<ComputedStyles>().cloned() {
                if let Some(mut entity) = world.get_entity_mut(*target_entity) {
                    entity.insert(styles);
                }
            }
            if let Some(children) = world.entity(entity.0).get::<KChildren>().cloned() {
                if let Some(mut entity) = world.get_entity_mut(*target_entity) {
                    entity.insert(children);
                }
            }
        }
    }
    // for (_, child_entity, _, changes) in diff.changes.iter() {
    //     // Clone to entity.
    //     if changes.iter().any(|change| *change == Change::Deleted) {

    //     }
    // }
    // }

    (widget_context, should_update_children)
}

/// The default Kayak Context plugin
/// Creates systems and resources for kayak.
pub struct KayakContextPlugin;

#[derive(Resource)]
pub struct CustomEventReader<T: bevy::ecs::event::Event>(pub ManualEventReader<T>);

impl Plugin for KayakContextPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowSize::default())
            .insert_resource(CustomEventReader(ManualEventReader::<
                bevy::window::CursorMoved,
            >::default()))
            .insert_resource(CustomEventReader(ManualEventReader::<
                bevy::input::mouse::MouseButtonInput,
            >::default()))
            .insert_resource(CustomEventReader(ManualEventReader::<
                bevy::input::mouse::MouseWheel,
            >::default()))
            .insert_resource(CustomEventReader(ManualEventReader::<
                bevy::window::ReceivedCharacter,
            >::default()))
            .insert_resource(CustomEventReader(ManualEventReader::<
                bevy::input::keyboard::KeyboardInput,
            >::default()))
            .add_plugin(crate::camera::KayakUICameraPlugin)
            .add_plugin(crate::render::BevyKayakUIRenderPlugin)
            .register_type::<Node>()
            .add_system(crate::input::process_events.in_base_set(CoreSet::Update))
            .add_system(update_widgets_sys.in_base_set(CoreSet::PostUpdate))
            .add_system(
                calculate_ui
                    .after(update_widgets_sys)
                    .in_base_set(CoreSet::PostUpdate),
            )
            .add_system(crate::window_size::update_window_size);

        // Register reflection types.
        // A bit annoying..
        app.register_type::<ComputedStyles>()
            .register_type::<KStyle>()
            .register_type::<KChildren>()
            .register_type::<WidgetName>()
            .register_type::<StyleProp<Color>>()
            .register_type::<StyleProp<Corner<f32>>>()
            .register_type::<StyleProp<Edge<f32>>>()
            .register_type::<StyleProp<Units>>()
            .register_type::<StyleProp<KCursorIcon>>()
            .register_type::<StyleProp<String>>()
            .register_type::<StyleProp<f32>>()
            .register_type::<StyleProp<LayoutType>>()
            .register_type::<StyleProp<Edge<Units>>>()
            .register_type::<StyleProp<PointerEvents>>()
            .register_type::<StyleProp<KPositionType>>()
            .register_type::<StyleProp<RenderCommand>>()
            .register_type::<StyleProp<i32>>();
    }
}

fn calculate_ui(world: &mut World) {
    // dbg!("Calculating nodes!");

    let mut context_data = Vec::new();

    query_world::<Query<(Entity, &mut EventDispatcher, &mut KayakRootContext)>, _, _>(
        |mut query| {
            for (entity, mut event_dispatcher, mut kayak_root_context) in query.iter_mut() {
                context_data.push((
                    entity,
                    std::mem::take(&mut *event_dispatcher),
                    std::mem::take(&mut *kayak_root_context),
                ));
            }
        },
        world,
    );

    for (entity, event_dispatcher, mut context) in context_data.drain(..) {
        let mut node_system = IntoSystem::into_system(calculate_nodes);
        node_system.initialize(world);
        let mut layout_system = IntoSystem::into_system(calculate_layout);
        layout_system.initialize(world);

        for _ in 0..2 {
            context = node_system.run(context, world);
            node_system.apply_buffers(world);

            context = layout_system.run(context, world);
            layout_system.apply_buffers(world);
            LayoutEventDispatcher::dispatch(&mut context, world);
        }

        if event_dispatcher.hovered.is_none() {
            context.current_cursor = CursorIcon::Default;
        } else {
            let hovered = event_dispatcher.hovered.unwrap();
            if let Some(entity) = world.get_entity(hovered.0) {
                if let Some(node) = entity.get::<crate::node::Node>() {
                    let icon = node.resolved_styles.cursor.resolve();
                    context.current_cursor = icon.0;
                }
            }

            if let Ok(mut window) = world
                .query_filtered::<&mut Window, With<PrimaryWindow>>()
                .get_single_mut(world)
            {
                window.cursor.icon = context.current_cursor;
            }
        }

        world.entity_mut(entity).insert((event_dispatcher, context));
    }
}

/// A simple component that stores the type name of a widget
/// This is used by Kayak in order to find out which systems to run.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
pub struct WidgetName(pub String);

impl Default for WidgetName {
    fn default() -> Self {
        log::warn!("You did not specify a widget name for a widget!");
        Self("NO_NAME".to_string())
    }
}

impl From<String> for WidgetName {
    fn from(value: String) -> Self {
        WidgetName(value)
    }
}

impl Into<String> for WidgetName {
    fn into(self) -> String {
        self.0
    }
}
