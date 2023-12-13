use crate::{
    render::unified::pipeline::{ExtractedQuad, UIQuadType},
    styles::Corner,
};
use bevy::{math::Vec2, prelude::*, render::color::Color};

pub fn extract_images(
    camera_entity: Entity,
    border_radius: Corner<f32>,
    layout: crate::layout::Rect,
    handle: Handle<Image>,
    opacity_layer: u32,
    _dpi: f32,
) -> Vec<ExtractedQuad> {
    vec![ExtractedQuad {
        camera_entity,
        rect: Rect {
            min: Vec2::new(layout.posx, layout.posy),
            max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height),
        },
        color: Color::WHITE,
        char_id: 0,
        z_index: layout.z_index,
        font_handle: None,
        quad_type: UIQuadType::Image,
        type_index: 0,
        border_radius,
        image: Some(handle.clone_weak()),
        uv_max: None,
        uv_min: None,
        opacity_layer,
        ..Default::default()
    }]
}
