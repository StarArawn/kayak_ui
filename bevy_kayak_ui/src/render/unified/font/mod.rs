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

use self::{extract::extract_texts, sdf::Sdf};

use super::pipeline::UnifiedPipeline;

mod extract;
mod font;
mod font_mapping;
mod font_texture_cache;
mod sdf;

pub use font::*;
pub use font_mapping::*;
pub(crate) use font_texture_cache::FontTextureCache;

#[derive(Default)]
pub struct TextRendererPlugin;

impl Plugin for TextRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<KayakFont>()
            .init_resource::<FontMapping>()
            .add_startup_system(load_fonts)
            .add_system(set_font_texture);

        let render_app = app.sub_app(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract_texts);

        render_app
            .init_resource::<FontTextureCache>()
            .init_resource::<ExtractedFonts>()
            .add_system_to_stage(RenderStage::Extract, extract_fonts)
            .add_system_to_stage(RenderStage::Prepare, prepare_fonts)
            .add_system_to_stage(RenderStage::Queue, create_and_update_font_cache_texture);
    }
}

#[derive(Default)]
pub struct ExtractedFonts {
    pub fonts: Vec<(Handle<KayakFont>, KayakFont)>,
}

fn load_fonts(
    mut font_assets: ResMut<Assets<KayakFont>>,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let font_bytes = include_bytes!("../../../../../resources/Roboto-Regular.ttf");
    let sdf = Sdf::from_string(include_str!("../../../../../assets/roboto.json").to_string());
    let max_glyph_size = sdf.max_glyph_size();
    let mut font = kayak_font::Font::new(
        font_bytes,
        (max_glyph_size.x as u32, max_glyph_size.y as u32),
    );

    let atlas_image: Handle<Image> = asset_server.load("roboto.png");

    for glyph in sdf.glyphs.iter() {
        if !font.cache.has_character(glyph.unicode) {
            font.cache.add_character(glyph.unicode);
        }
    }

    let handle = font_assets.add(KayakFont {
        font,
        atlas_image: Some(atlas_image),
        sdf: Some(sdf),
    });
    font_mapping.add(handle);
}

pub fn set_font_texture(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        dbg!(&event);
        match event {
            AssetEvent::Created { handle } => {
                let handle_path = asset_server.get_handle_path(handle).unwrap();
                dbg!(&handle_path);
                if handle_path.path().to_str().unwrap().contains("roboto") {
                    if let Some(mut texture) = textures.get_mut(handle) {
                        dbg!("Setting font texture!");
                        texture.texture_descriptor.format = TextureFormat::Rgba8Unorm;
                        texture.sampler_descriptor.min_filter = FilterMode::Linear;
                        texture.sampler_descriptor.mipmap_filter = FilterMode::Linear;
                        texture.sampler_descriptor.mag_filter = FilterMode::Linear;
                        texture.texture_descriptor.usage =
                            TextureUsages::COPY_DST | TextureUsages::COPY_SRC;
                    }
                }
            }
            _ => (),
        }
    }
}

fn extract_fonts(
    mut not_processed: Local<Vec<Handle<KayakFont>>>,
    mut commands: Commands,
    font_assets: Res<Assets<KayakFont>>,
    mut events: EventReader<AssetEvent<KayakFont>>,
    mut textures: ResMut<Assets<Image>>,
) {
    let mut extracted_fonts = ExtractedFonts { fonts: Vec::new() };
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Modified { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                if !changed_assets.remove(handle) {
                    removed.push(handle.clone_weak());
                }
            }
        }
    }

    for handle in not_processed.drain(..) {
        changed_assets.insert(handle);
    }

    for handle in changed_assets {
        let font_asset = font_assets.get(&handle).unwrap();
        if let Some(image) = font_asset.atlas_image.as_ref() {
            if let Some(image) = textures.get(image) {
                if !image
                    .texture_descriptor
                    .usage
                    .contains(TextureUsages::COPY_SRC)
                {
                    not_processed.push(handle);
                    continue;
                }
            } else {
                not_processed.push(handle);
                continue;
            }
        }

        let font = font_asset.clone();
        extracted_fonts.fonts.push((handle, font));
    }

    commands.insert_resource(extracted_fonts);
}

fn prepare_fonts(
    mut extracted_fonts: ResMut<ExtractedFonts>,
    mut font_texture_cache: ResMut<FontTextureCache>,
) {
    let fonts: Vec<_> = extracted_fonts.fonts.drain(..).collect();
    for (handle, font) in fonts {
        font_texture_cache.add(handle, font);
    }
}

fn create_and_update_font_cache_texture(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<UnifiedPipeline>,
    mut font_texture_cache: ResMut<FontTextureCache>,
    images: Res<RenderAssets<Image>>,
) {
    font_texture_cache.process_new(&device, &queue, &pipeline, &images);
    font_texture_cache.process_updated(&queue);
}
