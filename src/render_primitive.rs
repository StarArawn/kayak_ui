use crate::{
    layout::Rect,
    styles::{BoxShadow, Corner, Edge, KStyle, RenderCommand, StyleProp},
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
        opacity_layer: u32,
    },
    Quad {
        layout: Rect,
        background_color: Color,
        border_color: Color,
        border: Edge<f32>,
        border_radius: Corner<f32>,
        opacity_layer: u32,
        box_shadow: Option<Vec<BoxShadow>>,
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
        opacity_layer: u32,
    },
    Image {
        border_radius: Corner<f32>,
        layout: Rect,
        handle: Handle<Image>,
        opacity_layer: u32,
    },
    TextureAtlas {
        size: Vec2,
        position: Vec2,
        layout: Rect,
        handle: Handle<Image>,
        opacity_layer: u32,
    },
    NinePatch {
        border: Edge<f32>,
        layout: Rect,
        handle: Handle<Image>,
        opacity_layer: u32,
    },
    Svg {
        handle: Handle<Svg>,
        background_color: Option<Color>,
        layout: Rect,
        opacity_layer: u32,
    },
    OpacityLayer {
        index: u32,
        z: f32,
    },
    DrawOpacityLayer {
        opacity: f32,
        index: u32,
        z: f32,
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
            RenderPrimitive::DrawOpacityLayer { layout, .. } => *layout = new_layout,
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
            RenderPrimitive::DrawOpacityLayer { layout, .. } => *layout,
            _ => Rect::default(),
        }
    }

    pub fn get_opacity_layer(&self) -> u32 {
        match self {
            RenderPrimitive::Clip { opacity_layer, .. } => *opacity_layer,
            RenderPrimitive::Quad { opacity_layer, .. } => *opacity_layer,
            RenderPrimitive::Text { opacity_layer, .. } => *opacity_layer,
            RenderPrimitive::Image { opacity_layer, .. } => *opacity_layer,
            RenderPrimitive::NinePatch { opacity_layer, .. } => *opacity_layer,
            RenderPrimitive::TextureAtlas { opacity_layer, .. } => *opacity_layer,
            RenderPrimitive::Svg { opacity_layer, .. } => *opacity_layer,
            RenderPrimitive::OpacityLayer { index, .. } => *index,
            RenderPrimitive::DrawOpacityLayer { index, .. } => *index,
            _ => 0,
        }
    }

    pub fn set_opacity_layer(&mut self, layer: u32) {
        match self {
            RenderPrimitive::Clip { opacity_layer, .. } => *opacity_layer = layer,
            RenderPrimitive::Quad { opacity_layer, .. } => *opacity_layer = layer,
            RenderPrimitive::Text { opacity_layer, .. } => *opacity_layer = layer,
            RenderPrimitive::Image { opacity_layer, .. } => *opacity_layer = layer,
            RenderPrimitive::NinePatch { opacity_layer, .. } => *opacity_layer = layer,
            RenderPrimitive::TextureAtlas { opacity_layer, .. } => *opacity_layer = layer,
            RenderPrimitive::Svg { opacity_layer, .. } => *opacity_layer = layer,
            RenderPrimitive::OpacityLayer { index, .. } => *index = layer,
            RenderPrimitive::DrawOpacityLayer { index, .. } => *index = layer,
            _ => (),
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
            RenderPrimitive::OpacityLayer { .. } => "OpacityLayer".into(),
            RenderPrimitive::DrawOpacityLayer { .. } => "DrawOpacityLayer".into(),
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
                opacity_layer: 0,
            },
            RenderCommand::Quad => Self::Quad {
                background_color,
                border_color,
                border_radius: style.border_radius.resolve(),
                border: style.border.resolve(),
                layout: Rect::default(),
                box_shadow: match style.box_shadow.clone() {
                    StyleProp::Value(v) => Some(v),
                    _ => None,
                },
                opacity_layer: 0,
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
                opacity_layer: 0,
            },
            RenderCommand::Image { handle } => Self::Image {
                border_radius: style.border_radius.resolve(),
                layout: Rect::default(),
                handle,
                opacity_layer: 0,
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
                opacity_layer: 0,
            },
            RenderCommand::NinePatch { handle, border } => Self::NinePatch {
                border,
                layout: Rect::default(),
                handle,
                opacity_layer: 0,
            },
            RenderCommand::Svg { handle } => Self::Svg {
                background_color: match style.background_color {
                    StyleProp::Value(color) => Some(color),
                    _ => None,
                },
                handle,
                layout: Rect::default(),
                opacity_layer: 0,
            },
        }
    }
}
