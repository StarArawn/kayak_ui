use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Edge, Style, StyleProp},
    widget, Children, OnEvent, WidgetProps,
};

/// Props used by the [`NinePatch`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct NinePatchProps {
    /// The handle to image
    pub handle: u16,
    /// The size of each edge (in pixels)
    pub border: Edge<f32>,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

#[widget]
/// A widget that renders a nine-patch image background
///
/// A nine-patch is a special type of image that's broken into nine parts:
///
/// * Edges - Top, Bottom, Left, Right
/// * Corners - Top-Left, Top-Right, Bottom-Left, Bottom-Right
/// * Center
///
/// Using these parts of an image, we can construct a scalable background and border
/// all from a single image. This is done by:
///
/// * Stretching the edges (vertically for left/right and horizontally for top/bottom)
/// * Preserving the corners
/// * Scaling the center to fill the remaining space
///
///
/// # Props
///
/// __Type:__ [`NinePatchProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `focusable` | ✅        |
///
pub fn NinePatch(props: NinePatchProps) {
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::NinePatch {
            handle: props.handle,
            border: props.border,
        }),
        ..props.styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
