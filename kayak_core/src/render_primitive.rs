use morphorm::layout;
use kayak_font::{TextLayout, TextProperties};
use crate::{
    color::Color,
    layout_cache::Rect,
    render_command::RenderCommand,
    styles::{Corner, Edge, Style},
};

#[derive(Debug, Clone, PartialEq)]
pub enum RenderPrimitive {
    Empty,
    Clip {
        layout: Rect,
    },
    Quad {
        layout: Rect,
        background_color: Color,
        border_color: Color,
        border: Edge<f32>,
        border_radius: Corner<f32>,
    },
    Text {
        color: Color,
        content: String,
        font: String,
        text_layout: TextLayout,
        layout: Rect,
        properties: TextProperties,
    },
    Image {
        border_radius: Corner<f32>,
        layout: Rect,
        handle: u16,
    },
    NinePatch {
        border: Edge<f32>,
        layout: Rect,
        handle: u16,
    },
}

impl RenderPrimitive {
    pub fn set_layout(&mut self, new_layout: Rect) {
        match self {
            RenderPrimitive::Clip { layout, .. } => *layout = new_layout,
            RenderPrimitive::Quad { layout, .. } => *layout = new_layout,
            RenderPrimitive::Text { layout, .. } => *layout = new_layout,
            RenderPrimitive::Image { layout, .. } => *layout = new_layout,
            RenderPrimitive::NinePatch { layout, .. } => *layout = new_layout,
            _ => (),
        }
    }
}

impl From<&Style> for RenderPrimitive {
    fn from(style: &Style) -> Self {
        let render_command = style.render_command.resolve();

        let background_color = style.background_color.resolve_or(Color::TRANSPARENT);

        let border_color = style.border_color.resolve_or(Color::TRANSPARENT);

        let font = style
            .font
            .resolve_or_else(|| String::from(crate::DEFAULT_FONT));

        let font_size = style.font_size.resolve_or(14.0);

        let line_height = style.line_height.resolve_or(font_size * 1.2);

        match render_command {
            RenderCommand::Empty => Self::Empty,
            RenderCommand::Layout => Self::Empty,
            RenderCommand::Clip => Self::Clip {
                layout: Rect::default(),
            },
            RenderCommand::Quad => Self::Quad {
                background_color,
                border_color,
                border_radius: style.border_radius.resolve(),
                border: style.border.resolve(),
                layout: Rect::default(),
            },
            RenderCommand::Text { content } => Self::Text {
                color: style.color.resolve(),
                content,
                font,
                text_layout: TextLayout::default(),
                layout: Rect::default(),
                properties: TextProperties {
                    font_size,
                    line_height,
                    ..Default::default()
                },
            },
            RenderCommand::Image { handle } => Self::Image {
                border_radius: style.border_radius.resolve(),
                layout: Rect::default(),
                handle,
            },
            RenderCommand::NinePatch { handle, border } => Self::NinePatch {
                border,
                layout: Rect::default(),
                handle,
            },
        }
    }
}
