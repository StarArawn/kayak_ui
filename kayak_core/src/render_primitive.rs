use crate::{
    color::Color,
    layout_cache::{Rect, Space},
    render_command::RenderCommand,
    styles::{Style, StyleProp},
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
        border: (f32, f32, f32, f32),
        border_radius: (f32, f32, f32, f32),
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
        border_radius: (f32, f32, f32, f32),
        layout: Rect,
        handle: u16,
    },
    NinePatch {
        border: Space,
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
            RenderCommand::Text {
                content,
                font,
                line_height,
                parent_size,
                size,
            } => Self::Text {
                color: style.color.resolve(),
                content,
                font,
                layout: Rect::default(),
                line_height,
                parent_size,
                size,
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
