use bevy::prelude::*;

use crate::prelude::KayakWidgetContext;

/// Defers widgets being added to the widget tree.
#[derive(Component, Reflect, Debug, Default, Clone, PartialEq, Eq)]
#[reflect(Component)]
pub struct KChildren {
    inner: Vec<Entity>,
}

impl KChildren {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Adds a widget entity to child storage.
    pub fn add(&mut self, widget_entity: Entity) {
        self.inner.push(widget_entity);
    }

    pub fn get(&self, index: usize) -> Option<Entity> {
        self.inner.get(index).copied()
    }

    pub fn remove(&mut self, index: usize) -> Option<Entity> {
        if index < self.inner.len() {
            Some(self.inner.remove(index))
        } else {
            None
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Entity> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    pub fn despawn(&mut self, commands: &mut Commands) {
        for child in self.inner.drain(..) {
            commands.entity(child).despawn_recursive();
        }
    }

    /// Processes all widgets and adds them to the tree.
    pub fn process(
        &self,
        widget_context: &KayakWidgetContext,
        commands: &mut Commands,
        parent_id: Option<Entity>,
    ) {
        for child in self.inner.iter() {
            if let Some(parent_id) = parent_id {
                if let Some(mut entity_commands) = commands.get_entity(*child) {
                    entity_commands.set_parent(parent_id);
                }
            }
            widget_context.add_widget(parent_id, *child);
        }
    }
}
