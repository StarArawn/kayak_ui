use crate::ImageManager;
use bevy::{
    math::Vec2,
    prelude::{Assets, Res},
    render::{color::Color, texture::Image},
    sprite::Rect,
};
use bevy_kayak_renderer::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    Corner,
};
use kayak_core::render_primitive::RenderPrimitive;

pub fn extract_texture_atlas(
    render_primitive: &RenderPrimitive,
    image_manager: &Res<ImageManager>,
    images: &Res<Assets<Image>>,
    dpi: f32,
) -> Vec<ExtractQuadBundle> {
    let mut extracted_quads = Vec::new();

    let (size, position, layout, handle) = match render_primitive {
        RenderPrimitive::TextureAtlas {
        size,
        position,
            layout,
            handle,
        } => (size, position, layout, handle),
        _ => panic!(""),
    };

    let image_handle = image_manager
        .get_handle(handle)
        .and_then(|a| Some(a.clone_weak()));

    let image = images.get(image_handle.as_ref().unwrap());

    if image.is_none() {
        return vec![];
    }

    let image_size = image
        .and_then(|i| {
            Some(Vec2::new(
                i.texture_descriptor.size.width as f32,
                i.texture_descriptor.size.height as f32,
            ))
        })
        .unwrap()
        * dpi;

    let quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(layout.posx, layout.posy),
                max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height),
            },
            uv_min: Some(Vec2::new(
                position.0 / image_size.x,
                1.0 - ((position.1 + size.1) / image_size.y)
            )),
            uv_max: Some(Vec2::new(
                (position.0 + size.0) / image_size.x,
                1.0 - (position.1 / image_size.y),
            )),
        color: Color::WHITE,
        vertex_index: 0,
        char_id: 0,
        z_index: layout.z_index,
        font_handle: None,
        quad_type: UIQuadType::Image,
        type_index: 0,
        border_radius: Corner::default(),
        image: image_handle,
        },
    };
    extracted_quads.push(quad);

    extracted_quads
}
