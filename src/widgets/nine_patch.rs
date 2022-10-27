use bevy::prelude::{Bundle, Commands, Component, Entity, Handle, Image, In, Query};

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{Edge, KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

#[derive(Component, PartialEq, Clone, Default, Debug)]
pub struct NinePatch {
    /// The handle to image
    pub handle: Handle<Image>,
    /// The size of each edge (in pixels)
    pub border: Edge<f32>,
}

impl Widget for NinePatch {}

///
/// Render's a nine-patch image as a UI widget.
///
/// Also know as 9-slicing. This 2D technique allows users to render UI images at multiple
/// resolutions while maintaining a level of quality. The image in the middle is repeated.
///
/// Accepts Children and Styles.
///
/// Example: The border prop splits up the image into 9 quadrants like so:
/// 1----2----3
/// |         |
/// 4    9    5
/// |         |
/// 6----7----8
/// The sizes of sprites for a 15 pixel border are as follows:
/// TopLeft = (15, 15)
/// TopRight = (15, 15)
/// LeftCenter = (15, image_height)
/// RightCenter = (15, image_height)
/// TopCenter = (image_width, 15)
/// BottomCenter = (image_width, 15)
/// BottomRight = (15, 15)
/// BottomLeft = (15, 15)
/// Middle = (
/// 30 being left border + right border
///   image_width - 30
/// 30 being top border + bottom border
///   image_height - 30
/// )
#[derive(Bundle)]
pub struct NinePatchBundle {
    pub nine_patch: NinePatch,
    pub styles: KStyle,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for NinePatchBundle {
    fn default() -> Self {
        Self {
            nine_patch: Default::default(),
            styles: Default::default(),
            children: KChildren::default(),
            widget_name: NinePatch::default().get_name(),
        }
    }
}

pub fn nine_patch_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    _: Commands,
    mut query: Query<(&mut KStyle, &NinePatch, &KChildren)>,
) -> bool {
    if let Ok((mut style, nine_patch, children)) = query.get_mut(entity) {
        style.render_command = StyleProp::Value(RenderCommand::NinePatch {
            border: nine_patch.border,
            handle: nine_patch.handle.clone_weak(),
        });

        children.process(&widget_context, Some(entity));

        return true;
    }
    false
}
