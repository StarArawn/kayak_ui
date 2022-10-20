use bevy::prelude::{Bundle, Changed, Commands, Component, Entity, In, Query};
use kayak_ui::prelude::{KChildren, Widget, WidgetContext, WidgetName};

#[derive(Component, Default)]
pub struct TabContext {
    pub current_index: usize,
}

#[derive(Component, Default)]
pub struct TabContextProvider {
    pub initial_index: usize,
}

impl Widget for TabContextProvider {}

#[derive(Bundle)]
pub struct TabContextProviderBundle {
    pub tab_provider: TabContextProvider,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for TabContextProviderBundle {
    fn default() -> Self {
        Self {
            tab_provider: Default::default(),
            children: Default::default(),
            widget_name: TabContextProvider::default().get_name(),
        }
    }
}

pub fn tab_context_update(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<
        (&KChildren, &TabContextProvider),
        (Changed<KChildren>, Changed<TabContextProvider>),
    >,
) -> bool {
    if let Ok((children, tab_context_provider)) = query.get(entity) {
        let context_entity = commands
            .spawn(TabContext {
                current_index: tab_context_provider.initial_index,
            })
            .id();
        widget_context.set_context_entity::<TabContext>(Some(entity), context_entity);

        children.process(&widget_context, Some(entity));

        return true;
    }
    false
}
