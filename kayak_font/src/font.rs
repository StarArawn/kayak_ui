use std::collections::HashMap;

use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset},
    math::Vec2,
    prelude::Handle,
    reflect::TypeUuid,
    render2::texture::Image,
};

use crate::Sdf;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4fe4732c-6731-49bb-bafc-4690d636b848"]
pub struct KayakFont {
    pub sdf: Sdf,
    pub atlas_image: Handle<Image>,
    char_ids: HashMap<char, u32>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LayoutRect {
    pub position: Vec2,
    pub size: Vec2,
    pub content: char,
}

#[derive(Debug, Clone, Copy)]
pub enum CoordinateSystem {
    PositiveYUp,
    PositiveYDown,
}

impl KayakFont {
    pub fn new(sdf: Sdf, atlas_image: Handle<Image>) -> Self {
        Self {
            sdf,
            atlas_image,
            char_ids: HashMap::default(),
        }
    }

    pub fn generate_char_ids(&mut self) {
        let mut count = 0;
        for glyph in self.sdf.glyphs.iter() {
            self.char_ids.insert(glyph.unicode, count);
            count += 1;
        }
    }

    pub fn get_char_id(&self, c: char) -> Option<u32> {
        self.char_ids.get(&c).and_then(|id| Some(*id))
    }

    pub fn get_layout(
        &self,
        axis_alignment: CoordinateSystem,
        position: Vec2,
        content: &String,
        font_size: f32,
    ) -> Vec<LayoutRect> {
        let mut positions_and_size = Vec::new();
        let max_glyph_size = self.sdf.max_glyph_size();
        let font_ratio = font_size / self.sdf.atlas.size;
        let resized_max_glyph_size = (max_glyph_size.x * font_ratio, max_glyph_size.y * font_ratio);

        let mut x = 0.0;
        for c in content.chars() {
            if let Some(glyph) = self.sdf.glyphs.iter().find(|glyph| glyph.unicode == c) {
                let plane_bounds = glyph.plane_bounds.as_ref();
                let (left, top, _width, _height) = match plane_bounds {
                    Some(val) => (
                        val.left,
                        val.top,
                        val.size().x * font_size,
                        val.size().y * font_size,
                    ),
                    None => (0.0, 0.0, 0.0, 0.0),
                };

                let shift_sign = match axis_alignment {
                    CoordinateSystem::PositiveYDown => -1.0,
                    CoordinateSystem::PositiveYUp => 1.0,
                };

                let position_x = position.x + x + left * font_size;
                let position_y = (position.y + (shift_sign * top * font_size)) + font_size;

                positions_and_size.push(LayoutRect {
                    position: Vec2::new(position_x, position_y),
                    size: Vec2::new(resized_max_glyph_size.0, resized_max_glyph_size.1),
                    content: c,
                });

                x += glyph.advance * font_size;
            }
        }

        positions_and_size
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
            let mut font = KayakFont::new(
                Sdf::from_bytes(bytes),
                load_context.get_handle(atlas_image_path.clone()),
            );

            font.generate_char_ids();

            load_context
                .set_default_asset(LoadedAsset::new(font).with_dependency(atlas_image_path));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["kayak_font"];
        EXTENSIONS
    }
}
