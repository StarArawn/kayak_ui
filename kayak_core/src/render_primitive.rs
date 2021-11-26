use crate::{
    color::Color,
    layout_cache::Rect,
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
    },
    Text {
        layout: Rect,
        color: Color,
        size: f32,
        content: String,
        font: u16,
    },
}

impl RenderPrimitive {
    pub fn set_layout(&mut self, new_layout: Rect) {
        match self {
            RenderPrimitive::Clip { layout, .. } => *layout = new_layout,
            RenderPrimitive::Quad { layout, .. } => *layout = new_layout,
            RenderPrimitive::Text { layout, .. } => *layout = new_layout,
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

        match render_command {
            RenderCommand::Window => Self::Empty,
            RenderCommand::Empty => Self::Empty,
            RenderCommand::Clip => Self::Clip {
                layout: Rect::default(),
            },
            RenderCommand::Quad => Self::Quad {
                background_color: background_color,
                layout: Rect::default(),
            },
            RenderCommand::Text {
                content,
                size,
                font,
            } => Self::Text {
                layout: Rect::default(),
                color: style.color.resolve(),
                size,
                content,
                font,
            },
        }
    }
}
