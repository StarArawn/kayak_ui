use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query, Res};

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, RenderCommand, Units},
    widget::Widget,
};

#[derive(Component, PartialEq, Eq, Clone, Default)]
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
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for ClipBundle {
    fn default() -> Self {
        Self {
            clip: Clip::default(),
            styles: KStyle::default(),
            computed_styles: ComputedStyles::default(),
            children: KChildren::default(),
            widget_name: Clip::default().get_name(),
        }
    }
}

pub fn clip_render(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<(&KStyle, &mut ComputedStyles, &KChildren)>,
) -> bool {
    if let Ok((styles, mut computed_styles, children)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(KStyle {
                render_command: RenderCommand::Clip.into(),
                ..Default::default()
            })
            .with_style(styles)
            .with_style(KStyle {
                width: Units::Stretch(1.0).into(),
                height: Units::Stretch(1.0).into(),
                ..Default::default()
            })
            .into();
        children.process(&widget_context, &mut commands, Some(entity));
    }
    true
}
