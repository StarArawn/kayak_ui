use bevy::{
    prelude::{
        AddAsset, AssetEvent, AssetServer, Assets, Commands, EventReader, Handle, Local, Plugin,
        Res, ResMut,
    },
    render2::{
        render_asset::RenderAssets,
        render_resource::{FilterMode, TextureFormat, TextureUsages},
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
        RenderApp, RenderStage,
    },
    utils::HashSet,
};
use kayak_font::{FontTextureCache, KayakFont, KayakFontPlugin};

mod extract;
mod font_mapping;

use self::extract::extract_texts;
use super::pipeline::UnifiedPipeline;
pub use font_mapping::*;

#[derive(Default)]
pub struct TextRendererPlugin;

impl Plugin for TextRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(KayakFontPlugin)
            .init_resource::<FontMapping>();

        let render_app = app.sub_app(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract_texts);
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
