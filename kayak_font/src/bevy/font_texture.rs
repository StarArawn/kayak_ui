use crate::KayakFont;
use bevy::prelude::{AssetEvent, Assets, EventReader, Handle, Image, Local, Res, ResMut};
use bevy::render::render_resource::{FilterMode, SamplerDescriptor, TextureFormat, TextureUsages};
use bevy::render::texture::ImageSampler;

pub fn init_font_texture(
    mut not_processed: Local<Vec<Handle<KayakFont>>>,
    mut font_events: EventReader<AssetEvent<KayakFont>>,
    mut images: ResMut<Assets<Image>>,
    fonts: Res<Assets<KayakFont>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in font_events.iter() {
        if let AssetEvent::Created { handle } = event {
            not_processed.push(handle.clone_weak());
        }
    }

    let not_processed_fonts = not_processed.drain(..).collect::<Vec<_>>();
    for font_handle in not_processed_fonts {
        if let Some(font) = fonts.get(&font_handle) {
            if let Some(mut texture) = images.get_mut(font.image.get()) {
                texture.texture_descriptor.format = TextureFormat::Rgba8Unorm;
                texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    label: Some("Present Sampler"),
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Linear,

                    ..Default::default()
                });
                texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC;
            } else {
                not_processed.push(font_handle.clone_weak());
            }
        }
    }
}
