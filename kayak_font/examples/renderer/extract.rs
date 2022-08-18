use bevy::{
    math::Vec2,
    prelude::{Assets, Commands, Handle, Query, Res},
    render::Extract,
    sprite::Rect,
};
use kayak_font::{KayakFont, TextProperties};

use super::{
    pipeline::{ExtractCharBundle, ExtractedChar},
    Text,
};

pub fn extract(
    mut commands: Commands,
    fonts: Extract<Res<Assets<KayakFont>>>,
    texts: Extract<Query<(&Text, &Handle<KayakFont>)>>,
) {
    let mut extracted_texts = Vec::new();

    for (text, font_handle) in texts.iter() {
        if let Some(font) = fonts.get(font_handle) {
            let properties = TextProperties {
                font_size: text.font_size,
                line_height: text.line_height,
                max_size: (text.size.x, text.size.y),
                alignment: text.horz_alignment,
                ..Default::default()
            };

            let text_layout = font.measure(&text.content, properties);

            for glyph_rect in text_layout.glyphs() {
                let mut position = Vec2::from(glyph_rect.position);
                position.y *= -1.0;
                position += text.position;

                let size = Vec2::from(glyph_rect.size);

                extracted_texts.push(ExtractCharBundle {
                    extracted_quad: ExtractedChar {
                        font_handle: Some(font_handle.clone()),
                        rect: Rect {
                            min: position,
                            max: position + size,
                        },
                        color: text.color,
                        vertex_index: 0,
                        char_id: font.get_char_id(glyph_rect.content).unwrap(),
                        z_index: 0.0,
                    },
                });
            }
        }
    }

    commands.spawn_batch(extracted_texts);
}
