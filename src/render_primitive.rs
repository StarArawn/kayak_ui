use bevy::prelude::*;
use kayak_font::KayakFont;

use crate::{
    render::{
        font::FontMapping,
        unified::pipeline::{ExtractedQuad, ExtractedQuads, UIQuadType},
    },
    styles::{Corner, KStyle, RenderCommand, StyleProp},
};

pub trait RenderPrimitive {
    fn extract(
        &self,
        commands: &mut Commands,
        layout: &crate::layout::Rect,
        opacity_layer: u32,
        extracted_quads: &mut ExtractedQuads,
        camera_entity: Entity,
        fonts: &Assets<KayakFont>,
        font_mapping: &FontMapping,
        images: &Assets<Image>,
        dpi: f32,
        prev_clip: Option<ExtractedQuad>,
    ) -> Option<ExtractedQuad>;
}

impl RenderPrimitive for KStyle {
    fn extract(
        &self,
        _commands: &mut Commands,
        layout: &crate::layout::Rect,
        opacity_layer: u32,
        extracted_quads: &mut ExtractedQuads,
        camera_entity: Entity,
        fonts: &Assets<KayakFont>,
        font_mapping: &FontMapping,
        images: &Assets<Image>,
        dpi: f32,
        prev_clip: Option<ExtractedQuad>,
    ) -> Option<ExtractedQuad> {
        let background_color = self.background_color.resolve();
        let render_command = self.render_command.resolve();
        match render_command {
            RenderCommand::Clip => {
                let mut rect = Rect {
                    min: Vec2::new(layout.posx, layout.posy) * dpi,
                    max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height) * dpi,
                };
                if let Some(prev_clip) = prev_clip {
                    let y1 = rect.max.y;
                    let y2 = prev_clip.rect.max.y;
                    rect.max.y = y1.min(y2);
                    if prev_clip.rect.min.y > rect.min.y {
                        rect.min.y = prev_clip.rect.min.y;
                    }
                }

                let clip = ExtractedQuad {
                    camera_entity,
                    rect,
                    color: Color::default(),
                    char_id: 0,
                    z_index: layout.z_index,
                    font_handle: None,
                    quad_type: UIQuadType::Clip,
                    type_index: 0,
                    border_radius: Corner::default(),
                    image: None,
                    uv_min: None,
                    uv_max: None,
                    opacity_layer,
                    ..Default::default()
                };
                extracted_quads.quads.push(clip.clone());
                return Some(clip);
            }
            RenderCommand::Quad => {
                let border_color = self.border_color.resolve();
                let border_radius = self.border_radius.resolve();
                let border = self.border.resolve();
                let box_shadow = self.box_shadow.resolve();
                let quads = crate::render::quad::extract_quads(
                    camera_entity,
                    background_color,
                    border_color,
                    *layout,
                    border_radius,
                    border,
                    opacity_layer,
                    box_shadow,
                    1.0,
                );
                extracted_quads.quads.extend(quads);
            }
            RenderCommand::Text {
                subpixel,
                text_layout,
                properties,
                ..
            } => {
                let font = self
                    .font
                    .resolve_or_else(|| String::from(crate::DEFAULT_FONT));
                let quads = crate::render::font::extract_texts(
                    camera_entity,
                    background_color,
                    text_layout,
                    *layout,
                    font,
                    properties,
                    subpixel,
                    opacity_layer,
                    fonts,
                    font_mapping,
                    dpi,
                );
                extracted_quads.quads.extend(quads);
            }
            RenderCommand::Image { handle } => {
                let border_radius = self.border_radius.resolve();
                let quads = crate::render::image::extract_images(
                    camera_entity,
                    border_radius,
                    *layout,
                    handle,
                    opacity_layer,
                    dpi,
                );
                extracted_quads.quads.extend(quads);
            }
            RenderCommand::TextureAtlas {
                position,
                size,
                handle,
            } => {
                let quads = crate::render::texture_atlas::extract_texture_atlas(
                    camera_entity,
                    size,
                    position,
                    *layout,
                    handle,
                    opacity_layer,
                    images,
                    dpi,
                );
                extracted_quads.quads.extend(quads);
            }
            RenderCommand::NinePatch { border, handle } => {
                let quads = crate::render::nine_patch::extract_nine_patch(
                    camera_entity,
                    *layout,
                    handle,
                    border,
                    opacity_layer,
                    images,
                    dpi,
                );
                extracted_quads.quads.extend(quads);
            }
            RenderCommand::Svg { handle } => {
                let quads = crate::render::svg::extract_svg(
                    camera_entity,
                    handle,
                    *layout,
                    match self.background_color {
                        StyleProp::Value(color) => Some(color),
                        _ => None,
                    },
                    opacity_layer,
                    dpi,
                );
                extracted_quads.quads.extend(quads);
            }
            _ => {}
        }

        None
    }
}
