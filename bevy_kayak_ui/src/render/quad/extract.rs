use crate::to_bevy_color;
use bevy::{math::Vec2, sprite::Rect};
use bevy_kayak_renderer::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    Corner,
};
use kayak_core::render_primitive::RenderPrimitive;

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
                color: to_bevy_color(&border_color),
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
                color: to_bevy_color(&background_color),
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
