use bevy::{
    ecs::system::SystemParam,
    prelude::{Changed, Component, Entity, In, Query, With},
};

use crate::{
    children::KChildren,
    context::{Mounted, WidgetName},
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle},
};

pub trait Widget: Send + Sync {
    fn get_name(&self) -> WidgetName {
        WidgetName(std::any::type_name::<Self>().into())
    }
}

#[derive(Component, Default, PartialEq, Eq, Clone)]
pub struct EmptyState;

pub fn widget_update<Props: PartialEq + Component + Clone, State: PartialEq + Component + Clone>(
    In((widget_context, entity, previous_entity)): In<(KayakWidgetContext, Entity, Entity)>,
    widget_param: WidgetParam<Props, State>,
) -> bool {
    widget_param.has_changed(&widget_context, entity, previous_entity)
}

pub fn widget_update_with_context<
    Props: PartialEq + Component + Clone,
    State: PartialEq + Component + Clone,
    Context: PartialEq + Component + Clone + Default,
>(
    In((widget_context, entity, previous_entity)): In<(KayakWidgetContext, Entity, Entity)>,
    widget_param: WidgetParam<Props, State>,
    context_query: Query<Entity, Changed<Context>>,
) -> bool {
    // Uses bevy state changes to see if context has changed.
    if let Some(context_entity) = widget_context.get_context_entity::<Context>(entity) {
        if context_query.contains(context_entity) {
            log::trace!(
                "Entity context: {} has changed! {}-{}",
                std::any::type_name::<Context>(),
                widget_param.widget_names.get(entity).unwrap().0,
                entity.index()
            );
            return true;
        }
    }

    widget_param.has_changed(&widget_context, entity, previous_entity)
}

#[derive(SystemParam)]
pub struct WidgetParam<'w, 's, Props: PartialEq + Component, State: PartialEq + Component> {
    pub props_query: Query<'w, 's, &'static Props>,
    pub old_props_query: Query<'w, 's, &'static Props>,
    pub mounted_query: Query<'w, 's, Entity, With<Mounted>>,
    pub style_query: Query<'w, 's, &'static KStyle>,
    pub computed_style_query: Query<'w, 's, &'static ComputedStyles>,
    pub children_query: Query<'w, 's, &'static KChildren>,
    pub state_query: Query<'w, 's, &'static State>,
    pub widget_names: Query<'w, 's, &'static WidgetName>,
}

impl<'w, 's, Props: PartialEq + Component, State: PartialEq + Component>
    WidgetParam<'w, 's, Props, State>
{
    pub fn has_changed(
        &self,
        widget_context: &KayakWidgetContext,
        current_entity: Entity,
        previous_entity: Entity,
    ) -> bool {
        if !self.mounted_query.is_empty() {
            log::trace!(
                "Entity was mounted! {}-{}",
                self.widget_names.get(current_entity).unwrap().0,
                current_entity.index()
            );
            return true;
        }

        // Compare the widget names. Sometimes our widget entity changes with the same ID but differing components.
        if let (Ok(name), Ok(old_name)) = (
            self.widget_names.get(current_entity),
            self.widget_names.get(previous_entity),
        ) {
            if name != old_name {
                return true;
            }
        }

        // Compare styles
        if let (Ok(style), Ok(old_style)) = (
            self.style_query.get(current_entity),
            self.style_query.get(previous_entity),
        ) {
            if style != old_style {
                log::trace!(
                    "Entity styles have changed! {}-{}",
                    self.widget_names.get(current_entity).unwrap().0,
                    current_entity.index()
                );
                return true;
            }
        }

        // Compare computed styles
        if let (Ok(style), Ok(old_style)) = (
            self.computed_style_query.get(current_entity),
            self.computed_style_query.get(previous_entity),
        ) {
            if style != old_style {
                log::trace!(
                    "Entity computed styles have changed! {}-{}",
                    self.widget_names.get(current_entity).unwrap().0,
                    current_entity.index()
                );
                return true;
            }
        }

        // Compare children
        // If children don't exist ignore as mount will add them!
        if let (Ok(children), Ok(old_children)) = (
            self.children_query.get(current_entity),
            self.children_query.get(previous_entity),
        ) {
            if children != old_children {
                log::trace!(
                    "Entity children have changed! {}-{}",
                    self.widget_names.get(current_entity).unwrap().0,
                    current_entity.index()
                );
                return true;
            }
        }

        // Check props
        if let (Ok(props), Ok(previous_props)) = (
            self.props_query.get(current_entity),
            self.old_props_query.get(previous_entity),
        ) {
            if previous_props != props {
                log::trace!(
                    "Entity props have changed! {}-{}",
                    self.widget_names.get(current_entity).unwrap().0,
                    current_entity.index()
                );
                return true;
            }
        }

        // Check state
        let previous_state_entity = widget_context.get_state(previous_entity);
        let current_state_entity = widget_context.get_state(current_entity);

        // Check if state was nothing but now is something
        if current_state_entity.is_some() && previous_state_entity.is_none() {
            return true;
        }

        // Check state
        if current_state_entity.is_some() && previous_state_entity.is_some() {
            let previous_state_entity = previous_state_entity.unwrap();
            let current_state_entity = current_state_entity.unwrap();
            if let (Ok(state), Ok(previous_state)) = (
                self.state_query.get(current_state_entity),
                self.state_query.get(previous_state_entity),
            ) {
                if previous_state != state {
                    return true;
                }
            }
        }

        false
    }
}
