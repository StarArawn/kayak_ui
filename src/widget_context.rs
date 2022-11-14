use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Commands, Component, Entity},
    utils::HashMap,
};
use morphorm::Hierarchy;

use crate::{
    context_entities::ContextEntities, layout::LayoutCache, node::WrappedIndex, prelude::Tree,
    widget_state::WidgetState,
};

/// KayakWidgetContext manages tree, state, and context updates within a single widget.
/// Unlike the root context this manages a single widget and it's children.
/// At the end of a render system call KayakWidgetContext will be consumed by the root context.
/// It has some knowledge about the existing tree and it knows about a subset of the new tree.
/// It is not possible to create a KayakWidgetContext from scratch. One will be provided
/// to the render system via it's In parameters.
#[derive(Clone)]
pub struct KayakWidgetContext {
    old_tree: Arc<RwLock<Tree>>,
    new_tree: Arc<RwLock<Tree>>,
    context_entities: ContextEntities,
    layout_cache: Arc<RwLock<LayoutCache>>,
    pub(crate) index: Arc<RwLock<HashMap<Entity, usize>>>,
    widget_state: WidgetState,
    order_tree: Arc<RwLock<Tree>>,
    pub camera_entity: Option<Entity>,
}

impl KayakWidgetContext {
    pub(crate) fn new(
        old_tree: Arc<RwLock<Tree>>,
        context_entities: ContextEntities,
        layout_cache: Arc<RwLock<LayoutCache>>,
        widget_state: WidgetState,
        order_tree: Arc<RwLock<Tree>>,
        index: Arc<RwLock<HashMap<Entity, usize>>>,
        camera_entity: Option<Entity>,
    ) -> Self {
        Self {
            old_tree,
            new_tree: Arc::new(RwLock::new(Tree::default())),
            context_entities,
            layout_cache,
            index,
            widget_state,
            order_tree,
            camera_entity,
        }
    }

    pub(crate) fn store(&self, new_tree: &Tree) {
        if let Ok(mut tree) = self.new_tree.write() {
            *tree = new_tree.clone();
        }
    }

    /// Creates a new context using the context entity for the given type_id + parent id.
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

    /// Finds the closest matching context entity by traversing up the tree.
    pub fn get_context_entity<T: Default + 'static>(
        &self,
        current_entity: Entity,
    ) -> Option<Entity> {
        // Check self first..
        if let Some(entity) = self
            .context_entities
            .get_context_entity::<T>(current_entity)
        {
            return Some(entity);
        }

        // Check parents
        if let Ok(tree) = self.old_tree.read() {
            let mut parent = tree.get_parent(WrappedIndex(current_entity));
            while parent.is_some() {
                if let Some(entity) = self
                    .context_entities
                    .get_context_entity::<T>(parent.unwrap().0)
                {
                    return Some(entity);
                }
                parent = tree.get_parent(parent.unwrap());
            }
        }

        None
    }

    pub(crate) fn copy_from_point(&self, other_tree: &Arc<RwLock<Tree>>, entity: WrappedIndex) {
        if let Ok(other_tree) = other_tree.read() {
            if let Ok(mut tree) = self.new_tree.write() {
                tree.copy_from_point(&other_tree, entity);
            }
        }
    }

    /// Removes all children from the new tree.
    /// Changes to the current tree will happen when KayakWidgetContext
    /// is consumed.
    pub fn clear_children(&self, entity: Entity) {
        if let Ok(mut tree) = self.new_tree.write() {
            tree.children.insert(WrappedIndex(entity), vec![]);
        }
    }

    /// Retrieves a list of all children.
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        let mut children = vec![];
        if let Ok(tree) = self.new_tree.read() {
            let iterator = tree.child_iter(WrappedIndex(entity));

            children = iterator.map(|index| index.0).collect::<Vec<_>>();
        }

        children
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

    /// Creates or grabs the existing state entity
    pub fn use_state<State: Component + PartialEq + Clone + Default>(
        &self,
        commands: &mut Commands,
        widget_entity: Entity,
        initial_state: State,
    ) -> Entity {
        self.widget_state
            .add(commands, widget_entity, initial_state)
    }

    /// Grabs the existing state returns none if it does not exist.
    pub fn get_state(&self, widget_entity: Entity) -> Option<Entity> {
        self.widget_state.get(widget_entity)
    }

    /// Returns a new/existing widget entity.
    /// Because a re-render can potentially spawn new entities it's advised to use this
    /// to avoid creating a new entity.
    pub fn spawn_widget(&self, commands: &mut Commands, parent_id: Option<Entity>) -> Entity {
        let mut entity = None;
        if let Some(parent_entity) = parent_id {
            let children = self.get_children_ordered(parent_entity);
            let index = self.get_and_add_index(parent_entity);
            let child = children.get(index).cloned();
            if let Some(child) = child {
                log::trace!(
                    "Reusing widget entity {:?} with parent: {:?}!",
                    child.index(),
                    parent_id.unwrap().index()
                );
                entity = Some(commands.get_or_spawn(child).id());
            }
        }

        // If we have no entity spawn it!
        if entity.is_none() {
            entity = Some(commands.spawn_empty().id());
            log::trace!(
                "Spawning new widget with entity {:?}!",
                entity.unwrap().index()
            );

            // We need to add it to the ordered tree
            if let Ok(mut tree) = self.order_tree.try_write() {
                tree.add(WrappedIndex(entity.unwrap()), parent_id.map(WrappedIndex))
            }
        }
        entity.unwrap()
    }

    /// Removes all matching children from the tree.
    pub fn remove_children(&self, children_to_remove: Vec<Entity>) {
        if let Ok(mut tree) = self.new_tree.write() {
            for child in children_to_remove.iter() {
                tree.remove(WrappedIndex(*child));
            }
        }
    }

    /// Adds a new widget to the tree with a given parent.
    pub fn add_widget(&self, parent: Option<Entity>, entity: Entity) {
        if let Some(parent) = parent {
            assert!(parent != entity, "Parent cannot equal entity!");
        }
        if let Ok(mut tree) = self.new_tree.write() {
            tree.add(WrappedIndex(entity), parent.map(WrappedIndex));
        }
    }

    /// Attempts to get the layout rect for the widget with the given ID
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the widget
    ///
    pub fn get_layout(&self, widget_id: Entity) -> Option<crate::layout::Rect> {
        if let Ok(cache) = self.layout_cache.try_read() {
            cache.rect.get(&WrappedIndex(widget_id)).cloned()
        } else {
            None
        }
    }

    /// Dumps the tree to the console in a human readable format.
    /// This is relatively slow to do if the tree is large
    /// so avoid doing unless necessary.
    pub fn dbg_tree(&self) {
        if let Ok(tree) = self.new_tree.read() {
            tree.dump()
        }
    }

    /// Consumes the tree
    pub(crate) fn take(self) -> Tree {
        Arc::try_unwrap(self.new_tree)
            .unwrap()
            .into_inner()
            .unwrap()
    }
}
