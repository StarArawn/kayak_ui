use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Commands, Component, Entity},
    utils::HashMap,
};

/// Stores mappings between widget entities and their corresponding state entities.
#[derive(Default, Debug, Clone)]
pub struct WidgetState {
    mapping: Arc<RwLock<HashMap<Entity, Entity>>>,
}

impl WidgetState {
    /// Attempts to create a state entity or return the existing entity.
    pub fn add<State: Component + PartialEq + Clone>(
        &self,
        commands: &mut Commands,
        widget_entity: Entity,
        initial_state: State,
    ) -> Entity {
        if let Ok(mut mapping) = self.mapping.try_write() {
            if mapping.contains_key(&widget_entity) {
                *mapping.get(&widget_entity).unwrap()
            } else {
                let state_entity = commands.spawn(initial_state).id();
                mapping.insert(widget_entity, state_entity);
                state_entity
            }
        } else {
            panic!("Couldn't get mapping lock!");
        }
    }

    /// Attempts to get a state entity
    pub fn get(&self, widget_entity: Entity) -> Option<Entity> {
        if let Ok(mapping) = self.mapping.try_read() {
            return mapping.get(&widget_entity).cloned();
        }

        None
    }
}
