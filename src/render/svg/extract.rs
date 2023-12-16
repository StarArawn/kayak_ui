use bevy::prelude::*;
use bevy_svg::prelude::Svg;

use crate::render::unified::pipeline::ExtractedQuad;

pub fn extract_svg(
    camera_entity: Entity,
    handle: Handle<Svg>,
    layout: crate::layout::Rect,
    background_color: Option<Color>,
    opacity_layer: u32,
    _dpi: f32,
) -> Vec<ExtractedQuad> {
    vec![ExtractedQuad {
        camera_entity,
        rect: Rect {
            min: Vec2::new(layout.posx, layout.posy),
            max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height),
        },
        svg_handle: (Some(handle), background_color),
        opacity_layer,
        ..Default::default()
    }]

}
