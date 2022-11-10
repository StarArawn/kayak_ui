use crate::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    render_primitive::RenderPrimitive,
    styles::Corner,
};
use bevy::{
    math::Vec2,
    prelude::{Assets, Rect, Res},
    render::{color::Color, texture::Image},
};

pub fn extract_texture_atlas(
    render_primitive: &RenderPrimitive,
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

    let image = images.get(handle);

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
        .unwrap()
        * dpi;

    let quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
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
            vertex_index: 0,
            char_id: 0,
            z_index: layout.z_index,
            font_handle: None,
            quad_type: UIQuadType::Image,
            type_index: 0,
            border_radius: Corner::default(),
            image: Some(handle.clone_weak()),
        },
    };
    extracted_quads.push(quad);

    extracted_quads
}
