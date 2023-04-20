use crate::{
    layout::Rect,
    styles::{Corner, Edge, KStyle, RenderCommand, StyleProp},
};
use bevy::{
    prelude::{Color, Handle, Image, Vec2},
    reflect::Reflect,
};
use bevy_svg::prelude::Svg;
use kayak_font::{TextLayout, TextProperties};

#[derive(Debug, Reflect, Clone, PartialEq)]
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
        word_wrap: bool,
        subpixel: bool,
    },
    Image {
        border_radius: Corner<f32>,
        layout: Rect,
        handle: Handle<Image>,
    },
    TextureAtlas {
        size: Vec2,
        position: Vec2,
        layout: Rect,
        handle: Handle<Image>,
    },
    NinePatch {
        border: Edge<f32>,
        layout: Rect,
        handle: Handle<Image>,
    },
    Svg {
        handle: Handle<Svg>,
        background_color: Option<Color>,
        layout: Rect,
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
            RenderPrimitive::TextureAtlas { layout, .. } => *layout = new_layout,
            RenderPrimitive::Svg { layout, .. } => *layout = new_layout,
            _ => (),
        }
    }

    pub fn get_layout(&self) -> Rect {
        match self {
            RenderPrimitive::Clip { layout, .. } => *layout,
            RenderPrimitive::Quad { layout, .. } => *layout,
            RenderPrimitive::Text { layout, .. } => *layout,
            RenderPrimitive::Image { layout, .. } => *layout,
            RenderPrimitive::NinePatch { layout, .. } => *layout,
            RenderPrimitive::TextureAtlas { layout, .. } => *layout,
            RenderPrimitive::Svg { layout, .. } => *layout,
            _ => Rect::default(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RenderPrimitive::Clip { .. } => "Clip".into(),
            RenderPrimitive::Quad { .. } => "Quad".into(),
            RenderPrimitive::Text { .. } => "Text".into(),
            RenderPrimitive::Image { .. } => "Image".into(),
            RenderPrimitive::NinePatch { .. } => "NinePatch".into(),
            RenderPrimitive::TextureAtlas { .. } => "TextureAtlas".into(),
            RenderPrimitive::Svg { .. } => "Svg".into(),
            RenderPrimitive::Empty { .. } => "Empty".into(),
        }
    }
}

impl From<&KStyle> for RenderPrimitive {
    fn from(style: &KStyle) -> Self {
        let render_command = style.render_command.resolve();

        let background_color = style
            .background_color
            .resolve_or(Color::rgba(1.0, 1.0, 1.0, 0.0));

        let border_color = style
            .border_color
            .resolve_or(Color::rgba(1.0, 1.0, 1.0, 0.0));

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
            RenderCommand::Text {
                content,
                alignment,
                word_wrap,
                subpixel,
            } => Self::Text {
                color: style.color.resolve(),
                content,
                font,
                text_layout: TextLayout::default(),
                layout: Rect::default(),
                properties: TextProperties {
                    font_size,
                    line_height,
                    alignment,
                    ..Default::default()
                },
                word_wrap,
                subpixel,
            },
            RenderCommand::Image { handle } => Self::Image {
                border_radius: style.border_radius.resolve(),
                layout: Rect::default(),
                handle,
            },
            RenderCommand::TextureAtlas {
                handle,
                size,
                position,
            } => Self::TextureAtlas {
                handle,
                layout: Rect::default(),
                size,
                position,
            },
            RenderCommand::NinePatch { handle, border } => Self::NinePatch {
                border,
                layout: Rect::default(),
                handle,
            },
            RenderCommand::Svg { handle } => Self::Svg {
                background_color: match style.background_color {
                    StyleProp::Value(color) => Some(color),
                    _ => None,
                },
                handle,
                layout: Rect::default(),
            },
        }
    }
}
