use crate::{BevyContext, FontMapping, ImageManager};
use bevy::{
    math::Vec2,
    prelude::{Assets, Commands, Plugin, Res},
    render::{color::Color, texture::Image, Extract, RenderApp, RenderStage},
    sprite::Rect,
    window::Windows,
};
use bevy_kayak_renderer::{
    render::unified::pipeline::{ExtractQuadBundle, ExtractedQuad, UIQuadType},
    Corner,
};
use kayak_core::render_primitive::RenderPrimitive;
use kayak_font::KayakFont;

pub mod font;
pub mod image;
mod nine_patch;
mod quad;

pub struct BevyKayakUIExtractPlugin;

impl Plugin for BevyKayakUIExtractPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(font::TextRendererPlugin)
            .add_plugin(image::ImageRendererPlugin);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract);
    }
}

pub fn extract(
    mut commands: Commands,
    context: Extract<Option<Res<BevyContext>>>,
    fonts: Extract<Res<Assets<KayakFont>>>,
    font_mapping: Extract<Res<FontMapping>>,
    image_manager: Extract<Res<ImageManager>>,
    images: Extract<Res<Assets<Image>>>,
    windows: Extract<Res<Windows>>,
) {
    if context.is_none() {
        return;
    }

    let context = context.as_ref().unwrap();

    let render_primitives = if let Ok(context) = context.kayak_context.read() {
        context.widget_manager.build_render_primitives()
    } else {
        vec![]
    };

    // dbg!(&render_primitives);

    let dpi = if let Some(window) = windows.get_primary() {
        window.scale_factor() as f32
    } else {
        1.0
    };

    let mut extracted_quads = Vec::new();
    for render_primitive in render_primitives {
        match render_primitive {
            RenderPrimitive::Text { .. } => {
                let text_quads = font::extract_texts(&render_primitive, &fonts, &font_mapping, dpi);
                extracted_quads.extend(text_quads);
            }
            RenderPrimitive::Image { .. } => {
                let image_quads = image::extract_images(&render_primitive, &image_manager, dpi);
                extracted_quads.extend(image_quads);
            }
            RenderPrimitive::Quad { .. } => {
                let quad_quads = quad::extract_quads(&render_primitive, 1.0);
                extracted_quads.extend(quad_quads);
            }
            RenderPrimitive::NinePatch { .. } => {
                let nine_patch_quads =
                    nine_patch::extract_nine_patch(&render_primitive, &image_manager, &images, dpi);
                extracted_quads.extend(nine_patch_quads);
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

    commands.spawn_batch(extracted_quads);
}
