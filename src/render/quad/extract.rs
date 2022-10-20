use crate::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    render_primitive::RenderPrimitive,
    styles::Corner,
};
use bevy::{math::Vec2, prelude::Rect};

pub fn extract_quads(render_primitive: &RenderPrimitive, dpi: f32) -> Vec<ExtractQuadBundle> {
    let (background_color, border_color, layout, border_radius, mut border) = match render_primitive
    {
        RenderPrimitive::Quad {
            background_color,
            border_color,
            layout,
            border_radius,
            border,
        } => (
            *background_color,
            *border_color,
            *layout,
            *border_radius,
            *border,
        ),
        _ => panic!(""),
    };

    border *= dpi;

    vec![
        // Border
        ExtractQuadBundle {
            extracted_quad: ExtractedQuad {
                rect: Rect {
                    min: Vec2::new(layout.posx, layout.posy),
                    max: Vec2::new(
                        layout.posx + (layout.width * dpi),
                        layout.posy + (layout.height * dpi),
                    ),
                },
                color: border_color,
                vertex_index: 0,
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
            },
        },
        ExtractQuadBundle {
            extracted_quad: ExtractedQuad {
                rect: Rect {
                    min: Vec2::new(layout.posx + border.left, layout.posy + border.top),
                    max: Vec2::new(
                        (layout.posx + (layout.width * dpi)) - border.right,
                        (layout.posy + (layout.height * dpi)) - border.bottom,
                    ),
                },
                color: background_color,
                vertex_index: 0,
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
            },
        },
    ]
}
