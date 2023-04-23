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
    let (
        background_color,
        border_color,
        layout,
        border_radius,
        mut border,
        opacity_layer,
        box_shadow,
    ) = match render_primitive {
        RenderPrimitive::Quad {
            background_color,
            border_color,
            layout,
            border_radius,
            border,
            opacity_layer,
            box_shadow,
        } => (
            *background_color,
            *border_color,
            *layout,
            *border_radius,
            *border,
            *opacity_layer,
            box_shadow.clone(),
        ),
        _ => panic!(""),
    };

    border *= dpi;

    let mut extracted_quads = vec![
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
            z_index: layout.z_index - 0.0005,
            quad_type: UIQuadType::Quad,
            border_radius: Corner {
                top_left: border_radius.top_left,
                top_right: border_radius.top_right,
                bottom_left: border_radius.bottom_left,
                bottom_right: border_radius.bottom_right,
            },
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
            z_index: layout.z_index,
            quad_type: UIQuadType::Quad,
            border_radius: Corner {
                top_left: border_radius.top_left,
                top_right: border_radius.top_right,
                bottom_left: border_radius.bottom_left,
                bottom_right: border_radius.bottom_right,
            },
            opacity_layer,
            ..Default::default()
        },
    ];

    if let Some(box_shadow) = box_shadow {
        let count = box_shadow.len();
        for (i, box_shadow) in box_shadow.iter().enumerate() {
            let z = ((i + 1) as f32 / (count + 1) as f32) * 0.0001;
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
                z_index: layout.z_index - z,
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
    }

    extracted_quads
}
