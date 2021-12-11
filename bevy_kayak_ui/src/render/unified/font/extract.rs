use bevy::{
    math::Vec2,
    prelude::{Assets, Commands, Res},
    sprite2::Rect,
};
use kayak_core::render_primitive::RenderPrimitive;
use kayak_font::{Alignment, CoordinateSystem, KayakFont};

use crate::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    to_bevy_color, BevyContext,
};

use super::font_mapping::FontMapping;

pub fn extract_texts(
    mut commands: Commands,
    context: Res<BevyContext>,
    fonts: Res<Assets<KayakFont>>,
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
        let font = fonts.get(font_handle.clone());

        if font.is_none() {
            continue;
        }

        let font = font.unwrap();

        let chars_layouts = font.get_layout(
            CoordinateSystem::PositiveYDown,
            Alignment::Start,
            Vec2::new(layout.posx, layout.posy + font_size),
            Vec2::new(layout.width, layout.height),
            content,
            font_size * 1.2,
            font_size,
        );

        for char_layout in chars_layouts {
            extracted_texts.push(ExtractQuadBundle {
                extracted_quad: ExtractedQuad {
                    font_handle: Some(font_handle.clone()),
                    rect: Rect {
                        min: char_layout.position,
                        max: char_layout.position + char_layout.size,
                    },
                    color: to_bevy_color(background_color),
                    vertex_index: 0,
                    char_id: font.get_char_id(char_layout.content).unwrap(),
                    z_index: layout.z_index,
                    quad_type: UIQuadType::Text,
                    type_index: 0,
                    border_radius: (0.0, 0.0, 0.0, 0.0),
                    image: None,
                    uv_max: None,
                    uv_min: None,
                },
            });
        }
    }
    commands.spawn_batch(extracted_texts);
}
