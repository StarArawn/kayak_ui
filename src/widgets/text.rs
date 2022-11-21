use bevy::prelude::*;
use kayak_font::Alignment;

use crate::{
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{KCursorIcon, KStyle, RenderCommand, StyleProp, Units},
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
    /// Custom styles to pass in.
    pub user_styles: KStyle,
    /// Basic word wrapping.
    /// Defautls to true
    pub word_wrap: bool,
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
            user_styles: Default::default(),
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
    pub widget_name: WidgetName,
}

impl Default for TextWidgetBundle {
    fn default() -> Self {
        Self {
            text: Default::default(),
            styles: KStyle {
                width: Units::Stretch(1.0).into(),
                height: Units::Stretch(1.0).into(),
                ..Default::default()
            },
            widget_name: TextProps::default().get_name(),
        }
    }
}

pub fn text_render(
    In((_widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut query: Query<(&mut KStyle, &TextProps)>,
) -> bool {
    if let Ok((mut styles, text)) = query.get_mut(entity) {
        *styles = KStyle::default()
            .with_style(&text.user_styles)
            .with_style(KStyle {
                render_command: StyleProp::Value(RenderCommand::Text {
                    content: text.content.clone(),
                    alignment: text.alignment,
                    word_wrap: text.word_wrap,
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
                // bottom: Units::Stretch(1.0).into(),
                // top: Units::Stretch(1.0).into(),
                // left: Units::Stretch(0.0).into(),
                // right: Units::Stretch(0.0).into(),
                ..Default::default()
            });

        // style.cursor = StyleProp::Value(KCursorIcon(CursorIcon::Hand));
    }

    true
}
