use bevy::{
    core_pipeline::Transparent2d,
    prelude::{Assets, HandleUntyped, Plugin, Res, ResMut},
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_phase::DrawFunctions,
        render_resource::Shader,
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
        RenderApp, RenderStage,
    },
};
use kayak_font::FontTextureCache;

use self::pipeline::{DrawUI, FontPipeline, QuadMeta};

mod extract;
pub mod pipeline;
mod text;

pub use text::*;

pub const FONT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7604018236855288450);

pub struct FontRenderPlugin;

impl Plugin for FontRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let unified_shader = Shader::from_wgsl(include_str!("shader.wgsl"));
        shaders.set_untracked(FONT_SHADER_HANDLE, unified_shader);

        let render_app = app.sub_app(RenderApp);
        render_app
            .init_resource::<QuadMeta>()
            .init_resource::<FontPipeline>()
            .add_system_to_stage(RenderStage::Extract, extract::extract)
            .add_system_to_stage(RenderStage::Prepare, pipeline::prepare_quads)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_quads)
            .add_system_to_stage(RenderStage::Queue, create_and_update_font_cache_texture);

        let draw_quad = DrawUI::new(&mut render_app.world);

        render_app
            .world
            .get_resource::<DrawFunctions<Transparent2d>>()
            .unwrap()
            .write()
            .add(draw_quad);
    }
}

fn create_and_update_font_cache_texture(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<FontPipeline>,
    mut font_texture_cache: ResMut<FontTextureCache>,
    images: Res<RenderAssets<Image>>,
) {
    font_texture_cache.process_new(&device, &queue, pipeline.into_inner(), &images);
}
