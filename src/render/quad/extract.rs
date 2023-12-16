use crate::{
    render::unified::pipeline::{ExtractedQuad, UIQuadType},
    styles::{BoxShadow, Corner, Edge},
};
use bevy::{
    math::Vec2,
    prelude::{Color, Entity, Rect},
};

pub fn extract_quads(
    camera_entity: Entity,
    background_color: Color,
    border_color: Color,
    layout: crate::layout::Rect,
    border_radius: Corner<f32>,
    mut border: Edge<f32>,
    opacity_layer: u32,
    box_shadow: Vec<BoxShadow>,
    dpi: f32,
) -> Vec<ExtractedQuad> {
    border *= dpi;

    let mut extracted_quads = vec![];

    for box_shadow in box_shadow.iter().rev() {
        let half_spread = box_shadow.spread;
        let radius = box_shadow.radius * 3.0;
        let pos: Vec2 = Vec2::new(layout.posx, layout.posy) + box_shadow.offset;
        extracted_quads.push(ExtractedQuad {
            camera_entity,
            rect: Rect {
                min: pos - Vec2::splat(radius) - half_spread,
                max: pos
                    + Vec2::new(
                        (layout.width + radius) * dpi,
                        (layout.height + radius) * dpi,
                    )
                    + half_spread,
            },
            color: box_shadow.color,
            z_index: layout.z_index,
            quad_type: UIQuadType::BoxShadow,
            border_radius: Corner {
                top_left: border_radius.top_left,
                top_right: border_radius.top_right,
                bottom_left: border_radius.bottom_left,
                bottom_right: border_radius.bottom_right,
            },
            // Small hack to pass box shadow radius to shader.
            uv_min: Some(Vec2::splat(box_shadow.radius)),
            uv_max: Some(Vec2::splat(box_shadow.radius)),
            opacity_layer,
            ..Default::default()
        });
    }

    // Border
    if border.bottom > 0.0 || border.top > 0.0 || border.right > 0.0 || border.left > 0.0 {
        extracted_quads.push(ExtractedQuad {
            camera_entity,
            rect: Rect {
                min: Vec2::new(layout.posx, layout.posy) * dpi,
                max: Vec2::new(layout.posx + (layout.width), layout.posy + (layout.height)) * dpi,
            },
            color: border_color,
            z_index: layout.z_index,
            quad_type: UIQuadType::Quad,
            border_radius,
            opacity_layer,
            ..Default::default()
        });
    }

   extracted_quads.push(ExtractedQuad {
        camera_entity,
        rect: Rect {
            min: Vec2::new(layout.posx + border.left, layout.posy + border.top),
            max: Vec2::new(
                (layout.posx + (layout.width * dpi)) - border.right,
                (layout.posy + (layout.height * dpi)) - border.bottom,
            ),
        },
        color: background_color,
        z_index: layout.z_index,
        quad_type: UIQuadType::Quad,
        border_radius,
        opacity_layer,
        ..Default::default()
    });

    extracted_quads
}
