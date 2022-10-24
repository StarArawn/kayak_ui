use bevy::prelude::{Bundle, Changed, Component, Entity, Handle, In, Or, Query, With};

use crate::{
    context::{Mounted, WidgetName},
    prelude::WidgetContext,
    styles::{KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

#[derive(Component, PartialEq, Clone, Default)]
pub struct Image(pub Handle<bevy::prelude::Image>);

impl Widget for Image {}

#[derive(Bundle)]
pub struct ImageBundle {
    pub image: Image,
    pub style: KStyle,
    pub widget_name: WidgetName,
}

impl Default for ImageBundle {
    fn default() -> Self {
        Self {
            image: Default::default(),
            style: Default::default(),
            widget_name: Image::default().get_name(),
        }
    }
}

pub fn image_render(
    In((_widget_context, entity)): In<(WidgetContext, Entity)>,
    mut query: Query<(&mut KStyle, &Image), Or<((Changed<Image>, Changed<KStyle>), With<Mounted>)>>,
) -> bool {
    if let Ok((mut style, image)) = query.get_mut(entity) {
        style.render_command = StyleProp::Value(RenderCommand::Image {
            handle: image.0.clone_weak(),
        });
        return true;
    }
    false
}
