use bevy::prelude::*;
use kayak_font::KayakFont;

use crate::{
    render::{
        font::FontMapping,
        unified::pipeline::{ExtractedQuad, ExtractedQuads, UIQuadType, QuadOrMaterial},
    },
    styles::{Corner, KStyle, RenderCommand, StyleProp},
};

pub trait RenderPrimitive {
    fn extract(
        &self,
        current_node: Entity,
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
        current_node: Entity,
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
    ) -> Option<ExtractedQuad> {
        let background_color = self.background_color.resolve();
        let render_command = self.render_command.resolve();
        let material = self.material.resolve_as_option();
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

                // println!("New Clip: {:?}", (rect, layout.z_index));

                let extracted: ExtractedQuad = ExtractedQuad {
                    org_entity: current_node,
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
                if let Some(_material) = material {
                    panic!("Materials not supported on clips right now.");
                } else {
                    extracted_quads.push(QuadOrMaterial::Quad(extracted.clone()));
                    return Some(extracted);
                }
            }
            RenderCommand::Quad => {
                let border_color = self.border_color.resolve();
                let border_radius = self.border_radius.resolve();
                let border = self.border.resolve();
                let box_shadow = self.box_shadow.resolve();
                let mut quads = crate::render::quad::extract_quads(
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

                for quad in quads.iter_mut() {
                    quad.org_entity = current_node;
                }

                if let Some(material) = material {
                    for extracted in quads {
                        let id = commands.spawn(extracted).id();
                        material.run(commands, id);
                        extracted_quads.push(QuadOrMaterial::Material(id));
                    }
                    return None;
                } else {
                    extracted_quads.extend(quads.into_iter().map(|q| QuadOrMaterial::Quad(q)).collect::<Vec<_>>());
                }
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
                let color = self.color.resolve_or(Color::WHITE);
                let text = crate::render::font::extract_texts(
                    camera_entity,
                    color,
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
                if let Some(material) = material {
                    for extracted in text {
                        let id = commands.spawn(extracted).id();
                        material.run(commands, id);
                        extracted_quads.push(QuadOrMaterial::Material(id));
                    }
                    return None;
                } else {
                    extracted_quads.extend(text.into_iter().map(|q| QuadOrMaterial::Quad(q)).collect::<Vec<_>>());
                }
            }
            RenderCommand::Image { handle } => {
                let border_radius = self.border_radius.resolve();
                let mut images = crate::render::image::extract_images(
                    camera_entity,
                    border_radius,
                    *layout,
                    handle,
                    opacity_layer,
                    dpi,
                );
                for image in images.iter_mut() {
                    image.org_entity = current_node;
                }
                if let Some(material) = material {
                    for extracted in images {
                        let id = commands.spawn(extracted).id();
                        material.run(commands, id);
                    }
                    return None;
                } else {
                    extracted_quads.extend(images.into_iter().map(|q| QuadOrMaterial::Quad(q)).collect::<Vec<_>>());
                }
            }
            RenderCommand::TextureAtlas {
                position,
                size,
                handle,
            } => {
                let mut atlases = crate::render::texture_atlas::extract_texture_atlas(
                    camera_entity,
                    size,
                    position,
                    *layout,
                    handle,
                    opacity_layer,
                    images,
                    dpi,
                );
                for atlas in atlases.iter_mut() {
                    atlas.org_entity = current_node;
                }
                if let Some(material) = material {
                    for extracted in atlases {
                        let id = commands.spawn(extracted).id();
                        material.run(commands, id);
                        extracted_quads.push(QuadOrMaterial::Material(id));
                    }
                    return None;
                } else {
                    extracted_quads.extend(atlases.into_iter().map(|q| QuadOrMaterial::Quad(q)).collect::<Vec<_>>());
                }
            }
            RenderCommand::NinePatch { border, handle } => {
                let mut nines = crate::render::nine_patch::extract_nine_patch(
                    camera_entity,
                    *layout,
                    handle,
                    border,
                    opacity_layer,
                    images,
                    dpi,
                );
                for nine in nines.iter_mut() {
                    nine.org_entity = current_node;
                }
                if let Some(material) = material {
                    for extracted in nines {
                        let id = commands.spawn(extracted).id();
                        material.run(commands, id);
                        extracted_quads.push(QuadOrMaterial::Material(id));
                    }
                    return None;
                } else {
                    extracted_quads.extend(nines.into_iter().map(|q| QuadOrMaterial::Quad(q)).collect::<Vec<_>>());
                }
            }
            RenderCommand::Svg { handle } => {
                let mut svgs = crate::render::svg::extract_svg(
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
                for svg in svgs.iter_mut() {
                    svg.org_entity = current_node;
                }
                if let Some(material) = material {
                    for extracted in svgs {
                        let id = commands.spawn(extracted).id();
                        material.run(commands, id);
                        extracted_quads.push(QuadOrMaterial::Material(id));
                    }
                    return None;
                } else {
                    extracted_quads.extend(svgs.into_iter().map(|q| QuadOrMaterial::Quad(q)).collect::<Vec<_>>());
                }
            }
            _ => {}
        }

        None
    }
}
