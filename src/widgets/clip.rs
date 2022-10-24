use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query};

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::WidgetContext,
    styles::{KStyle, RenderCommand, StyleProp, Units},
    widget::Widget,
};

#[derive(Component, PartialEq, Clone, Default)]
pub struct Clip;

impl Widget for Clip {}

#[derive(Bundle)]
pub struct ClipBundle {
    pub clip: Clip,
    pub styles: KStyle,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for ClipBundle {
    fn default() -> Self {
        Self {
            clip: Clip::default(),
            styles: KStyle {
                render_command: StyleProp::Value(RenderCommand::Clip),
                height: StyleProp::Value(Units::Stretch(1.0)),
                width: StyleProp::Value(Units::Stretch(1.0)),
                ..KStyle::default()
            },
            children: KChildren::default(),
            widget_name: Clip::default().get_name(),
        }
    }
}

pub fn clip_render(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    _: Commands,
    mut query: Query<(&KStyle, &KChildren)>,
) -> bool {
    if let Ok((_, children)) = query.get_mut(entity) {
        children.process(&widget_context, Some(entity));
    }
    true
}
