use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query};

use crate::{
    children::KChildren,
    context::WidgetName,
    on_event::OnEvent,
    prelude::WidgetContext,
    styles::{KStyle, RenderCommand, StyleProp},
    widget::{Widget, WidgetProps},
};

#[derive(Component, PartialEq, Clone, Default)]
pub struct Element;

impl Widget for Element {}
impl WidgetProps for Element {}

#[derive(Bundle)]
pub struct ElementBundle {
    pub element: Element,
    pub styles: KStyle,
    pub on_event: OnEvent,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for ElementBundle {
    fn default() -> Self {
        Self {
            element: Default::default(),
            styles: Default::default(),
            children: Default::default(),
            on_event: OnEvent::default(),
            widget_name: Element::default().get_name(),
        }
    }
}

pub fn update_element(
    In((mut widget_context, entity)): In<(WidgetContext, Entity)>,
    _: Commands,
    mut query: Query<(&mut KStyle, &KChildren)>,
) -> bool {
    if let Ok((mut style, children)) = query.get_mut(entity) {
        *style = KStyle::default()
            .with_style(style.clone())
            .with_style(KStyle {
                render_command: StyleProp::Value(RenderCommand::Layout),
                ..Default::default()
            });
        children.process(&mut widget_context, Some(entity));
    }
    true
}
