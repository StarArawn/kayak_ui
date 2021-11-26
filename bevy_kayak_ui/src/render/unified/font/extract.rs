use bevy::{
    math::Vec2,
    prelude::{Assets, Commands, Res, ResMut},
    sprite2::Rect,
};
use kayak_core::render_primitive::RenderPrimitive;

use crate::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    to_bevy_color, BevyContext,
};

use super::{font::KayakFont, font_mapping::FontMapping};

pub fn extract_texts(
    mut commands: Commands,
    context: Res<BevyContext>,
    mut fonts: ResMut<Assets<KayakFont>>,
    font_mapping: Res<FontMapping>,
) {
    let render_commands = if let Ok(context) = context.kayak_context.read() {
        context.widget_manager.build_render_primitives()
    } else {
        vec![]
    };

    let text_commands: Vec<&RenderPrimitive> = render_commands
        .iter()
        .filter(|command| matches!(command, RenderPrimitive::Text { .. }))
        .collect::<Vec<_>>();

    let mut extracted_texts = Vec::new();
    for render_primitive in text_commands {
        let (background_color, layout, font_size, content, font) = match render_primitive {
            RenderPrimitive::Text {
                color,
                layout,
                size,
                content,
                font,
            } => (color, layout, *size, content, *font),
            _ => panic!(""),
        };

        let font_handle = font_mapping.get_handle(font).unwrap();
        let new_chars = {
            let font = fonts.get(font_handle.clone()).unwrap();
            font.font.check_chars(content.chars())
        };
        // Filter out non-renderable spaces.
        let new_chars: Vec<_> = new_chars.into_iter().filter(|c| *c != ' ').collect();
        // Add chars to font.
        if new_chars.len() > 0 {
            let font = fonts.get_mut(font_handle.clone()).unwrap();
            for c in new_chars {
                font.font.add_character(c);
            }
        }
        let font = fonts.get(font_handle.clone()).unwrap();
        let char_layouts = font.font.get_layout(content, font_size);

        for (c, (x, y), (width, height)) in char_layouts {
            // let size = font.font.get_size(c, font_size);
            let position_x = layout.posx + x;
            let position_y = layout.posy + y;
            extracted_texts.push(ExtractQuadBundle {
                extracted_quad: ExtractedQuad {
                    font_handle: Some(font_handle.clone()),
                    rect: Rect {
                        min: Vec2::new(position_x, position_y),
                        max: Vec2::new(position_x + width, position_y + height),
                    },
                    color: to_bevy_color(background_color),
                    vertex_index: 0,
                    char_id: font.font.get_char_id(c),
                    z_index: layout.z_index,
                    quad_type: UIQuadType::Text,
                    type_index: 0,
                },
            });
        }
    }
    commands.spawn_batch(extracted_texts);
}
