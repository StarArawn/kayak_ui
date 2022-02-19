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

pub fn extract_nine_patch(
    render_primitive: &RenderPrimitive,
    image_manager: &Res<ImageManager>,
    images: &Res<Assets<Image>>,
    dpi: f32,
) -> Vec<ExtractQuadBundle> {
    let mut extracted_quads = Vec::new();

    let (layout, handle, border) = match render_primitive {
        RenderPrimitive::NinePatch {
            layout,
            handle,
            border,
        } => (layout, handle, border),
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

    let extracted_quad_template = ExtractedQuad {
        rect: Rect {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        },
        color: Color::WHITE,
        vertex_index: 0,
        char_id: 0,
        z_index: layout.z_index,
        font_handle: None,
        quad_type: UIQuadType::Image,
        type_index: 0,
        border_radius: Corner::default(),
        image: image_handle,
        uv_max: None,
        uv_min: None,
    };

    // TOP
    let top_left_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(layout.posx, layout.posy),
                max: Vec2::new(layout.posx + border.left, layout.posy + border.top),
            },
            uv_min: Some(Vec2::new(0.0, border.top / image_size.y)),
            uv_max: Some(Vec2::new(border.left / image_size.x, 0.0)),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(top_left_quad);

    let top_right_pos_x = (layout.posx + layout.width) - border.left;
    let top_right_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(top_right_pos_x, layout.posy),
                max: Vec2::new(top_right_pos_x + border.left, layout.posy + border.top),
            },
            uv_min: Some(Vec2::new(
                (image_size.x - border.left) / image_size.x,
                border.top / image_size.y,
            )),
            uv_max: Some(Vec2::new(1.0, 0.0)),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(top_right_quad);

    let top_middle_pos_x = layout.posx + border.left;
    let top_middle_size_x = layout.width - (border.left + border.right);
    let top_middle_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(top_middle_pos_x, layout.posy),
                max: Vec2::new(
                    top_middle_pos_x + top_middle_size_x,
                    layout.posy + border.top,
                ),
            },
            uv_min: Some(Vec2::new(
                border.left / image_size.x,
                border.top / image_size.y,
            )),
            uv_max: Some(Vec2::new((image_size.x - border.left) / image_size.x, 0.0)),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(top_middle_quad);

    // Bottom
    let bottom_y_pos = layout.posy + (layout.height - border.bottom);
    let bottom_left_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(layout.posx, bottom_y_pos),
                max: Vec2::new(layout.posx + border.left, bottom_y_pos + border.bottom),
            },
            uv_min: Some(Vec2::new(0.0, 1.0)),
            uv_max: Some(Vec2::new(
                border.left / image_size.x,
                (image_size.y - border.bottom) / image_size.y,
            )),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(bottom_left_quad);

    let bottom_right_pos_x = (layout.posx + layout.width) - border.left;
    let bottom_right_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(bottom_right_pos_x, bottom_y_pos),
                max: Vec2::new(bottom_right_pos_x + border.left, bottom_y_pos + border.top),
            },
            uv_min: Some(Vec2::new((image_size.x - border.left) / image_size.x, 1.0)),
            uv_max: Some(Vec2::new(
                1.0,
                (image_size.y - border.bottom) / image_size.y,
            )),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(bottom_right_quad);

    let bottom_middle_pos_x = layout.posx + border.left;
    let bottom_middle_size_x = layout.width - (border.left + border.right);
    let bottom_middle_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(bottom_middle_pos_x, bottom_y_pos),
                max: Vec2::new(
                    bottom_middle_pos_x + bottom_middle_size_x,
                    bottom_y_pos + border.top,
                ),
            },
            uv_min: Some(Vec2::new(border.left / image_size.x, 1.0)),
            uv_max: Some(Vec2::new(
                (image_size.x - border.left) / image_size.x,
                (image_size.y - border.bottom) / image_size.y,
            )),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(bottom_middle_quad);

    // Left + Right center
    let left_middle_pos_y = layout.posy + border.top;
    let left_middle_size_y = layout.height - (border.top + border.bottom);
    let left_middle_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(layout.posx, left_middle_pos_y),
                max: Vec2::new(
                    layout.posx + border.left,
                    left_middle_pos_y + left_middle_size_y,
                ),
            },
            uv_min: Some(Vec2::new(
                0.0,
                (image_size.y - border.bottom) / image_size.y,
            )),
            uv_max: Some(Vec2::new(
                border.left / image_size.x,
                border.top / image_size.y,
            )),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(left_middle_quad);

    let right_middle_pos_x = layout.posx + (layout.width - border.right);
    let right_middle_pos_y = layout.posy + border.top;
    let right_middle_size_y = layout.height - (border.top + border.bottom);
    let right_middle_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(right_middle_pos_x, right_middle_pos_y),
                max: Vec2::new(
                    right_middle_pos_x + border.left,
                    right_middle_pos_y + right_middle_size_y,
                ),
            },
            uv_min: Some(Vec2::new(
                (image_size.x - border.left) / image_size.x,
                (image_size.y - border.bottom) / image_size.y,
            )),
            uv_max: Some(Vec2::new(1.0, border.top / image_size.y)),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(right_middle_quad);

    // Last quad in middle.
    let middle_pos_x = layout.posx + border.left;
    let middle_pos_y = layout.posy + border.top;
    let middle_size_x = layout.width - (border.left + border.right);
    let middle_size_y = layout.height - (border.top + border.bottom);
    let middle_quad = ExtractQuadBundle {
        extracted_quad: ExtractedQuad {
            rect: Rect {
                min: Vec2::new(middle_pos_x, middle_pos_y),
                max: Vec2::new(middle_pos_x + middle_size_x, middle_pos_y + middle_size_y),
            },
            uv_min: Some(Vec2::new(
                border.left / image_size.x,
                border.top / image_size.y,
            )),
            uv_max: Some(Vec2::new(
                (image_size.x - border.right) / image_size.x,
                (image_size.y - border.bottom) / image_size.y,
            )),
            ..extracted_quad_template.clone()
        },
    };
    extracted_quads.push(middle_quad);

    extracted_quads
}
