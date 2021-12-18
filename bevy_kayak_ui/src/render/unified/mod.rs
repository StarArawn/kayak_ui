use bevy::{
    math::Vec2,
    prelude::{Assets, Commands, HandleUntyped, Plugin, Res},
    reflect::TypeUuid,
    render::{
        color::Color, render_phase::DrawFunctions, render_resource::Shader, texture::Image,
        RenderApp, RenderStage,
    },
    sprite::Rect,
    window::Windows,
};
use kayak_core::{render_primitive::RenderPrimitive, Binding, Bound};
use kayak_font::KayakFont;

use crate::{
    render::{
        ui_pass::TransparentUI,
        unified::pipeline::{DrawUI, QuadMeta, UnifiedPipeline},
    },
    BevyContext, FontMapping, ImageManager, WindowSize,
};

use self::pipeline::{ExtractQuadBundle, ExtractedQuad, ImageBindGroups, UIQuadType};

pub mod font;
pub mod image;
mod nine_patch;
mod pipeline;
mod quad;

pub const UNIFIED_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7604018236855288450);

pub struct UnifiedRenderPlugin;

impl Plugin for UnifiedRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let unified_shader = Shader::from_wgsl(include_str!("shader.wgsl"));
        shaders.set_untracked(UNIFIED_SHADER_HANDLE, unified_shader);

        app.add_plugin(font::TextRendererPlugin)
            .add_plugin(image::ImageRendererPlugin);

        let render_app = app.sub_app(RenderApp);
        render_app
            .init_resource::<ImageBindGroups>()
            .init_resource::<UnifiedPipeline>()
            .init_resource::<QuadMeta>()
            .add_system_to_stage(RenderStage::Extract, extract)
            .add_system_to_stage(RenderStage::Prepare, pipeline::prepare_quads)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_quads);

        let draw_quad = DrawUI::new(&mut render_app.world);

        render_app
            .world
            .get_resource::<DrawFunctions<TransparentUI>>()
            .unwrap()
            .write()
            .add(draw_quad);
    }
}

pub fn extract(
    mut commands: Commands,
    context: Res<BevyContext>,
    fonts: Res<Assets<KayakFont>>,
    font_mapping: Res<FontMapping>,
    image_manager: Res<ImageManager>,
    images: Res<Assets<Image>>,
    windows: Res<Windows>,
    window_size: Res<Binding<WindowSize>>,
) {
    let render_primitives = if let Ok(context) = context.kayak_context.read() {
        context.widget_manager.build_render_primitives()
    } else {
        vec![]
    };

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
                let image_quads = image::extract_images(&render_primitive, &image_manager, dpi);
                extracted_quads.extend(image_quads);
            }
            RenderPrimitive::Quad { .. } => {
                let quad_quads = quad::extract_quads(&render_primitive, dpi);
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
                            min: Vec2::new(layout.posx, layout.posy),
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
                        border_radius: (0.0, 0.0, 0.0, 0.0),
                        image: None,
                        uv_min: None,
                        uv_max: None,
                    },
                });
            }
            _ => {}
        }
    }

    commands.insert_resource(window_size.get());
    commands.spawn_batch(extracted_quads);
}
