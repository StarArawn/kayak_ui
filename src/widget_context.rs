use std::sync::{Arc, RwLock};

use bevy::{prelude::Entity, utils::HashMap};
use morphorm::Hierarchy;

use crate::{
    context_entities::ContextEntities, layout::LayoutCache, node::WrappedIndex, prelude::Tree,
};

#[derive(Clone)]
pub struct WidgetContext {
    old_tree: Arc<RwLock<Tree>>,
    new_tree: Arc<RwLock<Tree>>,
    context_entities: ContextEntities,
    layout_cache: Arc<RwLock<LayoutCache>>,
    index: Arc<RwLock<HashMap<Entity, usize>>>,
}

impl WidgetContext {
    pub(crate) fn new(
        old_tree: Arc<RwLock<Tree>>,
        context_entities: ContextEntities,
        layout_cache: Arc<RwLock<LayoutCache>>,
    ) -> Self {
        Self {
            old_tree,
            new_tree: Arc::new(RwLock::new(Tree::default())),
            context_entities,
            layout_cache,
            index: Arc::new(RwLock::new(HashMap::default())),
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

    pub fn clear_children(&self, entity: Entity) {
        if let Ok(mut tree) = self.new_tree.write() {
            tree.children.insert(WrappedIndex(entity), vec![]);
        }
    }

    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        let mut children = vec![];
        if let Ok(tree) = self.new_tree.read() {
            let iterator = tree.child_iter(WrappedIndex(entity));

            children = iterator.map(|index| index.0).collect::<Vec<_>>();
        }

        children
    }

    fn get_children_old(&self, entity: Entity) -> Vec<Entity> {
        let mut children = vec![];
        if let Ok(tree) = self.old_tree.read() {
            let iterator = tree.child_iter(WrappedIndex(entity));

            children = iterator.map(|index| index.0).collect::<Vec<_>>();
        }

        children
    }

    fn get_and_add_index(&self, parent: Entity) -> usize {
        if let Ok(mut hash_map) = self.index.try_write() {
            if hash_map.contains_key(&parent) {
                let index = hash_map.get_mut(&parent).unwrap();
                let current_index = index.clone();
                *index += 1;
                return current_index;
            } else {
                hash_map.insert(parent, 1);
                return 0;
            }
        }

        0
    }

    pub fn get_child_at(&self, entity: Option<Entity>) -> Option<Entity> {
        if let Some(entity) = entity {
            let children = self.get_children_old(entity);
            return children.get(self.get_and_add_index(entity)).cloned();
        }
        None
    }

    pub fn remove_children(&self, children_to_remove: Vec<Entity>) {
        if let Ok(mut tree) = self.new_tree.write() {
            for child in children_to_remove.iter() {
                tree.remove(WrappedIndex(*child));
            }
        }
    }

    pub fn add_widget(&self, parent: Option<Entity>, entity: Entity) {
        if let Ok(mut tree) = self.new_tree.write() {
            tree.add(
                WrappedIndex(entity),
                parent.map(|parent| WrappedIndex(parent)),
            );
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

    pub fn dbg_tree(&self) {
        if let Ok(tree) = self.new_tree.read() {
            tree.dump()
        }
    }

    pub fn take(self) -> Tree {
        Arc::try_unwrap(self.new_tree)
            .unwrap()
            .into_inner()
            .unwrap()
    }
}
