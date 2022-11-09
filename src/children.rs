use bevy::prelude::*;

use crate::prelude::KayakWidgetContext;

/// Defers widgets being added to the widget tree.
#[derive(Component, Debug, Default, Clone, PartialEq)]
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
        self.inner.get(index).and_then(|e| Some(*e))
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

    pub fn despawn(&mut self, commands: &mut Commands) {
        for child in self.inner.drain(..) {
            commands.entity(child).despawn_recursive();
        }
    }

    /// Processes all widgets and adds them to the tree.
    pub fn process(&self, widget_context: &KayakWidgetContext, parent_id: Option<Entity>) {
        for child in self.inner.iter() {
            widget_context.add_widget(parent_id, *child);
        }
    }
}
