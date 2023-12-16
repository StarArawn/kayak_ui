use crate::{
    render::unified::pipeline::{ExtractedQuad, UIQuadType},
    styles::Corner,
};
use bevy::{
    math::Vec2,
    prelude::*,
    render::{color::Color, texture::Image},
};

pub fn extract_texture_atlas(
    camera_entity: Entity,
    size: Vec2,
    position: Vec2,
    layout: crate::layout::Rect,
    handle: Handle<Image>,
    opacity_layer: u32,
    images: &Assets<Image>,
    _dpi: f32,
) -> Vec<ExtractedQuad> {
    let mut extracted_quads = Vec::new();

    let image = images.get(&handle);

    if image.is_none() {
        return vec![];
    }

    let image_size = image
        .map(|i| {
            Vec2::new(
                i.texture_descriptor.size.width as f32,
                i.texture_descriptor.size.height as f32,
            )
        })
        .unwrap();

    let quad = ExtractedQuad {
        camera_entity,
        rect: Rect {
            min: Vec2::new(layout.posx, layout.posy),
            max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height),
        },
        uv_min: Some(Vec2::new(
            position.x / image_size.x,
            1.0 - ((position.y + size.y) / image_size.y),
        )),
        uv_max: Some(Vec2::new(
            (position.x + size.x) / image_size.x,
            1.0 - (position.y / image_size.y),
        )),
        color: Color::WHITE,
        char_id: 0,
        font_handle: None,
        quad_type: UIQuadType::Image,
        type_index: 0,
        border_radius: Corner::default(),
        image: Some(handle.clone_weak()),
        opacity_layer,
        ..Default::default()
    };
    extracted_quads.push(quad);

    extracted_quads
}
