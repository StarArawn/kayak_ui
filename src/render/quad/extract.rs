use crate::{
    render::unified::pipeline::{ExtractedQuad, UIQuadType},
    render_primitive::RenderPrimitive,
    styles::Corner,
};
use bevy::{
    math::Vec2,
    prelude::{Entity, Rect},
};

pub fn extract_quads(
    camera_entity: Entity,
    render_primitive: &RenderPrimitive,
    dpi: f32,
) -> Vec<ExtractedQuad> {
    let (background_color, border_color, layout, border_radius, mut border, opacity_layer) =
        match render_primitive {
            RenderPrimitive::Quad {
                background_color,
                border_color,
                layout,
                border_radius,
                border,
                opacity_layer,
            } => (
                *background_color,
                *border_color,
                *layout,
                *border_radius,
                *border,
                *opacity_layer,
            ),
            _ => panic!(""),
        };

    border *= dpi;

    vec![
        // Border
        ExtractedQuad {
            camera_entity,
            rect: Rect {
                min: Vec2::new(layout.posx, layout.posy),
                max: Vec2::new(
                    layout.posx + (layout.width * dpi),
                    layout.posy + (layout.height * dpi),
                ),
            },
            color: border_color,
            char_id: 0,
            z_index: layout.z_index - 0.0005,
            font_handle: None,
            quad_type: UIQuadType::Quad,
            type_index: 0,
            border_radius: Corner {
                top_left: border_radius.top_left,
                top_right: border_radius.top_right,
                bottom_left: border_radius.bottom_left,
                bottom_right: border_radius.bottom_right,
            },
            image: None,
            uv_max: None,
            uv_min: None,
            opacity_layer,
            ..Default::default()
        },
        ExtractedQuad {
            camera_entity,
            rect: Rect {
                min: Vec2::new(layout.posx + border.left, layout.posy + border.top),
                max: Vec2::new(
                    (layout.posx + (layout.width * dpi)) - border.right,
                    (layout.posy + (layout.height * dpi)) - border.bottom,
                ),
            },
            color: background_color,
            char_id: 0,
            z_index: layout.z_index,
            font_handle: None,
            quad_type: UIQuadType::Quad,
            type_index: 0,
            border_radius: Corner {
                top_left: border_radius.top_left,
                top_right: border_radius.top_right,
                bottom_left: border_radius.bottom_left,
                bottom_right: border_radius.bottom_right,
            },
            image: None,
            uv_max: None,
            uv_min: None,
            opacity_layer,
            ..Default::default()
        },
    ]
}
