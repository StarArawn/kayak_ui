use crate::bevy::renderer::FontTextureCache;
use crate::KayakFont;
use bevy::prelude::{
    AssetEvent, AssetId, AssetServer, Assets, Commands, EventReader, Handle, Image, Local, Res,
    ResMut, Resource,
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
    mut not_processed: Local<Vec<AssetId<KayakFont>>>,
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
        match *event {
            AssetEvent::Added { id } => {
                changed_assets.insert(id);
            }
            AssetEvent::LoadedWithDependencies { id } => {}
            AssetEvent::Modified { id } => {
                changed_assets.insert(id);
            }
            AssetEvent::Removed { id } => {
                if !changed_assets.remove(&id) {
                    removed.push(id);
                }
            }
        }
    }

    for handle in not_processed.drain(..) {
        changed_assets.insert(handle);
    }

    for id in changed_assets {
        let font_asset = font_assets.get(id).unwrap();
        if let Some(image) = textures.get(font_asset.image.get()) {
            if !image
                .texture_descriptor
                .usage
                .contains(TextureUsages::COPY_SRC)
                || image.texture_descriptor.format != TextureFormat::Rgba8Unorm
            {
                not_processed.push(id);
                continue;
            }
        } else {
            not_processed.push(id);
            continue;
        }

        let font = font_asset.clone();
        extracted_fonts
            .fonts
            .push((asset_server.get_id_handle(id).unwrap(), font));
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
