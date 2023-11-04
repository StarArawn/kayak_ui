use bevy::{
    prelude::{IntoSystemConfigs, Plugin, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
        Render, RenderApp, RenderSet,
    },
};
use kayak_font::bevy::{FontTextureCache, KayakFontPlugin};

#[derive(Default)]
pub struct TextRendererPlugin;

impl Plugin for TextRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(KayakFontPlugin);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            create_and_update_font_cache_texture.in_set(RenderSet::Queue),
        );
    }
}
fn create_and_update_font_cache_texture(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut font_texture_cache: ResMut<FontTextureCache>,
    images: Res<RenderAssets<Image>>,
) {
    font_texture_cache.process_new(&device, &queue, &images);
}
