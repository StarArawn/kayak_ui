use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query};

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{KStyle, RenderCommand, StyleProp, Units},
    widget::Widget,
};

#[derive(Component, PartialEq, Clone, Default)]
pub struct Clip;

impl Widget for Clip {}

/// Clips are used to "clip" or cut away sections of the screen.
/// For example text inside of another widget likely should not
/// overflow out of the widget's bounds. This widget will cut or clip
/// the text.
/// Note: Clips roughly translate to wGPU scissor commands.
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
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    _: Commands,
    mut query: Query<(&mut KStyle, &KChildren)>,
) -> bool {
    if let Ok((mut styles, children)) = query.get_mut(entity) {
        styles.render_command = StyleProp::Value(RenderCommand::Clip);
        children.process(&widget_context, Some(entity));
    }
    true
}
