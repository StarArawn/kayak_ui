use kayak_core::OnLayout;

use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children, OnEvent, WidgetProps,
};

/// Props used by the [`NinePatch`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TextureAtlasProps {
    /// The handle to image
    pub handle: u16,
    /// The position of the tile (in pixels)
    pub position: (f32, f32),
    /// The size of the tile (in pixels)
    pub tile_size: (f32, f32),
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(OnLayout)]
    pub on_layout: Option<OnLayout>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

#[widget]
/// A widget that renders a texture atlas
/// Allows for the use of a partial square of an image such as in a sprite sheet
/// 
/// # Props
///
/// __Type:__ [`TextureAtlasProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ✅        |
///
pub fn TextureAtlas(props: TextureAtlasProps) {
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::TextureAtlas {
            position: props.position,
            size: props.tile_size,
            handle: props.handle,
        }),
        ..props.styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
