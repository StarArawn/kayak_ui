use crate::{ImageType, KayakFont, Sdf};
use bevy::asset::{io::Reader, AssetLoader, BoxedFuture, LoadContext};
use futures_lite::AsyncReadExt;

#[derive(Default)]
pub struct KayakFontLoader;

impl AssetLoader for KayakFontLoader {
    type Settings = ();
    type Error = std::io::Error;
    type Asset = KayakFont;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            let path = path.with_extension("png");

            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let font = KayakFont::new(
                Sdf::from_bytes(&bytes),
                ImageType::Atlas(load_context.load(path)),
            );

            Ok(font)
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["kayak_font"];
        EXTENSIONS
    }
}
