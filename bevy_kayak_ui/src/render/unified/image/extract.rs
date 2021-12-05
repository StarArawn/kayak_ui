use bevy::{
    math::Vec2,
    prelude::{Commands, Res},
    render2::color::Color,
    sprite2::Rect,
};
use kayak_core::render_primitive::RenderPrimitive;

use crate::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    BevyContext, ImageManager,
};

pub fn extract_images(
    mut commands: Commands,
    context: Res<BevyContext>,
    image_manager: Res<ImageManager>,
) {
    let render_commands = if let Ok(context) = context.kayak_context.read() {
        context.widget_manager.build_render_primitives()
    } else {
        vec![]
    };

    let image_commands: Vec<&RenderPrimitive> = render_commands
        .iter()
        .filter(|command| matches!(command, RenderPrimitive::Image { .. }))
        .collect::<Vec<_>>();

    let mut extracted_quads = Vec::new();
    for render_primitive in image_commands {
        let (layout, handle) = match render_primitive {
            RenderPrimitive::Image { layout, handle } => (layout, handle),
            _ => panic!(""),
        };

        extracted_quads.push(ExtractQuadBundle {
            extracted_quad: ExtractedQuad {
                rect: Rect {
                    min: Vec2::new(layout.posx, layout.posy),
                    max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height),
                },
                color: Color::WHITE,
                vertex_index: 0,
                char_id: 0,
                z_index: layout.z_index,
                font_handle: None,
                quad_type: UIQuadType::Image,
                type_index: 0,
                border_radius: (0.0, 0.0, 0.0, 0.0),
                image: image_manager
                    .get_handle(handle)
                    .and_then(|a| Some(a.clone_weak())),
            },
        });
    }
    commands.spawn_batch(extracted_quads);
}
