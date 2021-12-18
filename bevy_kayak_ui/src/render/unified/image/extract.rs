use bevy::{math::Vec2, prelude::Res, render::color::Color, sprite::Rect};
use kayak_core::render_primitive::RenderPrimitive;

use crate::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    ImageManager,
};

pub fn extract_images(
    render_command: &RenderPrimitive,
    image_manager: &Res<ImageManager>,
    dpi: f32,
) -> Vec<ExtractQuadBundle> {
    let (layout, handle) = match render_command {
        RenderPrimitive::Image { layout, handle } => (layout, handle),
        _ => panic!(""),
    };

    vec![ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(layout.posx, layout.posy),
                max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height) * dpi,
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
            uv_max: None,
            uv_min: None,
        },
    }]
}
