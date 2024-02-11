use bevy::{
    math::Vec2,
    prelude::{Assets, Color, Entity, Rect},
};
use kayak_font::{KayakFont, TextLayout, TextProperties};

use crate::{
    render::unified::pipeline::{ExtractedQuad, UIQuadType},
    styles::Corner,
};

use super::font_mapping::FontMapping;

pub fn extract_texts(
    camera_entity: Entity,
    color: Color,
    text_layout: TextLayout,
    layout: crate::layout::Rect,
    font: String,
    properties: TextProperties,
    subpixel: bool,
    opacity_layer: u32,
    fonts: &Assets<KayakFont>,
    font_mapping: &FontMapping,
    _dpi: f32,
) -> Vec<ExtractedQuad> {
    let mut extracted_texts = Vec::new();

    let font_handle = font_mapping.get_handle(font).unwrap();
    let font = match fonts.get(&font_handle) {
        Some(font) => font,
        None => {
            return Vec::new();
        }
    };

    let forced = font_mapping.get_subpixel_forced(&font_handle);

    let base_position = Vec2::new(layout.posx, layout.posy + properties.font_size);

    for glyph_rect in text_layout.glyphs() {
        let mut position = Vec2::from(glyph_rect.position);
        position += base_position;

        let size = Vec2::from(glyph_rect.size);

        extracted_texts.push(ExtractedQuad {
            camera_entity,
            font_handle: Some(font_handle.clone()),
            rect: Rect {
                min: position,
                max: position + size,
            },
            color,
            char_id: font.get_char_id(glyph_rect.content).unwrap(),
            quad_type: if subpixel || forced {
                UIQuadType::TextSubpixel
            } else {
                UIQuadType::Text
            },
            type_index: 0,
            border_radius: Corner::default(),
            image: None,
            uv_max: None,
            uv_min: None,
            opacity_layer,
            c: glyph_rect.content,
            ..Default::default()
        });
    }

    extracted_texts
}
