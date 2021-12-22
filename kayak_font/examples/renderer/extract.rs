use bevy::{
    math::Vec2,
    prelude::{Assets, Commands, Handle, Query, Res},
    sprite::Rect,
};
use kayak_font::{CoordinateSystem, KayakFont};

use super::{
    pipeline::{ExtractCharBundle, ExtractedChar},
    Text,
};

pub fn extract(
    mut commands: Commands,
    fonts: Res<Assets<KayakFont>>,
    texts: Query<(&Text, &Handle<KayakFont>)>,
) {
    let mut extracted_texts = Vec::new();

    for (text, font_handle) in texts.iter() {
        if let Some(font) = fonts.get(font_handle) {
            let layouts = font.get_layout(
                CoordinateSystem::PositiveYUp,
                text.horz_alignment,
                (text.position.x, text.position.y),
                (text.size.x, text.size.y),
                &text.content,
                text.line_height,
                text.font_size,
            );

            for layout in layouts {
                let position = Vec2::new(layout.position.0, layout.position.1);
                let size = Vec2::new(layout.size.0, layout.size.1);
                extracted_texts.push(ExtractCharBundle {
                    extracted_quad: ExtractedChar {
                        font_handle: Some(font_handle.clone()),
                        rect: Rect {
                            min: position,
                            max: position + size,
                        },
                        color: text.color,
                        vertex_index: 0,
                        char_id: font.get_char_id(layout.content).unwrap(),
                        z_index: 0.0,
                    },
                });
            }
        }
    }

    commands.spawn_batch(extracted_texts);
}
