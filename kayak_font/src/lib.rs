mod atlas;
mod font;
mod glyph;
mod layout;
mod metrics;
mod sdf;
mod utility;

pub use atlas::*;
pub use font::*;
pub use glyph::*;
pub use layout::*;
pub use metrics::*;
pub use sdf::*;

#[cfg(feature = "bevy_renderer")]
mod renderer;

#[cfg(feature = "bevy_renderer")]
pub mod bevy {
    pub use crate::renderer::*;
    use crate::{KayakFont, Sdf};
    use bevy::{
        asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset},
        prelude::{
            AddAsset, AssetEvent, Assets, Commands, EventReader, Handle, Local, Plugin, Res, ResMut,
        },
        render::{
            render_resource::{FilterMode, TextureFormat, TextureUsages},
            texture::Image,
            RenderApp, RenderStage,
        },
        utils::HashSet,
    };

    pub struct KayakFontPlugin;

    impl Plugin for KayakFontPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_asset::<KayakFont>()
                .add_asset_loader(KayakFontLoader)
                .add_system(init_font_texture);

            let render_app = app.sub_app_mut(RenderApp);
            render_app
                .init_resource::<FontTextureCache>()
                .init_resource::<ExtractedFonts>()
                .add_system_to_stage(RenderStage::Extract, extract_fonts)
                .add_system_to_stage(RenderStage::Prepare, prepare_fonts);
        }
    }

    pub fn init_font_texture(
        mut not_processed: Local<Vec<Handle<KayakFont>>>,
        mut font_events: EventReader<AssetEvent<KayakFont>>,
        mut images: ResMut<Assets<Image>>,
        fonts: Res<Assets<KayakFont>>,
    ) {
        // quick and dirty, run this for all textures anytime a texture is created.
        for event in font_events.iter() {
            match event {
                AssetEvent::Created { handle } => {
                    not_processed.push(handle.clone_weak());
                }
                _ => (),
            }
        }

        let not_processed_fonts = not_processed.drain(..).collect::<Vec<_>>();
        for font_handle in not_processed_fonts {
            if let Some(font) = fonts.get(&font_handle) {
                if let Some(mut texture) = images.get_mut(&font.atlas_image) {
                    texture.texture_descriptor.format = TextureFormat::Rgba8Unorm;
                    texture.sampler_descriptor.min_filter = FilterMode::Linear;
                    texture.sampler_descriptor.mipmap_filter = FilterMode::Linear;
                    texture.sampler_descriptor.mag_filter = FilterMode::Linear;
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::COPY_SRC;
                } else {
                    not_processed.push(font_handle.clone_weak());
                }
            }
        }
    }

    #[derive(Default)]
    pub struct ExtractedFonts {
        pub fonts: Vec<(Handle<KayakFont>, KayakFont)>,
    }

    fn extract_fonts(
        mut not_processed: Local<Vec<Handle<KayakFont>>>,
        mut commands: Commands,
        font_assets: Res<Assets<KayakFont>>,
        mut events: EventReader<AssetEvent<KayakFont>>,
        textures: Res<Assets<Image>>,
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

    fn prepare_fonts(
        mut extracted_fonts: ResMut<ExtractedFonts>,
        mut font_texture_cache: ResMut<FontTextureCache>,
    ) {
        let fonts: Vec<_> = extracted_fonts.fonts.drain(..).collect();
        for (handle, font) in fonts {
            font_texture_cache.add(handle, font);
        }
    }

    #[derive(Default)]
    pub struct KayakFontLoader;

    impl AssetLoader for KayakFontLoader {
        fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut LoadContext,
        ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
            Box::pin(async move {
                let path = load_context.path();
                let path = path.with_extension("png");
                let atlas_image_path = AssetPath::new(path, None);
                let font = KayakFont::new(
                    Sdf::from_bytes(bytes),
                    load_context.get_handle(atlas_image_path.clone()),
                );

                let asset = LoadedAsset::new(font).with_dependency(atlas_image_path);
                load_context.set_default_asset(asset);

                Ok(())
            })
        }

        fn extensions(&self) -> &[&str] {
            static EXTENSIONS: &[&str] = &["kayak_font"];
            EXTENSIONS
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Alignment, KayakFont, Sdf, TextProperties};

    fn make_font() -> KayakFont {
        let bytes = std::fs::read("assets/roboto.kayak_font")
            .expect("a `roboto.kayak_font` file in the `assets/` directory of this crate");

        #[cfg(feature = "bevy_renderer")]
        return KayakFont::new(Sdf::from_bytes(&bytes), bevy::asset::Handle::default());

        #[cfg(not(feature = "bevy_renderer"))]
        return KayakFont::new(Sdf::from_bytes(&bytes));
    }

    fn make_properties() -> TextProperties {
        TextProperties {
            line_height: 14.0 * 1.2,
            font_size: 14.0,
            alignment: Alignment::Start,
            max_size: (200.0, 300.0),
            tab_size: 4,
        }
    }

    #[test]
    fn should_contain_correct_number_of_chars() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        assert_eq!(content.len(), layout.total_chars())
    }

    #[test]
    fn should_contain_correct_number_of_glyphs() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        // Since this string is pure ascii, we can just get the total characters - total whitespace
        let expected = content.split_whitespace().collect::<String>().len();

        assert_eq!(expected, layout.total_glyphs())
    }

    #[test]
    fn should_contain_correct_number_of_graphemes() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        // Since this string is pure ascii, we can just get the total characters
        let expected = content.len();

        assert_eq!(expected, layout.total_graphemes())
    }

    #[test]
    fn should_contain_correct_number_of_lines() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        assert_eq!(4, layout.total_lines())
    }

    #[test]
    fn should_find_line_containing_grapheme() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        let lines = [
            (content.find("Hello").unwrap(), content.rfind('\n').unwrap()),
            (
                content.find("How").unwrap(),
                content.rfind("this ").unwrap(),
            ),
            (
                content.find("super").unwrap(),
                content.rfind("doing ").unwrap(),
            ),
            (content.find("today").unwrap(), content.rfind('?').unwrap()),
        ];

        for (line_index, (line_start, line_end)) in lines.into_iter().enumerate() {
            let result = layout.find_grapheme(line_start).unwrap().row;
            assert_eq!(line_index, result);
            let result = layout.find_grapheme(line_end).unwrap().row;
            assert_eq!(line_index, result);
        }
    }

    #[test]
    fn grapheme_should_be_correct_position() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        for (line_index, line) in layout.lines().iter().enumerate() {
            let mut expected_x = 0.0;
            let expected_y = properties.line_height * line_index as f32;

            for grapheme in line.graphemes() {
                assert_eq!(expected_x, grapheme.position.0);
                assert_eq!(expected_y, grapheme.position.1);
                expected_x += grapheme.size.0;
            }
        }
    }
}
