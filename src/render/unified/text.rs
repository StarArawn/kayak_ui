use bevy::{
    prelude::{Plugin, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
        RenderApp, RenderStage,
    },
};
use kayak_font::bevy::{FontTextureCache, KayakFontPlugin};

use super::pipeline::UnifiedPipeline;

#[derive(Default)]
pub struct TextRendererPlugin;

impl Plugin for TextRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(KayakFontPlugin);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system_to_stage(RenderStage::Queue, create_and_update_font_cache_texture);
    }
}
fn create_and_update_font_cache_texture(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<UnifiedPipeline>,
    mut font_texture_cache: ResMut<FontTextureCache>,
    images: Res<RenderAssets<Image>>,
) {
    font_texture_cache.process_new(&device, &queue, pipeline.into_inner(), &images);
}
