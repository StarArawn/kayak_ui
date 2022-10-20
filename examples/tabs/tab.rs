use bevy::prelude::{
    Bundle, ChangeTrackers, Changed, Color, Commands, Component, Entity, In, ParamSet, Query,
};
use kayak_ui::prelude::{
    widgets::BackgroundBundle, Edge, KChildren, KStyle, StyleProp, Units, Widget, WidgetContext,
    WidgetName,
};
use kayak_ui_macros::rsx;

use crate::tab_context::TabContext;

#[derive(Component, Default)]
pub struct Tab {
    pub index: usize,
}

impl Widget for Tab {}

#[derive(Bundle)]
pub struct TabBundle {
    pub tab: Tab,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for TabBundle {
    fn default() -> Self {
        Self {
            tab: Default::default(),
            children: Default::default(),
            widget_name: Tab::default().get_name(),
        }
    }
}

pub fn tab_update(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&KChildren, &mut Tab)>,
    mut tab_context_query: ParamSet<(
        Query<ChangeTrackers<TabContext>>,
        Query<&mut TabContext, Changed<TabContext>>,
    )>,
) -> bool {
    if !tab_context_query.p1().is_empty() {
        if let Ok((children, tab)) = query.get_mut(entity) {
            let context_entity = widget_context
                .get_context_entity::<TabContext>(entity)
                .unwrap();
            if let Ok(tab_context) = tab_context_query.p1().get(context_entity) {
                let parent_id = Some(entity);
                let styles = KStyle {
                    background_color: StyleProp::Value(Color::rgba(0.0781, 0.0898, 0.101, 1.0)),
                    padding: StyleProp::Value(Edge::all(Units::Pixels(5.0))),
                    ..Default::default()
                };
                if tab_context.current_index == tab.index {
                    rsx! {
                        <BackgroundBundle styles={styles} children={children.clone()} />
                    }
                }
                return true;
            }
        }
    }
    false
}
