use bevy::prelude::*;
use kayak_font::Alignment;

use crate::{
    context::WidgetName,
    styles::{ComputedStyles, KCursorIcon, KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

#[derive(Component, Debug, PartialEq, Clone)]
pub struct TextProps {
    /// The string to display
    pub content: String,
    /// The name of the font to use
    ///
    /// The given font must already be loaded into the [`KayakContext`](kayak_core::KayakContext)
    pub font: Option<String>,
    /// The height of a line of text (currently in pixels)
    pub line_height: Option<f32>,
    /// If true, displays the default text cursor when hovered.
    ///
    /// This _will_ override the `cursor` style.
    pub show_cursor: bool,
    /// The font size (in pixels)
    ///
    /// Negative values have no effect
    pub size: f32,
    /// Text alignment.
    pub alignment: Alignment,
    /// Basic word wrapping.
    /// Defautls to true
    pub word_wrap: bool,
    /// Enables subpixel rendering of text. This is useful on smaller low-dpi screens.
    pub subpixel: bool,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            font: None,
            line_height: None,
            show_cursor: false,
            size: -1.0,
            alignment: Alignment::Start,
            word_wrap: true,
            subpixel: false,
        }
    }
}

impl Widget for TextProps {}

/// A widget that renders text
///
#[derive(Bundle)]
pub struct TextWidgetBundle {
    pub text: TextProps,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub widget_name: WidgetName,
}

impl Default for TextWidgetBundle {
    fn default() -> Self {
        Self {
            text: Default::default(),
            styles: KStyle::default(),
            computed_styles: ComputedStyles::default(),
            widget_name: TextProps::default().get_name(),
        }
    }
}

pub fn text_render(
    In(entity): In<Entity>,
    mut query: Query<(&KStyle, &mut ComputedStyles, &TextProps)>,
) -> bool {
    if let Ok((styles, mut computed_styles, text)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(styles)
            .with_style(KStyle {
                render_command: StyleProp::Value(RenderCommand::Text {
                    content: text.content.clone(),
                    alignment: text.alignment,
                    word_wrap: text.word_wrap,
                    subpixel: text.subpixel,
                }),
                font: if let Some(ref font) = text.font {
                    StyleProp::Value(font.clone())
                } else {
                    StyleProp::default()
                },
                cursor: if text.show_cursor {
                    StyleProp::Value(KCursorIcon(CursorIcon::Text))
                } else {
                    StyleProp::default()
                },
                font_size: if text.size >= 0.0 {
                    StyleProp::Value(text.size)
                } else {
                    StyleProp::default()
                },
                line_height: if let Some(line_height) = text.line_height {
                    StyleProp::Value(line_height)
                } else {
                    StyleProp::default()
                },
                ..Default::default()
            })
            .into();
        // style.cursor = StyleProp::Value(KCursorIcon(CursorIcon::Hand));
    }

    true
}
