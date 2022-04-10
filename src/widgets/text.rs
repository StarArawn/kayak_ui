use crate::core::{
    render_command::RenderCommand,
    styles::{Style, StyleProp},
    CursorIcon, OnLayout,
    widget, OnEvent, WidgetProps,
};

/// Props used by the [`Text`] widget
#[derive(WidgetProps, Debug, PartialEq, Clone)]
pub struct TextProps {
    /// The string to display
    pub content: String,
    /// The name of the font to use
    ///
    /// The given font must already be loaded into the [`KayakContext`](kayak_core::KayakContext)
    pub font: Option<String>,
    /// The height of a line of text (currently in pixels)
    pub line_height: Option<f32>,
    /// The font size (in pixels)
    ///
    /// Negative values have no effect
    pub size: f32,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(OnLayout)]
    pub on_layout: Option<OnLayout>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            font: None,
            line_height: None,
            size: -1.0,
            styles: None,
            on_event: None,
            on_layout: None,
            focusable: None,
        }
    }
}

#[widget]
/// A widget that renders plain text
///
/// # Props
///
/// __Type:__ [`TextProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ❌        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ✅        |
///
pub fn Text(props: TextProps) {
    let mut styles = Style {
        render_command: StyleProp::Value(RenderCommand::Text {
            content: props.content.clone(),
        }),
        ..Default::default()
    };

    if let Some(ref font) = props.font {
        styles.font = StyleProp::Value(font.clone());
    }
    if props.size >= 0.0 {
        styles.font_size = StyleProp::Value(props.size);
    }
    if let Some(line_height) = props.line_height {
        styles.line_height = StyleProp::Value(line_height);
    }

    props.styles = Some(styles.with_style(&props.styles).with_style(Style {
        cursor: StyleProp::Value(CursorIcon::Text),
        ..Default::default()
    }));
}
