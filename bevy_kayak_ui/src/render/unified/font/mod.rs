use bevy::{
    prelude::{Assets, Plugin, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
        RenderApp, RenderStage,
    },
};
use kayak_font::{
    bevy::{FontTextureCache, KayakFontPlugin},
    KayakFont,
};

mod extract;
mod font_mapping;

use crate::BevyContext;

use super::pipeline::UnifiedPipeline;
pub use extract::extract_texts;
pub use font_mapping::*;

#[derive(Default)]
pub struct TextRendererPlugin;

impl Plugin for TextRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(KayakFontPlugin)
            .init_resource::<FontMapping>()
            .add_system(process_loaded_fonts);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system_to_stage(RenderStage::Queue, create_and_update_font_cache_texture);
    }
}

fn process_loaded_fonts(
    mut font_mapping: ResMut<FontMapping>,
    fonts: Res<Assets<KayakFont>>,
    bevy_context: Option<Res<BevyContext>>,
) {
    if let Some(context) = bevy_context {
        if context.is_added() {
            font_mapping.mark_all_as_new();
        }
        font_mapping.add_loaded_to_kayak(&fonts, &context);
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
