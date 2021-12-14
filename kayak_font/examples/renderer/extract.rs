use bevy::{
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
                text.position,
                text.size,
                &text.content,
                text.line_height,
                text.font_size,
            );

            for layout in layouts {
                extracted_texts.push(ExtractCharBundle {
                    extracted_quad: ExtractedChar {
                        font_handle: Some(font_handle.clone()),
                        rect: Rect {
                            min: layout.position,
                            max: layout.position + layout.size,
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
