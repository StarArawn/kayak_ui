use crate::bevy::renderer::FontTextureCache;
use crate::KayakFont;
use bevy::asset::AssetServer;
use bevy::prelude::{
    AssetEvent, Assets, Commands, EventReader, Handle, Image, Local, Res, ResMut, Resource,
};
use bevy::render::{
    render_resource::{TextureFormat, TextureUsages},
    Extract,
};
use bevy::utils::HashSet;

#[derive(Default, Resource)]
pub struct ExtractedFonts {
    pub fonts: Vec<(Handle<KayakFont>, KayakFont)>,
}

pub(crate) fn extract_fonts(
    mut not_processed: Local<Vec<Handle<KayakFont>>>,
    mut commands: Commands,
    font_assets: Extract<Res<Assets<KayakFont>>>,
    mut events: Extract<EventReader<AssetEvent<KayakFont>>>,
    textures: Extract<Res<Assets<Image>>>,
    asset_server: Res<AssetServer>,
) {
    let mut extracted_fonts = ExtractedFonts { fonts: Vec::new() };
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.read() {
        match event {
            AssetEvent::Added { id } => {
                let id = asset_server.get_id_handle(*id).unwrap();
                changed_assets.insert(id.clone_weak());
            }
            AssetEvent::Modified { id } => {
                let id = asset_server.get_id_handle(*id).unwrap();
                changed_assets.insert(id.clone_weak());
            }
            AssetEvent::Removed { id } => {
                let id = asset_server.get_id_handle(*id).unwrap();
                if !changed_assets.remove(&id) {
                    removed.push(id.clone_weak());
                }
            }
            AssetEvent::LoadedWithDependencies { id } => {
                let id = asset_server.get_id_handle(*id).unwrap();
                changed_assets.insert(id.clone_weak());
            }
        }
    }

    for handle in not_processed.drain(..) {
        changed_assets.insert(handle);
    }

    for handle in changed_assets {
        let font_asset = font_assets.get(&handle).unwrap();
        if let Some(image) = textures.get(font_asset.image.get()) {
            if !image
                .texture_descriptor
                .usage
                .contains(TextureUsages::COPY_SRC)
                || image.texture_descriptor.format != TextureFormat::Rgba8Unorm
            {
                not_processed.push(handle);
                continue;
            }
        } else {
            not_processed.push(handle);
            continue;
        }

        let font = font_asset.clone();
        extracted_fonts.fonts.push((handle, font));
    }

    commands.insert_resource(extracted_fonts);
}

pub(crate) fn prepare_fonts(
    mut extracted_fonts: ResMut<ExtractedFonts>,
    mut font_texture_cache: ResMut<FontTextureCache>,
) {
    let fonts: Vec<_> = extracted_fonts.fonts.drain(..).collect();
    for (handle, font) in fonts {
        font_texture_cache.add(handle, font);
    }
}
