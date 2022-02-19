use bevy::{
    math::Vec2,
    prelude::{Assets, Res},
    sprite::Rect,
};
use kayak_core::render_primitive::RenderPrimitive;
use kayak_font::{Alignment, CoordinateSystem, KayakFont};

use crate::to_bevy_color;
use bevy_kayak_renderer::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    Corner,
};

use super::font_mapping::FontMapping;

pub fn extract_texts(
    render_primitive: &RenderPrimitive,
    fonts: &Res<Assets<KayakFont>>,
    font_mapping: &Res<FontMapping>,
    _dpi: f32,
) -> Vec<ExtractQuadBundle> {
    let mut extracted_texts = Vec::new();
    let (background_color, layout, font_size, content, font, parent_size, line_height) =
        match render_primitive {
            RenderPrimitive::Text {
                color,
                layout,
                size,
                content,
                font,
                parent_size,
                line_height,
            } => (
                color,
                layout,
                *size,
                content,
                font,
                parent_size,
                line_height,
            ),
            _ => panic!(""),
        };

    let font_handle = font_mapping.get_handle(font.clone()).unwrap();
    let font = fonts.get(font_handle.clone());

    if font.is_none() {
        return vec![];
    }

    let font = font.unwrap();

    let chars_layouts = font.get_layout(
        CoordinateSystem::PositiveYDown,
        Alignment::Start,
        (layout.posx, layout.posy + font_size),
        (parent_size.0, parent_size.1),
        content,
        *line_height,
        font_size,
    );

    for char_layout in chars_layouts {
        let position = char_layout.position.into();
        let size: Vec2 = char_layout.size.into();
        extracted_texts.push(ExtractQuadBundle {
            extracted_quad: ExtractedQuad {
                font_handle: Some(font_handle.clone()),
                rect: Rect {
                    min: position,
                    max: position + size,
                },
                color: to_bevy_color(background_color),
                vertex_index: 0,
                char_id: font.get_char_id(char_layout.content).unwrap(),
                z_index: layout.z_index,
                quad_type: UIQuadType::Text,
                type_index: 0,
                border_radius: Corner::default(),
                image: None,
                uv_max: None,
                uv_min: None,
            },
        });
    }

    extracted_texts
}
