use crate::KayakFont;
use bevy::prelude::{AssetEvent, AssetId, Assets, EventReader, Image, Local, Res, ResMut};
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};

pub fn init_font_texture(
    mut not_processed: Local<Vec<AssetId<KayakFont>>>,
    mut font_events: EventReader<AssetEvent<KayakFont>>,
    mut images: ResMut<Assets<Image>>,
    fonts: Res<Assets<KayakFont>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in font_events.read() {
        if let AssetEvent::Added { id } = event {
            not_processed.push(*id);
        }
    }

    let not_processed_fonts = not_processed.drain(..).collect::<Vec<_>>();
    for font_id in not_processed_fonts {
        if let Some(font) = fonts.get(font_id) {
            if let Some(texture) = images.get_mut(font.image.get()) {
                texture.texture_descriptor.format = TextureFormat::Rgba8Unorm;
                texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    label: Some("Present Sampler".into()),
                    mag_filter: ImageFilterMode::Linear,
                    min_filter: ImageFilterMode::Linear,

                    ..Default::default()
                });
                texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC;
            } else {
                not_processed.push(font_id);
            }
        }
    }
}
