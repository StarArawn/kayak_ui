use bevy::{prelude::*, utils::HashMap};

use crate::{
    children::KChildren, context::WidgetName, prelude::KayakWidgetContext, widget::Widget,
};

#[derive(Component, Default, Debug, Clone, PartialEq, Eq)]
pub struct WindowContext {
    order: Vec<Entity>,
    z_indices: HashMap<Entity, usize>,
}

impl WindowContext {
    pub fn add(&mut self, entity: Entity, index: usize) {
        self.order.push(entity);
        self.z_indices.insert(entity, index);
    }

    pub fn shift_to_top(&mut self, entity: Entity) {
        if let Some(index) = self.order.iter().position(|e| *e == entity) {
            self.order.remove(index);
            self.order.push(entity);
        }

        self.z_indices.clear();
        for (index, entity) in self.order.iter().enumerate() {
            self.z_indices.insert(*entity, index);
        }
    }

    pub fn get(&self, entity: Entity) -> usize {
        *self.z_indices.get(&entity).unwrap()
    }

    pub fn get_or_add(&mut self, entity: Entity) -> usize {
        if self.order.iter().any(|e| *e == entity) {
            self.get(entity)
        } else {
            self.add(entity, 0);
            self.shift_to_top(entity);
            self.get(entity)
        }
    }
}

#[derive(Component, Default, Debug, Clone, PartialEq, Eq)]
pub struct WindowContextProvider;

impl Widget for WindowContextProvider {}

#[derive(Bundle, Debug, Clone, PartialEq, Eq)]
pub struct WindowContextProviderBundle {
    pub context_provider: WindowContextProvider,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for WindowContextProviderBundle {
    fn default() -> Self {
        Self {
            context_provider: Default::default(),
            children: Default::default(),
            widget_name: WindowContextProvider::default().get_name(),
        }
    }
}

pub fn window_context_render(
    In((widget_context, window_context_entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    children_query: Query<&KChildren>,
) -> bool {
    if let Ok(children) = children_query.get(window_context_entity) {
        let context_entity = if let Some(context_entity) =
            widget_context.get_context_entity::<WindowContext>(window_context_entity)
        {
            context_entity
        } else {
            commands.spawn(WindowContext::default()).id()
        };
        widget_context
            .set_context_entity::<WindowContext>(Some(window_context_entity), context_entity);
        children.process(&widget_context, Some(window_context_entity));
    }

    true
}
