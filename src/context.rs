use std::sync::{Arc, RwLock};

use bevy::{
    ecs::{event::ManualEventReader, system::CommandQueue},
    prelude::*,
    utils::{HashMap, HashSet},
    window::PrimaryWindow,
};
use kayak_font::KayakFont;
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
    node::{DirtyNode, WrappedIndex},
    prelude::KayakWidgetContext,
    render::{
        font::FontMapping,
        unified::pipeline::{ExtractedQuad, ExtractedQuads, UIQuadType},
        MAX_OPACITY_LAYERS,
    },
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
        Box<dyn System<In = (Entity, Entity), Out = bool>>,
        Box<dyn System<In = Entity, Out = bool>>,
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
            return tree.current().map(|a| a.0);
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
        update: impl IntoSystem<(Entity, Entity), bool, Params>,
        render: impl IntoSystem<Entity, bool, Params2>,
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
    pub fn spawn_widget<'a>(
        &self,
        commands: &mut Commands,
        key: Option<&'a str>,
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
                        "Reusing child {} as widget entity {:?} with parent: {:?}!",
                        index,
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
            } else if let Ok(_tree) = self.order_tree.try_write() {
                // let root_node = tree.root_node;
                // if entity.map(WrappedIndex) != root_node {
                //     tree.add(entity.map(WrappedIndex).unwrap(), root_node);
                //     // Don't forget to advance indices or weird stuff can happen.
                //     if let Some(parent_entity) = root_node {
                //         self.get_and_add_index(parent_entity.0);
                //     }
                // }
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
        commands: &mut Commands,
        camera_entity: Entity,
        dpi: f32,
        nodes: &Query<&crate::node::Node>,
        widget_names: &Query<&WidgetName>,
        fonts: &Assets<KayakFont>,
        font_mapping: &FontMapping,
        images: &Assets<Image>,
        extracted_quads: &mut ExtractedQuads,
    ) {
        let node_tree = self.tree.try_read();
        if node_tree.is_err() {
            return;
        }

        let node_tree = node_tree.unwrap();

        if node_tree.root_node.is_none() {
            return;
        }

        if let Ok(mut layout_cache) = self.layout_cache.try_write() {
            recurse_node_tree_to_build_primitives(
                commands,
                camera_entity,
                dpi,
                &node_tree,
                &mut layout_cache,
                nodes,
                widget_names,
                fonts,
                font_mapping,
                images,
                extracted_quads,
                node_tree.root_node.unwrap(),
                0.0,
                0.0,
                None,
                0,
                0,
            );
        }
    }
}

pub const UI_Z_STEP: f32 = 0.001;

fn recurse_node_tree_to_build_primitives(
    commands: &mut Commands,
    camera_entity: Entity,
    dpi: f32,
    node_tree: &Tree,
    layout_cache: &mut LayoutCache,
    nodes: &Query<&crate::node::Node>,
    widget_names: &Query<&WidgetName>,
    fonts: &Assets<KayakFont>,
    font_mapping: &FontMapping,
    images: &Assets<Image>,
    extracted_quads: &mut ExtractedQuads,
    current_node: WrappedIndex,
    _parent_global_z: f32,
    mut current_global_z: f32,
    mut prev_clip: Option<ExtractedQuad>,
    mut current_opacity_layer: u32,
    mut total_opacity_layers: u32,
) -> (usize, f32, u32) {
    let mut opacity = None;
    let mut child_count = 0;
    if let Ok(node) = nodes.get(current_node.0) {
        // Skip rendering completely transparent objects.
        if node.opacity < 0.001 {
            return (0, current_global_z, total_opacity_layers);
        }
        current_global_z += UI_Z_STEP + if node.z <= 0.0 { 0.0 } else { node.z };
        // Set opacity layer on render primitive

        // let mut new_z_index = main_z_index;
        let new_z = current_global_z; // - parent_global_z;

        let layout = if let Some(layout) = layout_cache.rect.get_mut(&current_node) {
            log::trace!(
                "z_index is {} and node.z is {} for: {}-{}",
                new_z,
                node.z,
                widget_names.get(current_node.0).unwrap().0,
                current_node.0.index(),
            );
            layout.z_index = new_z;

            // new_z_index += if node.z <= 0.0 { 0.0 } else { node.z };

            *layout
        } else {
            log::warn!(
                "No layout for node: {}-{}",
                widget_names.get(current_node.0).unwrap().0,
                current_node.0.index()
            );
            Rect::default()
        };

        let new_clip = node.resolved_styles.extract(
            commands,
            &layout,
            current_opacity_layer,
            extracted_quads,
            camera_entity,
            fonts,
            font_mapping,
            images,
            dpi,
            prev_clip.clone(),
        );

        // Only spawn an opacity layer if we have an opacity greater than zero or less than one.
        if node.opacity < 1.0 {
            // If we've hit max opacity layer capacity skip rendering.
            if total_opacity_layers + 1 >= MAX_OPACITY_LAYERS {
                return (0, current_global_z, total_opacity_layers);
            }

            // Add in an opacity layer
            total_opacity_layers += 1;
            extracted_quads.quads.push(ExtractedQuad {
                camera_entity,
                z_index: layout.z_index,
                quad_type: UIQuadType::OpacityLayer,
                opacity_layer: total_opacity_layers,
                ..Default::default()
            });
            opacity = Some((node.opacity, total_opacity_layers));
            current_opacity_layer = total_opacity_layers;
        }

        // let _indent = "  ".repeat(depth);
        // if new_clip.is_some() {
        // println!("{} [{:?} current_global_z: {}, z: {}, x: {}, y: {}, width: {}, height: {}]", _indent, extracted_quads.quads.last().unwrap_or(&ExtractedQuad::default()).quad_type, current_global_z, layout.z_index, layout.posx, layout.posy, layout.width, layout.height);
        // }

        prev_clip = match &new_clip {
            Some(new_clip) => Some(new_clip.clone()),
            None => prev_clip.clone(),
        };
        if node_tree.children.contains_key(&current_node) {
            let current_parent_global_z = current_global_z;
            let children = node_tree.children.get(&current_node).unwrap();
            for child in children {
                let (new_child_count, new_global_z, new_total_opacity_layers) =
                    recurse_node_tree_to_build_primitives(
                        commands,
                        camera_entity,
                        dpi,
                        node_tree,
                        layout_cache,
                        nodes,
                        widget_names,
                        fonts,
                        font_mapping,
                        images,
                        extracted_quads,
                        *child,
                        current_parent_global_z,
                        current_global_z,
                        prev_clip.clone(),
                        current_opacity_layer,
                        total_opacity_layers,
                    );
                current_global_z = new_global_z;
                child_count += new_child_count;
                total_opacity_layers = new_total_opacity_layers;

                // Between each child node we need to reset the clip.
                if let Some(prev_clip) = &prev_clip {
                    current_global_z += UI_Z_STEP * 2.0; // * child_count as f32;
                    extracted_quads.quads.push(ExtractedQuad {
                        z_index: current_global_z,
                        ..prev_clip.clone()
                    });
                    // println!("{}   [previous_clip, z: {}, x: {}, y: {}, width: {}, height: {}", _indent, current_global_z, layout.posx, layout.posy, layout.width, layout.height);
                }
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

    // When an opacity layer has been added all of its children are drawn to the same render target.
    // After we need to draw the render target for that opacity layer to the screen.
    if let Some((opacity, opacity_layer)) = opacity {
        let root_node_layout = layout_cache
            .rect
            .get(&node_tree.root_node.unwrap())
            .unwrap();
        current_global_z += UI_Z_STEP * 2.0;
        extracted_quads.quads.push(ExtractedQuad {
            camera_entity,
            z_index: current_global_z,
            color: Color::rgba(1.0, 1.0, 1.0, opacity),
            opacity_layer,
            quad_type: UIQuadType::DrawOpacityLayer,
            rect: bevy::prelude::Rect {
                min: Vec2::new(root_node_layout.posx, root_node_layout.posy),
                max: Vec2::new(
                    root_node_layout.posx + root_node_layout.width,
                    root_node_layout.posy + root_node_layout.height,
                ),
            },
            ..Default::default()
        });
    }

    (child_count + 1, current_global_z, total_opacity_layers)
}

/// Updates the widgets
pub fn update_widgets_sys(world: &mut World) {
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

        let tick = world.read_change_tick();

        for (key, system) in context.systems.iter_mut() {
            if let Some(new_tick) = new_ticks.get(key) {
                system.0.set_last_change_tick(*new_tick);
                system.1.set_last_change_tick(*new_tick);
            } else {
                system.0.set_last_change_tick(tick);
                system.1.set_last_change_tick(tick);
            }
        }

        // Clear out indices
        if let Ok(mut indices) = context.index.try_write() {
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
        if let (Some(entity_ref), Some(_)) = (
            world.get_entity(entity.0),
            tree.try_write()
                .ok()
                .map(|tree| tree.contains(*entity).clone()),
        ) {
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
                        let diff = tree.diff_children(&widget_context, *entity, 0);
                        for (_index, child, _parent, changes) in diff.changes.iter() {
                            if changes
                                .iter()
                                .any(|change| matches!(change, Change::Inserted))
                            {
                                if let Ok(mut cache) = layout_cache.try_write() {
                                    cache.add(*child);
                                }
                            }
                        }

                        // Children of this node need to be despawned.
                        let mut despawn_list = Vec::default();
                        'outer: for (_index, changed_entity, parent, changes) in diff.changes.iter()
                        {
                            // If a tree node goes from A to B we need to know and delete the descendants.
                            if let Ok(previous_entities) = cloned_widget_entities.read() {
                                if let Some(previous_entity) =
                                    previous_entities.get(&changed_entity.0)
                                {
                                    if let (Some(entity_ref), Some(prev_entity_ref)) = (
                                        world.get_entity(changed_entity.0),
                                        world.get_entity(*previous_entity),
                                    ) {
                                        if let (Some(widget_name), Some(prev_widget_name)) = (
                                            entity_ref.get::<WidgetName>(),
                                            prev_entity_ref.get::<WidgetName>(),
                                        ) {
                                            if widget_name != prev_widget_name {
                                                if let Some(_) = tree.parent(*changed_entity) {
                                                    for child in
                                                        tree.down_iter_at(*changed_entity, false)
                                                    {
                                                        trace!(
                                                            "Removing AvsB children {}::{}",
                                                            entity_ref
                                                                .get::<WidgetName>()
                                                                .map(|n| n.0.clone())
                                                                .unwrap_or("Unknown".into()),
                                                            changed_entity.0.index()
                                                        );
                                                        let mut should_delete = true;
                                                        if let Ok(order_tree) =
                                                            order_tree.try_read()
                                                        {
                                                            'back_up: for sibling in order_tree
                                                                .child_iter(
                                                                    order_tree
                                                                        .parent(*changed_entity)
                                                                        .unwrap(),
                                                                )
                                                            {
                                                                for child in
                                                                    tree.down_iter_at(sibling, true)
                                                                {
                                                                    if let Some(entity_ref) =
                                                                        world.get_entity(child.0)
                                                                    {
                                                                        if let Some(children) =
                                                                            entity_ref
                                                                                .get::<KChildren>()
                                                                        {
                                                                            if children
                                                                                .contains_entity(
                                                                                    changed_entity
                                                                                        .0,
                                                                                )
                                                                            {
                                                                                trace!("Caught an entity that was marked as deleted but wasn't! {:?}", changed_entity.0);
                                                                                // Don't despawn changed entity because it exists as a child passed via props
                                                                                should_delete =
                                                                                    false;
                                                                                break 'back_up;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        if should_delete {
                                                            despawn_list.push((parent.0, child.0));
                                                            if let Ok(mut order_tree) =
                                                                order_tree.try_write()
                                                            {
                                                                order_tree.remove(*changed_entity);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if changes.iter().any(|change| *change == Change::Inserted) {
                                if let Some(mut entity_commands) =
                                    world.get_entity_mut(changed_entity.0)
                                {
                                    entity_commands.insert(Mounted);
                                    entity_commands.set_parent(parent.0);
                                }
                                if world.get_entity(changed_entity.0).is_some() {
                                    if let Some(mut entity_commands) =
                                        world.get_entity_mut(parent.0)
                                    {
                                        entity_commands.add_child(changed_entity.0);
                                    }
                                }
                            } else if changes
                                .iter()
                                .any(|change| matches!(change, Change::Deleted))
                            {
                                // If the child exists as a child of one of the children we do not need to remove it.
                                // TODO: This is kinda of expensive we should think of a way of making this faster..
                                if let Ok(order_tree) = order_tree.try_read() {
                                    for sibling in order_tree
                                        .child_iter(order_tree.parent(*changed_entity).unwrap())
                                    {
                                        for child in tree.down_iter_at(sibling, true) {
                                            if let Some(entity_ref) = world.get_entity(child.0) {
                                                if let Some(children) =
                                                    entity_ref.get::<KChildren>()
                                                {
                                                    if children.contains_entity(changed_entity.0) {
                                                        trace!("Caught an entity that was marked as deleted but wasn't! {:?}", changed_entity.0);
                                                        // Don't despawn changed entity because it exists as a child passed via props
                                                        continue 'outer;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                trace!(
                                    "Trying to remove: {:?} with parent {:?}",
                                    changed_entity.0,
                                    tree.parent(*changed_entity)
                                );
                                // Due to a bug in bevy we need to remove the parent manually otherwise we'll panic later.
                                if let Some(mut entity_mut) = world.get_entity_mut(changed_entity.0)
                                {
                                    entity_mut.remove_parent();
                                }
                                if let Some(parent) = tree.parent(*changed_entity) {
                                    despawn_list.push((parent.0, changed_entity.0));
                                }

                                if let Ok(mut order_tree) = order_tree.try_write() {
                                    order_tree.remove(*changed_entity);
                                }
                            }
                        }

                        // let had_change = diff.has_changes();
                        // if had_change {
                        //     println!("Tree Before:");
                        //     tree.dump_all_at(Some(world), entity.0);
                        //     println!("Changes:");
                        //     diff.debug_print(world);
                        // }

                        tree.merge(&widget_context, *entity, diff, UPDATE_DEPTH);

                        // if had_change {
                        //     println!("Tree After:");
                        //     tree.dump_all_at(Some(world), entity.0);
                        // }

                        for (parent, entity) in despawn_list.drain(..) {
                            // Clear out keyed entity.
                            if let (Ok(mut unique_ids), Ok(mut unique_ids_parents)) =
                                (unique_ids.try_write(), unique_ids_parents.try_write())
                            {
                                if let Some(parent) = unique_ids_parents.get(&entity) {
                                    if let Some(keyed_hashmap) = unique_ids.get_mut(parent) {
                                        let possible_key = keyed_hashmap
                                            .iter()
                                            .find(|(_, keyed_entity)| **keyed_entity == entity)
                                            .map(|(key, _)| key.clone());
                                        if let Some(key) = possible_key {
                                            keyed_hashmap.remove(&key);
                                            unique_ids_parents.remove(&entity);
                                            log::trace!(
                                                "Removing key {key}, for entity: {:?}",
                                                entity
                                            );
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
                                );
                                entity_mut.despawn();

                                // Also remove all cloned widget entities
                                if let Ok(cloned_widget_entities) =
                                    cloned_widget_entities.try_read()
                                {
                                    if let Some(entity) = cloned_widget_entities.get(&entity) {
                                        world.despawn(*entity);
                                    }
                                }
                            }
                        }

                        // if should_update_children {
                        if let Ok(cloned_widget_entities) = cloned_widget_entities.try_read() {
                            if let Some(target_entity) = cloned_widget_entities.get(&entity.0) {
                                if let Some(styles) =
                                    world.entity(entity.0).get::<KStyle>().cloned()
                                {
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
                            }
                        }

                        // Mark nodes left as dirty.
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

            // If the child exists as a child of one of the children we do not need to remove it.
            // TODO: This is kinda of expensive we should think of a way of making this faster..
            let mut contained_in_children = false;
            if let Ok(tree) = tree.read() {
                if let Ok(order_tree) = order_tree.try_read() {
                    if let Some(parent) = order_tree.parent(*entity) {
                        'outside_loop: for sibling in order_tree.child_iter(parent) {
                            for child in tree.down_iter_at(sibling, true) {
                                if let Some(entity_ref) = world.get_entity(child.0) {
                                    if let Some(children) = entity_ref.get::<KChildren>() {
                                        if children.contains_entity(entity.0) {
                                            trace!("Caught an entity that was marked as deleted but wasn't! {:?}", entity.0);
                                            // Don't despawn changed entity because it exists as a child passed via props
                                            contained_in_children = true;
                                            break 'outside_loop;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !contained_in_children {
                if let Ok(mut tree) = tree.write() {
                    if let Ok(mut order_tree) = order_tree.try_write() {
                        log::trace!("Removing dangling entity! {:?}", entity.0.index());
                        order_tree.remove(*entity);
                    }
                    tree.remove(*entity);
                    if let Some(entity_mut) = world.get_entity_mut(entity.0) {
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
    mut widget_context: KayakWidgetContext,
    previous_children: Vec<Entity>,
    clone_systems: &Arc<RwLock<EntityCloneSystems>>,
    cloned_widget_entities: &Arc<RwLock<HashMap<Entity, Entity>>>,
    widget_state: &WidgetState,
    new_ticks: &mut HashMap<String, u32>,
    _order_tree: &Arc<RwLock<Tree>>,
    _unique_ids: &Arc<RwLock<HashMap<Entity, HashMap<String, Entity>>>>,
    _unique_ids_parents: &Arc<RwLock<HashMap<Entity, Entity>>>,
) -> (Tree, bool) {
    // Check if we should update this widget
    let should_rerender = {
        // TODO: Move the spawning to when we create the widget.
        let old_props_entity =
            if let Ok(mut cloned_widget_entities) = cloned_widget_entities.try_write() {
                let old_parent_entity = if let Ok(tree) = tree.try_read() {
                    if let Some(parent_entity) = tree.get_parent(entity) {
                        cloned_widget_entities.get(&parent_entity.0).copied()
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(entity) = cloned_widget_entities.get(&entity.0).copied() {
                    if let Some(possible_entity) = world.get_entity(entity) {
                        let target = possible_entity.id();
                        cloned_widget_entities.insert(entity, target);
                        target
                    } else {
                        let target = world.spawn_empty().insert(PreviousWidget).id();
                        if let Some(parent_id) = old_parent_entity {
                            world.entity_mut(parent_id).add_child(target);
                        }
                        cloned_widget_entities.insert(entity, target);
                        target
                    }
                } else {
                    let target = world.spawn_empty().insert(PreviousWidget).id();
                    if let Some(parent_id) = old_parent_entity {
                        world.entity_mut(parent_id).add_child(target);
                    }
                    cloned_widget_entities.insert(entity.0, target);
                    target
                }
            } else {
                panic!("Couldn't get write lock!")
            };

        let widget_update_system = &mut systems
            .get_mut(&widget_type)
            .unwrap_or_else(|| {
                panic!(
                    "Wasn't able to find render/update systems for widget: {}!",
                    widget_type
                )
            })
            .0;
        let old_tick = widget_update_system.get_last_change_tick();

        // Insert context as a bevy resource.
        world.insert_resource(widget_context);
        let should_rerender = widget_update_system.run((entity.0, old_props_entity), world);
        let new_tick = widget_update_system.get_last_change_tick();
        new_ticks.insert(widget_type.clone(), new_tick);
        widget_update_system.set_last_change_tick(old_tick);
        widget_update_system.apply_buffers(world);

        // Extract context
        widget_context = world.remove_resource::<KayakWidgetContext>().unwrap();

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
        world.insert_resource(widget_context.clone());
        should_update_children = widget_render_system.run(entity.0, world);
        let new_tick = widget_render_system.get_last_change_tick();
        new_ticks.insert(widget_type.clone(), new_tick);
        widget_render_system.set_last_change_tick(old_tick);
        widget_render_system.apply_buffers(world);
        world.remove_resource::<KayakWidgetContext>();

        if let Ok(mut indices) = widget_context.index.try_write() {
            indices.insert(entity.0, 0);
        }
    }
    let widget_context = widget_context.take();
    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, world);

    commands.entity(entity.0).remove::<Mounted>();

    // Always mark widget dirty if it's re-rendered.
    // Mark node as needing a recalculation of rendering/layout.
    commands.entity(entity.0).insert(DirtyNode);

    command_queue.apply(world);

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
        app //.register_type::<Node>()
            .register_type::<ComputedStyles>()
            .register_type::<KStyle>()
            .register_type::<KChildren>()
            .register_type::<crate::layout::Rect>()
            .register_type::<crate::node::Node>()
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

impl From<WidgetName> for String {
    fn from(val: WidgetName) -> Self {
        val.0
    }
}
