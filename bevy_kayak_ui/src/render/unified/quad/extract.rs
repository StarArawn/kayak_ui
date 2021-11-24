use bevy::{
    math::Vec2,
    prelude::{Commands, Res},
    sprite2::Rect,
};
use kayak_core::render_primitive::RenderPrimitive;

use crate::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    to_bevy_color, BevyContext,
};

pub fn extract_quads(mut commands: Commands, context: Res<BevyContext>) {
    let render_commands = if let Ok(context) = context.kayak_context.read() {
        context.widget_manager.build_render_primitives()
    } else {
        vec![]
    };

    let quad_commands: Vec<&RenderPrimitive> = render_commands
        .iter()
        .filter(|command| matches!(command, RenderPrimitive::Quad { .. }))
        .collect::<Vec<_>>();

    let mut extracted_quads = Vec::new();
    for render_primitive in quad_commands {
        let (background_color, layout) = match render_primitive {
            RenderPrimitive::Quad {
                background_color,
                layout,
            } => (background_color, layout),
            _ => panic!(""),
        };

        extracted_quads.push(ExtractQuadBundle {
            extracted_quad: ExtractedQuad {
                rect: Rect {
                    min: Vec2::new(layout.posx, layout.posy),
                    max: Vec2::new(layout.width, layout.height),
                },
                color: to_bevy_color(background_color),
                vertex_index: 0,
                char_id: 0,
                z_index: layout.z_index,
                font_handle: None,
                quad_type: UIQuadType::Quad,
                type_index: 0,
            },
        });
    }
    commands.spawn_batch(extracted_quads);
}
