use crate::{
    render::unified::pipeline::{ExtractedQuad, UIQuadType},
    styles::{Corner, Edge},
};
use bevy::{
    math::Vec2,
    prelude::*,
    render::{color::Color, texture::Image},
};

pub fn extract_nine_patch(
    camera_entity: Entity,
    layout: crate::layout::Rect,
    handle: Handle<Image>,
    border: Edge<f32>,
    opacity_layer: u32,
    images: &Assets<Image>,
    dpi: f32,
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
        .unwrap()
        * dpi;

    let extracted_quad_template = ExtractedQuad {
        camera_entity,
        rect: Rect {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        },
        color: Color::WHITE,
        char_id: 0,
        font_handle: None,
        quad_type: UIQuadType::Image,
        type_index: 0,
        border_radius: Corner::default(),
        image: Some(handle.clone_weak()),
        uv_max: None,
        uv_min: None,
        opacity_layer,
        ..Default::default()
    };

    let top_uv_min_y = (image_size.y - border.top) / image_size.y;
    let top_uv_max_y = 1.0;
    let bottom_uv_min_y = 0.0;
    let bottom_uv_max_y = border.bottom / image_size.y;
    let middle_uv_min_y = border.bottom / image_size.y;
    let middle_uv_max_y = (image_size.y - border.top) / image_size.y;

    // TOP
    let top_left_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(layout.posx, layout.posy),
            max: Vec2::new(layout.posx + border.left, layout.posy + border.top),
        },
        uv_min: Some(Vec2::new(0.0, top_uv_min_y)),
        uv_max: Some(Vec2::new(border.left / image_size.x, top_uv_max_y)),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(top_left_quad);

    let top_right_pos_x = (layout.posx + layout.width) - border.left;
    let top_right_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(top_right_pos_x, layout.posy),
            max: Vec2::new(top_right_pos_x + border.left, layout.posy + border.top),
        },
        uv_min: Some(Vec2::new(
            (image_size.x - border.left) / image_size.x,
            top_uv_min_y,
        )),
        uv_max: Some(Vec2::new(1.0, top_uv_max_y)),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(top_right_quad);

    let top_middle_pos_x = layout.posx + border.left;
    let top_middle_size_x = layout.width - (border.left + border.right);
    let top_middle_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(top_middle_pos_x, layout.posy),
            max: Vec2::new(
                top_middle_pos_x + top_middle_size_x,
                layout.posy + border.top,
            ),
        },
        uv_min: Some(Vec2::new(border.left / image_size.x, top_uv_min_y)),
        uv_max: Some(Vec2::new(
            (image_size.x - border.left) / image_size.x,
            top_uv_max_y,
        )),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(top_middle_quad);

    // Bottom
    let bottom_y_pos = layout.posy + (layout.height - border.bottom);
    let bottom_left_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(layout.posx, bottom_y_pos),
            max: Vec2::new(layout.posx + border.left, bottom_y_pos + border.bottom),
        },
        uv_min: Some(Vec2::new(0.0, bottom_uv_min_y)),
        uv_max: Some(Vec2::new(border.left / image_size.x, bottom_uv_max_y)),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(bottom_left_quad);

    let bottom_right_pos_x = (layout.posx + layout.width) - border.left;
    let bottom_right_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(bottom_right_pos_x, bottom_y_pos),
            max: Vec2::new(bottom_right_pos_x + border.left, bottom_y_pos + border.top),
        },
        uv_min: Some(Vec2::new(
            (image_size.x - border.left) / image_size.x,
            bottom_uv_min_y,
        )),
        uv_max: Some(Vec2::new(1.0, bottom_uv_max_y)),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(bottom_right_quad);

    let bottom_middle_pos_x = layout.posx + border.left;
    let bottom_middle_size_x = layout.width - (border.left + border.right);
    let bottom_middle_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(bottom_middle_pos_x, bottom_y_pos),
            max: Vec2::new(
                bottom_middle_pos_x + bottom_middle_size_x,
                bottom_y_pos + border.top,
            ),
        },
        uv_min: Some(Vec2::new(border.left / image_size.x, bottom_uv_min_y)),
        uv_max: Some(Vec2::new(
            (image_size.x - border.left) / image_size.x,
            bottom_uv_max_y,
        )),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(bottom_middle_quad);

    // Left + Right center
    let left_middle_pos_y = layout.posy + border.top;
    let left_middle_size_y = layout.height - (border.top + border.bottom);
    let left_middle_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(layout.posx, left_middle_pos_y),
            max: Vec2::new(
                layout.posx + border.left,
                left_middle_pos_y + left_middle_size_y,
            ),
        },
        uv_min: Some(Vec2::new(0.0, middle_uv_min_y)),
        uv_max: Some(Vec2::new(border.left / image_size.x, middle_uv_max_y)),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(left_middle_quad);

    let right_middle_pos_x = layout.posx + (layout.width - border.right);
    let right_middle_pos_y = layout.posy + border.top;
    let right_middle_size_y = layout.height - (border.top + border.bottom);
    let right_middle_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(right_middle_pos_x, right_middle_pos_y),
            max: Vec2::new(
                right_middle_pos_x + border.left,
                right_middle_pos_y + right_middle_size_y,
            ),
        },
        uv_min: Some(Vec2::new(
            (image_size.x - border.left) / image_size.x,
            middle_uv_min_y,
        )),
        uv_max: Some(Vec2::new(1.0, middle_uv_max_y)),
        ..extracted_quad_template.clone()
    };
    extracted_quads.push(right_middle_quad);

    // Last quad in middle.
    let middle_pos_x = layout.posx + border.left;
    let middle_pos_y = layout.posy + border.top;
    let middle_size_x = layout.width - (border.left + border.right);
    let middle_size_y = layout.height - (border.top + border.bottom);
    let middle_quad = ExtractedQuad {
        rect: Rect {
            min: Vec2::new(middle_pos_x, middle_pos_y),
            max: Vec2::new(middle_pos_x + middle_size_x, middle_pos_y + middle_size_y),
        },
        uv_min: Some(Vec2::new(border.left / image_size.x, middle_uv_min_y)),
        uv_max: Some(Vec2::new(
            (image_size.x - border.right) / image_size.x,
            middle_uv_max_y,
        )),
        ..extracted_quad_template
    };
    extracted_quads.push(middle_quad);

    extracted_quads
}
