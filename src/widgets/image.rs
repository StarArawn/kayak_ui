use bevy::prelude::{Bundle, Component, Entity, Handle, In, Query};

use crate::{
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, RenderCommand},
    widget::Widget,
};

/// Renders a bevy image asset within the GUI
/// The rendered image respects the styles.
#[derive(Component, PartialEq, Eq, Clone, Default)]
pub struct KImage(pub Handle<bevy::prelude::Image>);

impl Widget for KImage {}

#[derive(Bundle)]
pub struct KImageBundle {
    pub image: KImage,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub widget_name: WidgetName,
}

impl Default for KImageBundle {
    fn default() -> Self {
        Self {
            image: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            widget_name: KImage::default().get_name(),
        }
    }
}

pub fn image_render(
    In((_widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut query: Query<(&KStyle, &mut ComputedStyles, &KImage)>,
) -> bool {
    if let Ok((style, mut computed_styles, image)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(KStyle {
                render_command: RenderCommand::Image {
                    handle: image.0.clone_weak(),
                }
                .into(),
                ..Default::default()
            })
            .with_style(style)
            .into();
    }
    true
}
