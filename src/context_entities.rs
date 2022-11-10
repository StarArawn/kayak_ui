use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use bevy::prelude::Entity;
use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct ContextEntities {
    ce: Arc<DashMap<Entity, DashMap<TypeId, Entity>>>,
}

impl ContextEntities {
    pub fn new() -> Self {
        Self {
            ce: Arc::new(DashMap::new()),
        }
    }

    pub fn add_context_entity<T: Default + 'static>(
        &self,
        parent_id: Entity,
        context_entity: Entity,
    ) {
        if !self.ce.contains_key(&parent_id) {
            self.ce.insert(parent_id, DashMap::new());
        }
        let inner = self.ce.get(&parent_id).unwrap();
        inner.insert(T::default().type_id(), context_entity);
    }

    pub fn get_context_entity<T: Default + 'static>(&self, parent_id: Entity) -> Option<Entity> {
        if !self.ce.contains_key(&parent_id) {
            return None;
        }
        let inner = self.ce.get(&parent_id).unwrap();
        inner.get(&T::default().type_id()).map(|e| *e)
    }
}
