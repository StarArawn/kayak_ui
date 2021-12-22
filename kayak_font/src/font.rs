use std::collections::HashMap;

#[cfg(feature = "bevy_renderer")]
use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset},
    prelude::Handle,
    reflect::TypeUuid,
    render::texture::Image,
};

use crate::Sdf;

#[cfg(feature = "bevy_renderer")]
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4fe4732c-6731-49bb-bafc-4690d636b848"]
pub struct KayakFont {
    pub sdf: Sdf,
    pub atlas_image: Handle<Image>,
    char_ids: HashMap<char, u32>,
}
#[cfg(not(feature = "bevy_renderer"))]
#[derive(Debug, Clone)]
pub struct KayakFont {
    pub sdf: Sdf,
    char_ids: HashMap<char, u32>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LayoutRect {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub content: char,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    PositiveYUp,
    PositiveYDown,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Alignment {
    Start,
    Middle,
    End,
}

impl KayakFont {
    #[cfg(feature = "bevy_renderer")]
    pub fn new(sdf: Sdf, atlas_image: Handle<Image>) -> Self {
        Self {
            sdf,
            atlas_image,
            char_ids: HashMap::default(),
        }
    }

    #[cfg(not(feature = "bevy_renderer"))]
    pub fn new(sdf: Sdf) -> Self {
        Self {
            sdf,
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

    pub fn get_word_width(&self, word: &str, font_size: f32) -> f32 {
        let mut width = 0.0;
        for c in word.chars() {
            if let Some(glyph) = self.sdf.glyphs.iter().find(|glyph| glyph.unicode == c) {
                let plane_bounds = glyph.plane_bounds.as_ref();
                let (_, _, char_width, _) = match plane_bounds {
                    Some(val) => (
                        val.left,
                        val.top,
                        val.size().0 * font_size,
                        val.size().1 * font_size,
                    ),
                    None => (0.0, 0.0, 0.0, 0.0),
                };

                width += char_width;
            }
        }

        width
    }

    pub fn get_layout(
        &self,
        axis_alignment: CoordinateSystem,
        alignment: Alignment,
        position: (f32, f32),
        max_size: (f32, f32),
        content: &String,
        line_height: f32,
        font_size: f32,
    ) -> Vec<LayoutRect> {
        let mut positions_and_size = Vec::new();
        let max_glyph_size = self.sdf.max_glyph_size();
        let font_ratio = font_size / self.sdf.atlas.size;
        let resized_max_glyph_size = (max_glyph_size.0 * font_ratio, max_glyph_size.1 * font_ratio);

        // TODO: Make this configurable?
        let split_chars = vec![' ', '\t', '-', '\n'];
        let missing_chars: Vec<char> = content
            .chars()
            .filter(|c| split_chars.iter().any(|c2| c == c2))
            .collect();

        let shift_sign = match axis_alignment {
            CoordinateSystem::PositiveYDown => -1.0,
            CoordinateSystem::PositiveYUp => 1.0,
        };

        let mut line_widths = Vec::new();

        let mut x = 0.0;
        let mut y = 0.0;
        let mut i = 0;
        let mut line_starting_index = 0;
        let mut last_width = 0.0;
        for word in content.split(&split_chars[..]) {
            let word_width = self.get_word_width(word, font_size);
            if x + word_width > max_size.0 {
                y -= shift_sign * line_height;
                line_widths.push((x, line_starting_index, positions_and_size.len()));
                line_starting_index = positions_and_size.len();
                x = 0.0;
            }
            for c in word.chars() {
                if let Some(glyph) = self.sdf.glyphs.iter().find(|glyph| glyph.unicode == c) {
                    let plane_bounds = glyph.plane_bounds.as_ref();
                    let (left, top, width, _height) = match plane_bounds {
                        Some(val) => (
                            val.left,
                            val.top,
                            val.size().0 * font_size,
                            val.size().1 * font_size,
                        ),
                        None => (0.0, 0.0, 0.0, 0.0),
                    };

                    last_width = width;

                    let position_x = x + left * font_size;
                    let position_y = y + (shift_sign * top * font_size);

                    positions_and_size.push(LayoutRect {
                        position: (position_x, position_y),
                        size: (resized_max_glyph_size.0, resized_max_glyph_size.1),
                        content: c,
                    });

                    x += glyph.advance * font_size;
                }
            }
            if let Some(next_missing) = missing_chars.get(i) {
                if let Some(glyph) = self
                    .sdf
                    .glyphs
                    .iter()
                    .find(|glyph| glyph.unicode == *next_missing)
                {
                    x += glyph.advance * font_size;
                }
                i += 1;
            }
        }

        line_widths.push((
            x + last_width,
            line_starting_index,
            positions_and_size.len(),
        ));

        for (line_width, starting_index, end_index) in line_widths {
            let shift_x = match alignment {
                Alignment::Start => 0.0,
                Alignment::Middle => (max_size.0 - line_width) / 2.0,
                Alignment::End => max_size.0 - line_width,
            };
            for i in starting_index..end_index {
                let layout_rect = &mut positions_and_size[i];

                layout_rect.position.0 += position.0 + shift_x;
                layout_rect.position.1 += position.1;
            }
        }

        positions_and_size
    }
}

#[cfg(feature = "bevy_renderer")]
#[derive(Default)]
pub struct KayakFontLoader;

#[cfg(feature = "bevy_renderer")]
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
