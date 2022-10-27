use bevy::prelude::{Bundle, Component, Entity, Handle, In, Query};

use crate::{
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

/// Renders a bevy image asset within the GUI
/// The rendered image respects the styles.
#[derive(Component, PartialEq, Clone, Default)]
pub struct Image(pub Handle<bevy::prelude::Image>);

impl Widget for Image {}

#[derive(Bundle)]
pub struct KImageBundle {
    pub image: Image,
    pub style: KStyle,
    pub widget_name: WidgetName,
}

impl Default for KImageBundle {
    fn default() -> Self {
        Self {
            image: Default::default(),
            style: Default::default(),
            widget_name: Image::default().get_name(),
        }
    }
}

pub fn image_render(
    In((_widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut query: Query<(&mut KStyle, &Image)>,
) -> bool {
    if let Ok((mut style, image)) = query.get_mut(entity) {
        style.render_command = StyleProp::Value(RenderCommand::Image {
            handle: image.0.clone_weak(),
        });
    }
    true
}
