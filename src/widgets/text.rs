use kayak_core::{styles::Units, Binding, Bound, CursorIcon};
use kayak_font::{CoordinateSystem, KayakFont};

use crate::core::{
    render_command::RenderCommand,
    styles::{Style, StyleProp},
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
    pub size: f32,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

impl Default for TextProps {
    fn default() -> Self {
        TextProps {
            content: "".to_string(),
            font: Some(bevy_kayak_ui::DEFAULT_FONT.into()),
            line_height: None,
            size: 0.0,
            styles: None,
            on_event: None,
            focusable: None
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
/// | `focusable` | ✅        |
///
pub fn Text(props: TextProps) {
    let TextProps {
        content,
        font,
        line_height,
        size,
        ..
    } = props.clone();
    let font_name = font;
    let default_style = props.clone().styles.unwrap_or_default().font.resolve();
    let font: Binding<Option<KayakFont>> =
        context.get_asset(font_name.clone()
            .unwrap_or(default_style.clone()));
    context.bind(&font);
    let mut should_render = true;

    // TODO: It might be worth caching the measurement here until content changes.
    let (layout_size, parent_size) =
        if let Some(parent_id) = context.get_valid_parent(parent_id.unwrap()) {
            if let Some(layout) = context.get_layout(&parent_id) {
                if let Some(font) = font.get() {
                    let measurement = font.measure(
                        CoordinateSystem::PositiveYDown,
                        &content,
                        size,
                        line_height.unwrap_or(size * 1.2),
                        (layout.width, layout.height),
                    );
                    (measurement, (layout.width, layout.height))
                } else {
                    should_render = false;
                    ((0.0, 0.0), (layout.width, layout.height))
                }
            } else {
                should_render = false;
                ((0.0, 0.0), (0.0, 0.0))
            }
        } else {
            should_render = false;
            ((0.0, 0.0), (0.0, 0.0))
        };



    if should_render {
        let render_command = RenderCommand::Text {
            content: content.clone(),
            size,
            parent_size,
            line_height: line_height.unwrap_or(size * 1.2),
            font: font_name.clone().unwrap_or(default_style),
        };

        let styles = props.styles.clone().unwrap_or_default();
        props.styles = Some(Style {
            render_command: StyleProp::Value(render_command),
            width: StyleProp::Value(Units::Pixels(layout_size.0)),
            height: StyleProp::Value(Units::Pixels(layout_size.1)),
            cursor: StyleProp::select(&[&styles.cursor, &StyleProp::Value(CursorIcon::Text)])
                .clone(),
            ..styles
        });
    } else {
        context.mark_dirty();
    }
}
