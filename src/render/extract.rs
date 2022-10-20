use crate::{context::Context, node::Node, render_primitive::RenderPrimitive, styles::Corner};
use bevy::{
    // math::Vec2,
    prelude::{Assets, Color, Commands, Image, Plugin, Query, Rect, Res, Vec2},
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
    context: Extract<Res<Context>>,
    fonts: Extract<Res<Assets<KayakFont>>>,
    font_mapping: Extract<Res<FontMapping>>,
    node_query: Extract<Query<&Node>>,
    images: Extract<Res<Assets<Image>>>,
    windows: Extract<Res<Windows>>,
) {
    // dbg!("STARTED");
    let render_primitives = context.build_render_primitives(&node_query);
    // dbg!("FINISHED");

    let dpi = if let Some(window) = windows.get_primary() {
        window.scale_factor() as f32
    } else {
        1.0
    };

    // dbg!(&render_primitives);

    let mut extracted_quads = Vec::new();
    for render_primitive in render_primitives {
        match render_primitive {
            RenderPrimitive::Text { .. } => {
                let text_quads = font::extract_texts(&render_primitive, &fonts, &font_mapping, dpi);
                extracted_quads.extend(text_quads);
            }
            RenderPrimitive::Image { .. } => {
                let image_quads = image::extract_images(&render_primitive, dpi);
                extracted_quads.extend(image_quads);
            }
            RenderPrimitive::Quad { .. } => {
                let quad_quads = super::quad::extract_quads(&render_primitive, 1.0);
                extracted_quads.extend(quad_quads);
            }
            RenderPrimitive::NinePatch { .. } => {
                let nine_patch_quads =
                    nine_patch::extract_nine_patch(&render_primitive, &images, dpi);
                extracted_quads.extend(nine_patch_quads);
            }
            RenderPrimitive::TextureAtlas { .. } => {
                let texture_atlas_quads =
                    texture_atlas::extract_texture_atlas(&render_primitive, &images, dpi);
                extracted_quads.extend(texture_atlas_quads);
            }
            RenderPrimitive::Clip { layout } => {
                extracted_quads.push(ExtractQuadBundle {
                    extracted_quad: ExtractedQuad {
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
