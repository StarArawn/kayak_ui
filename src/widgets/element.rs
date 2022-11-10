use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query};

use crate::{
    children::KChildren,
    context::WidgetName,
    on_event::OnEvent,
    prelude::KayakWidgetContext,
    styles::{KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

#[derive(Component, PartialEq, Eq, Clone, Default)]
pub struct Element;

impl Widget for Element {}

/// A generic widget
/// You can consider this to kind behave like a div in html
/// Accepts: KStyle, OnEvent, and KChildren.
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

pub fn element_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
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
        children.process(&widget_context, Some(entity));
    }
    true
}
