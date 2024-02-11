use bevy::prelude::{BuildChildren, Commands, Component, Entity, Resource};
use dashmap::DashMap;
use std::sync::Arc;

/// Stores mappings between widget entities and their corresponding state entities.
#[derive(Resource, Default, Debug, Clone)]
pub struct WidgetState {
    // Widget entity to state entity
    mapping: Arc<DashMap<Entity, Entity>>,
    // State entity to widget entity
    reverse_mapping: Arc<DashMap<Entity, Entity>>,
}

impl WidgetState {
    /// Attempts to create a state entity or return the existing entity.
    pub fn add<State: Component + PartialEq + Clone>(
        &self,
        commands: &mut Commands,
        widget_entity: Entity,
        initial_state: State,
    ) -> Entity {
        if self.mapping.contains_key(&widget_entity) {
            *self.mapping.get(&widget_entity).unwrap()
        } else {
            let mut state_entity = None;
            commands
                .entity(widget_entity)
                .with_children(|child_builder| {
                    state_entity = Some(child_builder.spawn(initial_state).id());
                    self.mapping.insert(widget_entity, state_entity.unwrap());
                    self.reverse_mapping
                        .insert(state_entity.unwrap(), widget_entity);
                });
            state_entity.expect("State entity did not spawn!")
        }
    }

    /// Attempts to get a state entity
    pub fn get(&self, widget_entity: Entity) -> Option<Entity> {
        self.mapping.get(&widget_entity).map(|entry| *entry.value())
    }

    pub fn get_widget_entity(&self, state_entity: Entity) -> Option<Entity> {
        self.reverse_mapping
            .get(&state_entity)
            .map(|entry| *entry.value())
    }

    pub fn remove(&self, widget_entity: Entity) -> Option<Entity> {
        let state_entity = self.mapping.remove(&widget_entity).map(|(_, v)| v);
        if let Some(state_entity) = state_entity {
            self.reverse_mapping.remove(&state_entity);
        }
        state_entity
    }
}
