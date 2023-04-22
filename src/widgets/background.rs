use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query, Res};

use crate::{
    children::KChildren,
    context::WidgetName,
    on_event::OnEvent,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, RenderCommand},
    widget::Widget,
};

#[derive(Component, PartialEq, Eq, Clone, Default)]
pub struct Background;

impl Widget for Background {}

/// Background Widget
///
/// The name of this widget is slightly misleading.
/// In actuality this widget renders a quad or multiple quads if a border is used.
/// You can customize the colors, border, border-radius, by passing in custom styles.
/// Children are rendered inside of the quad.
#[derive(Bundle)]
pub struct BackgroundBundle {
    pub background: Background,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub on_event: OnEvent,
    pub widget_name: WidgetName,
}

impl Default for BackgroundBundle {
    fn default() -> Self {
        Self {
            background: Default::default(),
            styles: Default::default(),
            computed_styles: Default::default(),
            children: Default::default(),
            on_event: Default::default(),
            widget_name: Background::default().get_name(),
        }
    }
}

pub fn background_render(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<(&KStyle, &mut ComputedStyles, &KChildren)>,
) -> bool {
    if let Ok((style, mut computed_styles, children)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(KStyle {
                render_command: RenderCommand::Quad.into(),
                ..Default::default()
            })
            .with_style(style)
            .into();
        children.process(&widget_context, &mut commands, Some(entity));
    }
    true
}
