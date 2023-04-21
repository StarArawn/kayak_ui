use bevy::prelude::*;

use crate::{render::unified::pipeline::ExtractedQuad, render_primitive::RenderPrimitive};

pub fn extract_svg(
    camera_entity: Entity,
    render_primitive: &RenderPrimitive,
    _dpi: f32,
) -> ExtractedQuad {
    let (handle, layout, background_color) = match render_primitive {
        RenderPrimitive::Svg {
            handle,
            layout,
            background_color,
        } => (handle.clone_weak(), *layout, *background_color),
        _ => panic!(""),
    };

    ExtractedQuad {
        camera_entity,
        rect: Rect {
            min: Vec2::new(layout.posx, layout.posy),
            max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height),
        },
        z_index: layout.z_index,
        svg_handle: (Some(handle), background_color),
        ..Default::default()
    }
}
