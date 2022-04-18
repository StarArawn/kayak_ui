use bevy::{
    math::Vec2,
    prelude::{Assets, Res},
    sprite::Rect,
};
use kayak_core::render_primitive::RenderPrimitive;
use kayak_font::{Alignment, KayakFont, TextProperties};

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

    let properties = TextProperties {
        alignment: Alignment::Start,
        font_size,
        line_height: *line_height,
        max_size: (parent_size.0, parent_size.1),
        ..Default::default()
    };

    let text_layout = font.measure(
        content,
        properties,
    );

    let base_position = Vec2::new(layout.posx, layout.posy + font_size);

    for glyph_rect in text_layout.glyphs() {
        let mut position = Vec2::from(glyph_rect.position);
        position += base_position;

        let size = Vec2::from(glyph_rect.size);

        extracted_texts.push(ExtractQuadBundle {
            extracted_quad: ExtractedQuad {
                font_handle: Some(font_handle.clone()),
                rect: Rect {
                    min: position,
                    max: position + size,
                },
                color: to_bevy_color(background_color),
                vertex_index: 0,
                char_id: font.get_char_id(glyph_rect.content).unwrap(),
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
