use crate::bevy::renderer::FontTextureCache;
use crate::KayakFont;
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
        if let Some(image) = textures.get(&font_asset.atlas_image) {
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
