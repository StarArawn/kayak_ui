use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query, Res};

use crate::{
    children::KChildren,
    context::WidgetName,
    on_event::OnEvent,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, RenderCommand, StyleProp},
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
    pub computed_styles: ComputedStyles,
    pub on_event: OnEvent,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for ElementBundle {
    fn default() -> Self {
        Self {
            element: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            children: Default::default(),
            on_event: OnEvent::default(),
            widget_name: Element::default().get_name(),
        }
    }
}

pub fn element_render(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<(&KStyle, &mut ComputedStyles, &KChildren)>,
) -> bool {
    if let Ok((style, mut computed_styles, children)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(style)
            .with_style(KStyle {
                render_command: StyleProp::Value(RenderCommand::Layout),
                ..Default::default()
            })
            .into();
        children.process(&widget_context, &mut commands, Some(entity));
    }
    true
}
