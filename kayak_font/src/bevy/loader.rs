use crate::{KayakFont, Sdf, ImageType};
use bevy::asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset};

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
                ImageType::Atlas(load_context.get_handle(atlas_image_path.clone())),
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
