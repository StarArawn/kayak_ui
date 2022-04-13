use crate::{
    color::Color,
    layout_cache::Rect,
    render_command::RenderCommand,
    styles::{Corner, Edge, Style, StyleProp},
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
        layout: Rect,
        line_height: f32,
        parent_size: (f32, f32),
        size: f32,
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

        let background_color = if matches!(style.background_color, StyleProp::Default) {
            Color::TRANSPARENT
        } else {
            style.background_color.resolve()
        };

        let border_color = if matches!(style.border_color, StyleProp::Default) {
            Color::TRANSPARENT
        } else {
            style.border_color.resolve()
        };

        let font = if matches!(style.font, StyleProp::Default) {
            String::from(crate::DEFAULT_FONT)
        } else {
            style.font.resolve()
        };

        let font_size = if matches!(style.font_size, StyleProp::Default) {
            14.0
        } else {
            style.font_size.resolve()
        };

        let line_height = if matches!(style.line_height, StyleProp::Default) {
            font_size * 1.2
        } else {
            style.line_height.resolve()
        };

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
                layout: Rect::default(),
                line_height,
                parent_size: (0.0, 0.0),
                size: font_size,
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
