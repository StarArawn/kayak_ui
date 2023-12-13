use crate::{ImageType, KayakFont, Sdf};
use bevy::asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, io::Reader, AsyncReadExt};

#[derive(Default)]
pub struct KayakFontLoader;

impl AssetLoader for KayakFontLoader {
    type Asset = KayakFont;

    type Settings = ();

    type Error = anyhow::Error;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let path = load_context.asset_path().path();
            let path = path.with_extension("png");
            let atlas_image_path = AssetPath::from_path(&path);
            let mut image_asset_context = load_context.begin_labeled_asset();
            let image_handle = image_asset_context.load(atlas_image_path);

            let mut bytes = vec![];
            let _ = reader.read_to_end(&mut bytes).await;
            let font = KayakFont::new(
                Sdf::from_bytes(&bytes),
                ImageType::Atlas(image_handle),
            );

            Ok(font)
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["kayak_font"];
        EXTENSIONS
    }
}
