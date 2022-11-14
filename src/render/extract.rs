use crate::{
    context::{KayakRootContext, WidgetName},
    node::Node,
    render_primitive::RenderPrimitive,
    styles::Corner,
};
use bevy::{
    prelude::{Assets, Camera, Color, Commands, Image, Plugin, Query, Rect, Res, Vec2, Entity},
    render::{Extract, RenderApp, RenderStage},
    window::Windows,
};
use kayak_font::KayakFont;

use super::{
    font::{self, FontMapping},
    image, nine_patch, texture_atlas,
    unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
};

// mod nine_patch;
// mod texture_atlas;

pub struct BevyKayakUIExtractPlugin;

impl Plugin for BevyKayakUIExtractPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract);
    }
}

pub fn extract(
    mut commands: Commands,
    context_query: Extract<Query<(Entity, &KayakRootContext, &Camera)>>,
    fonts: Extract<Res<Assets<KayakFont>>>,
    font_mapping: Extract<Res<FontMapping>>,
    node_query: Extract<Query<&Node>>,
    widget_names: Extract<Query<&WidgetName>>,
    images: Extract<Res<Assets<Image>>>,
    windows: Extract<Res<Windows>>,
) {
    let mut render_primitives = Vec::new();
    for (entity, context, camera) in context_query.iter() {
        let dpi = match &camera.target {
            bevy::render::camera::RenderTarget::Window(window_id) => {
                if let Some(window) = windows.get(*window_id) {
                    window.scale_factor() as f32
                } else {
                    1.0
                }
            }
            _ => 1.0,
        };
        let mut new_render_primitives = context.build_render_primitives(&node_query, &widget_names);
        render_primitives.extend(new_render_primitives.drain(..).map(|r| (entity, dpi, r)));
    }

    let mut extracted_quads = Vec::new();
    for (camera_entity, dpi, render_primitive) in render_primitives {
        match render_primitive {
            RenderPrimitive::Text { .. } => {
                let text_quads = font::extract_texts(camera_entity, &render_primitive, &fonts, &font_mapping, dpi);
                extracted_quads.extend(text_quads);
            }
            RenderPrimitive::Image { .. } => {
                let image_quads = image::extract_images(camera_entity, &render_primitive, dpi);
                extracted_quads.extend(image_quads);
            }
            RenderPrimitive::Quad { .. } => {
                let quad_quads = super::quad::extract_quads(camera_entity, &render_primitive, 1.0);
                extracted_quads.extend(quad_quads);
            }
            RenderPrimitive::NinePatch { .. } => {
                let nine_patch_quads =
                    nine_patch::extract_nine_patch(camera_entity, &render_primitive, &images, dpi);
                extracted_quads.extend(nine_patch_quads);
            }
            RenderPrimitive::TextureAtlas { .. } => {
                let texture_atlas_quads =
                    texture_atlas::extract_texture_atlas(camera_entity, &render_primitive, &images, dpi);
                extracted_quads.extend(texture_atlas_quads);
            }
            RenderPrimitive::Clip { layout } => {
                extracted_quads.push(ExtractQuadBundle {
                    extracted_quad: ExtractedQuad {
                        camera_entity,
                        rect: Rect {
                            min: Vec2::new(layout.posx, layout.posy) * dpi,
                            max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height)
                                * dpi,
                        },
                        color: Color::default(),
                        vertex_index: 0,
                        char_id: 0,
                        z_index: layout.z_index,
                        font_handle: None,
                        quad_type: UIQuadType::Clip,
                        type_index: 0,
                        border_radius: Corner::default(),
                        image: None,
                        uv_min: None,
                        uv_max: None,
                    },
                });
            }
            _ => {}
        }
    }

    // dbg!(&extracted_quads);
    commands.spawn_batch(extracted_quads);
}
