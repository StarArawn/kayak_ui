use std::collections::HashMap;

#[cfg(feature = "bevy_renderer")]
use bevy::{prelude::Handle, reflect::TypeUuid, render::texture::Image};
use unicode_segmentation::UnicodeSegmentation;

use crate::layout::{Alignment, Line, TextLayout};
use crate::{utility, Sdf, TextProperties};

#[cfg(feature = "bevy_renderer")]
#[derive(Debug, Clone, TypeUuid, PartialEq)]
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

impl KayakFont {
    pub fn new(sdf: Sdf, #[cfg(feature = "bevy_renderer")] atlas_image: Handle<Image>) -> Self {
        Self {
            sdf,
            #[cfg(feature = "bevy_renderer")]
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

    /// Measures the given text content and calculates an appropriate layout
    /// given a set of properties.
    ///
    /// # Arguments
    ///
    /// * `content`: The textual content to measure.
    /// * `properties`: The text properties to use.
    ///
    pub fn measure(&self, content: &str, properties: TextProperties) -> TextLayout {
        let mut size: (f32, f32) = (0.0, 0.0);
        let mut lines = Vec::new();

        let mut line = Line::default();
        let mut grapheme_index = 0;

        // We'll now split up the text content so that we can measure the layout.
        // This is the "text pipeline" for this function:
        //   1. Split the text by their UAX #29 word boundaries.
        //   2. Split each word by its UAX #29 grapheme clusters.
        //      This step is important since "a̐" is technically two characters (codepoints),
        //      but rendered as a single glyph.
        //   3. Process each character within the grapheme cluster.
        //
        // FIXME: I think #3 is wrong— we probably need to process the full grapheme cluster
        //        rather than each character individually,— however, this can probably be
        //        addressed later. Once resolved, this comment should be updated accordingly.

        for word in content.split_word_bounds() {
            let word_width = self.get_word_width(word, properties.font_size);

            // === Confine to Bounds === //
            if let Some((max_width, _)) = properties.max_size {
                if line.width + word_width > max_width {
                    // Word exceeds bounds -> New line
                    lines.push(line);
                    line = Line {
                        index: grapheme_index,
                        ..Default::default()
                    };
                }
            }

            // === Iterate Grapheme Clusters === //
            for grapheme in word.graphemes(true) {
                // Updated first so that any new lines are using the correct index
                grapheme_index += 1;

                for c in grapheme.chars() {
                    if utility::is_newline(c) {
                        // Character is new line -> New line
                        lines.push(line);
                        line = Line {
                            index: grapheme_index,
                            ..Default::default()
                        };
                    }

                    if let Some(glyph) = self.sdf.glyphs.iter().find(|glyph| glyph.unicode == c) {
                        line.width += glyph.advance * properties.font_size;
                        size.0 = size.0.max(line.width);
                    }
                }
            }
        }

        lines.push(line);
        size.1 = properties.line_height * lines.len() as f32;

        TextLayout::new(lines, size, properties)
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
        let split_chars = vec![' ', '\t'];
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
            if x + word_width + (font_size / 2.0) > max_size.0 {
                y -= shift_sign * line_height;
                line_widths.push((x, line_starting_index, positions_and_size.len()));
                line_starting_index = positions_and_size.len();
                x = 0.0;
            }
            for c in word.chars() {
                if c == '\n' {
                    y -= shift_sign * line_height;
                    line_widths.push((x, line_starting_index, positions_and_size.len()));
                    line_starting_index = positions_and_size.len();
                    x = 0.0;
                }

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
                    let position_y =
                        y + (shift_sign * top * font_size) + ((line_height - font_size) / 2.0);

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
