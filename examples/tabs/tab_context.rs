use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query};
use kayak_ui::prelude::*;

#[derive(Component, Default, PartialEq, Eq, Clone)]
pub struct TabContext {
    pub current_index: usize,
}

#[derive(Component, Default, PartialEq, Eq, Clone)]
pub struct TabContextProvider {
    pub initial_index: usize,
}

impl Widget for TabContextProvider {}

#[derive(Bundle)]
pub struct TabContextProviderBundle {
    pub tab_provider: TabContextProvider,
    pub children: KChildren,
    pub styles: KStyle,
    pub widget_name: WidgetName,
}

impl Default for TabContextProviderBundle {
    fn default() -> Self {
        Self {
            tab_provider: Default::default(),
            children: Default::default(),
            styles: KStyle {
                ..Default::default()
            },
            widget_name: TabContextProvider::default().get_name(),
        }
    }
}

pub fn tab_context_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<(&KChildren, &TabContextProvider)>,
) -> bool {
    if let Ok((children, tab_context_provider)) = query.get(entity) {
        if widget_context
            .get_context_entity::<TabContext>(entity)
            .is_none()
        {
            let context_entity = commands
                .spawn(TabContext {
                    current_index: tab_context_provider.initial_index,
                })
                .id();
            widget_context.set_context_entity::<TabContext>(Some(entity), context_entity);
        }

        children.process(&widget_context, &mut commands, Some(entity));
    }
    true
}
