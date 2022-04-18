use std::collections::HashMap;
use std::iter::Peekable;

#[cfg(feature = "bevy_renderer")]
use bevy::{prelude::Handle, reflect::TypeUuid, render::texture::Image};
use serde_json::de::Read;
use unicode_segmentation::UnicodeSegmentation;
use xi_unicode::LineBreakIterator;

use crate::layout::{Alignment, Line, TextLayout};
use crate::{utility, Sdf, TextProperties, Glyph, GlyphRect};
use crate::utility::{BreakableWord, BreakableWordIter, SPACE};

#[cfg(feature = "bevy_renderer")]
#[derive(Debug, Clone, TypeUuid, PartialEq)]
#[uuid = "4fe4732c-6731-49bb-bafc-4690d636b848"]
pub struct KayakFont {
    pub sdf: Sdf,
    pub atlas_image: Handle<Image>,
    char_ids: HashMap<char, u32>,
    max_glyph_size: (f32, f32),
}

#[cfg(not(feature = "bevy_renderer"))]
#[derive(Debug, Clone)]
pub struct KayakFont {
    pub sdf: Sdf,
    char_ids: HashMap<char, u32>,
    max_glyph_size: (f32, f32),
}

// TODO: Remove me
// #[derive(Default, Debug, Clone, Copy)]
// pub struct LayoutRect {
//     pub position: (f32, f32),
//     pub size: (f32, f32),
//     pub content: char,
// }

// TODO: Remove me (?)
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum CoordinateSystem {
//     PositiveYUp,
//     PositiveYDown,
// }

impl KayakFont {
    pub fn new(sdf: Sdf, #[cfg(feature = "bevy_renderer")] atlas_image: Handle<Image>) -> Self {
        let max_glyph_size = sdf.max_glyph_size();
        assert!(sdf.glyphs.len() < u32::MAX as usize, "SDF contains too many glyphs");
        let char_ids = sdf.glyphs.iter().enumerate().map(|(idx, glyph)| (glyph.unicode, idx as u32)).collect();

        Self {
            sdf,
            #[cfg(feature = "bevy_renderer")]
            atlas_image,
            char_ids,
            max_glyph_size,
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

    pub fn get_word_width(&self, word: &str, properties: TextProperties) -> f32 {
        let space_width = self.get_space_width(properties);
        let tab_width = self.get_tab_width(properties);

        let mut width = 0.0;
        for c in word.chars() {
            if utility::is_space(c) {
                width += space_width;
            } else if utility::is_tab(c) {
                width += tab_width;
            } else if let Some(glyph) = self.get_glyph(c) {
                width += glyph.advance * properties.font_size;
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
        let space_width = self.get_space_width(properties);
        let tab_width = self.get_tab_width(properties);

        let mut size: (f32, f32) = (0.0, 0.0);
        let mut glyph_rects = Vec::new();
        let mut lines = Vec::new();

        // The current line being calculated
        let mut line = Line::default();
        // The current grapheme cluster index
        let mut grapheme_index = 0;
        // The current character index
        let mut char_index = 0;

        // This is the normalized glyph bounds for all glyphs in the atlas.
        // It's needed to ensure all glyphs render proportional to each other.
        let norm_glyph_bounds = self.calc_glyph_size(properties.font_size);

        // The word index to break a line before
        let mut break_index = None;
        // The word index until attempting to find another line break
        let mut skip_until_index = None;

        /// Local function to apply the line break, if any
        fn try_break_line(index: usize, char_index: usize, grapheme_index: usize, line: &mut Line, lines: &mut Vec<Line>, break_index: &mut Option<usize>) {
            if let Some(idx) = break_index {
                if *idx == index {
                    add_line(char_index, grapheme_index, line, lines);
                    *break_index = None;
                }
            }
        }

        /// Local function to finalize the current line and start a new one
        fn add_line(char_index: usize, grapheme_index: usize, line: &mut Line, lines: &mut Vec<Line>) {
            lines.push(*line);
            *line = Line {
                grapheme_index,
                char_index,
                ..Default::default()
            };
        }

        // We'll now split up the text content so that we can measure the layout.
        // This is the "text pipeline" for this function:
        //   1. Split the text by their UAX #29 word boundaries.
        //   2. Split each word by its UAX #29 grapheme clusters.
        //      This step is important since "a̐" is technically two characters (codepoints),
        //      but rendered as a single glyph.
        //   3. Process each character within the grapheme cluster.
        //
        // FIXME: I think #3 is wrong— we probably need to process the full grapheme cluster
        //        rather than each character individually,— however, this might take some
        //        careful thought and consideration, so it should probably be addressed later.
        //        Once resolved, this comment should be updated accordingly.

        let mut words = utility::split_breakable_words(content).collect::<Vec<_>>();
        for (index, word) in words.iter().enumerate() {

            // === Line Break === //
            // If the `break_index` is set, apply it.
            try_break_line(index, char_index, grapheme_index, &mut line, &mut lines, &mut break_index);
            if break_index.is_none() {
                match skip_until_index {
                    Some(idx) if index < idx => {
                        // Skip finding a line break since we're guaranteed not to find one until `idx`
                    }
                    _ => {
                        let (next_break, next_skip) = self.find_next_break(index, &words, line.width, properties);
                        break_index = next_break;
                        skip_until_index = next_skip;
                    }
                }
            }
            // If the `break_index` is set, apply it
            try_break_line(index, char_index, grapheme_index, &mut line, &mut lines, &mut break_index);

            // === Iterate Grapheme Clusters === //
            for grapheme in word.content.graphemes(true) {
                // Updated first so that any new lines are using the correct index
                grapheme_index += 1;
                line.grapheme_len += 1;

                for c in grapheme.chars() {
                    if utility::is_newline(c) {
                        // Character is new line -> New line
                        add_line(char_index, grapheme_index, &mut line, &mut lines);
                        continue;
                    }

                    if utility::is_space(c) {
                        line.width += space_width;
                    } else if utility::is_tab(c) {
                        line.width += tab_width;
                    } else if let Some(glyph) = self.get_glyph(c) {
                        // Character is valid glyph -> calculate its size and position
                        let plane_bounds = glyph.plane_bounds.as_ref();
                        let (left, top, _width, _height) = match plane_bounds {
                            Some(rect) => (
                                rect.left,
                                rect.top,
                                rect.width() * properties.font_size,
                                rect.height() * properties.font_size,
                            ),
                            None => (0.0, 0.0, 0.0, 0.0),
                        };

                        // Calculate position relative to line and normalized glyph bounds
                        let pos_x = line.width + left * properties.font_size;
                        let mut pos_y = properties.line_height * lines.len() as f32;
                        pos_y -= top * properties.font_size;

                        glyph_rects.push(GlyphRect {
                            position: (pos_x, pos_y),
                            size: norm_glyph_bounds,
                            content: c,
                        });

                        char_index += 1;
                        line.char_len += 1;
                        line.width += glyph.advance * properties.font_size;
                    }

                    size.0 = size.0.max(line.width);
                }
            }
        }

        // Push the final line
        lines.push(line);
        size.1 = properties.line_height * lines.len() as f32;

        // === Shift Lines & Glyphs === //
        for line in lines.iter() {
            let shift_x = match properties.alignment {
                Alignment::Start => 0.0,
                Alignment::Middle => (properties.max_size.0 - line.width) / 2.0,
                Alignment::End => properties.max_size.0 - line.width,
            };

            let start = line.char_index;
            let end = start + line.char_len;

            for index in start..end {
                let rect = &mut glyph_rects[index];
                rect.position.0 += shift_x;
            }
        }

        TextLayout::new(glyph_rects, lines, size, properties)
    }

    /// Attempts to find the next line break for a given set of [breakable words](BreakableWord).
    ///
    /// # Returns
    ///
    /// A tuple. The first field of the tuple indicates which word index to break _before_, if any.
    /// The second field indicates which word index to wait _until_ before calling this method again
    /// (exclusive), if any. The reason for the second field is that there are cases where the line
    /// break behavior can be accounted for ahead of time.
    ///
    /// # Arguments
    ///
    /// * `index`: The current word index
    /// * `words`: The list of breakable words
    /// * `line_width`: The current line's current width
    /// * `properties`: The associated text properties
    ///
    fn find_next_break(&self, index: usize, words: &[BreakableWord], line_width: f32, properties: TextProperties) -> (Option<usize>, Option<usize>) {
        let curr_index = index;
        let mut next_index = index + 1;

        let curr = if let Some(curr) = words.get(curr_index) {
            curr
        } else {
            return (None, None);
        };

        if curr.hard_break {
            // Hard break -> break before next word
            return (Some(next_index), None)
        }

        let mut total_width = self.get_word_width(curr.content, properties);

        if curr.content.ends_with(char::is_whitespace) {
            // End in whitespace -> allow line break if needed

            let next = if let Some(next) = words.get(next_index) {
                next
            } else {
                return (None, None);
            };
            total_width += self.get_word_width(next.content.trim_end(), properties);

            // Current word will not be joining the next word
            return if total_width + line_width > properties.max_size.0 {
                // Break before the next word
                (Some(next_index), None)
            } else {
                // No break needed
                (None, None)
            };
        }

        let mut best_break_point = if total_width + line_width <= properties.max_size.0 {
            // Joined word could fit on current line
            Some(next_index)
        } else {
            // Joined word should start on new line
            Some(index)
        };

        while let Some(word) = words.get(next_index) {
            total_width += self.get_word_width(word.content, properties);

            if total_width + line_width <= properties.max_size.0 {
                // Still within confines of LINE -> break line here if needed
                best_break_point = Some(next_index + 1);
            }

            if word.content.ends_with(char::is_whitespace) {
                // End of joining words
                break;
            }

            next_index += 1;
        }

        // The index to skip until (i.e. the last joined word).
        let skip_until_index = next_index - 1;

        if total_width + line_width <= properties.max_size.0 {
            // Still within confines of LINE -> no need to break
            return (None, Some(skip_until_index));
        }

        if total_width <= properties.max_size.0 {
            // Still within confines of MAX (can fit within a single line)
            return (Some(index), Some(skip_until_index));
        }

        // Attempt to break at the best possible point
        (best_break_point, Some(skip_until_index))
    }

    /// Returns the pixel width of a space.
    fn get_space_width(&self, properties: TextProperties) -> f32 {
        if let Some(glyph) = self.get_glyph(SPACE) {
            glyph.advance * properties.font_size
        } else {
            0.0
        }
    }

    /// Returns the pixel width of a tab.
    fn get_tab_width(&self, properties: TextProperties) -> f32 {
        self.get_space_width(properties) * properties.tab_size as f32
    }

    /// Attempts to find the glyph corresponding to the given character.
    ///
    /// Returns `None` if no glyph was found.
    pub fn get_glyph(&self, c: char) -> Option<&Glyph> {
        self.char_ids.get(&c).and_then(|index| self.sdf.glyphs.get(*index as usize))
    }

    /// Calculates the appropriate glyph size for a desired font size.
    ///
    /// This glyph size can then be used to provide a normalized size across all glyphs
    /// in the atlas.
    fn calc_glyph_size(&self, font_size: f32) -> (f32, f32) {
        let font_scale = font_size / self.sdf.atlas.font_size;
        (self.max_glyph_size.0 * font_scale, self.max_glyph_size.1 * font_scale)
    }

    // TODO: Remove
    // pub fn get_layout(
    //     &self,
    //     axis_alignment: CoordinateSystem,
    //     alignment: Alignment,
    //     position: (f32, f32),
    //     max_size: (f32, f32),
    //     content: &String,
    //     line_height: f32,
    //     font_size: f32,
    // ) -> Vec<LayoutRect> {
    //     let mut positions_and_size = Vec::new();
    //     let max_glyph_size = self.sdf.max_glyph_size();
    //     let font_ratio = font_size / self.sdf.atlas.font_size;
    //     let resized_max_glyph_size = (max_glyph_size.0 * font_ratio, max_glyph_size.1 * font_ratio);
    //
    //     // TODO: Make this configurable?
    //     let split_chars = vec![' ', '\t'];
    //     let missing_chars: Vec<char> = content
    //         .chars()
    //         .filter(|c| split_chars.iter().any(|c2| c == c2))
    //         .collect();
    //
    //     let shift_sign = match axis_alignment {
    //         CoordinateSystem::PositiveYDown => -1.0,
    //         CoordinateSystem::PositiveYUp => 1.0,
    //     };
    //
    //     let mut line_widths = Vec::new();
    //
    //     let mut x = 0.0;
    //     let mut y = 0.0;
    //     let mut i = 0;
    //     let mut line_starting_index = 0;
    //     let mut last_width = 0.0;
    //     for word in content.split(&split_chars[..]) {
    //         let word_width = self.get_word_width(word, TextProperties::default());
    //         if x + word_width + (font_size / 2.0) > max_size.0 {
    //             y -= shift_sign * line_height;
    //             line_widths.push((x, line_starting_index, positions_and_size.len()));
    //             line_starting_index = positions_and_size.len();
    //             x = 0.0;
    //         }
    //         for c in word.chars() {
    //             if c == '\n' {
    //                 y -= shift_sign * line_height;
    //                 line_widths.push((x, line_starting_index, positions_and_size.len()));
    //                 line_starting_index = positions_and_size.len();
    //                 x = 0.0;
    //             }
    //
    //             if let Some(glyph) = self.get_glyph(c) {
    //                 let plane_bounds = glyph.plane_bounds.as_ref();
    //                 let (left, top, width, _height) = match plane_bounds {
    //                     Some(val) => (
    //                         val.left,
    //                         val.top,
    //                         val.size().0 * font_size,
    //                         val.size().1 * font_size,
    //                     ),
    //                     None => (0.0, 0.0, 0.0, 0.0),
    //                 };
    //
    //                 last_width = width;
    //
    //                 let position_x = x + left * font_size;
    //                 let position_y =
    //                     y + (shift_sign * top * font_size) + ((line_height - font_size) / 2.0);
    //
    //                 positions_and_size.push(LayoutRect {
    //                     position: (position_x, position_y),
    //                     size: (resized_max_glyph_size.0, resized_max_glyph_size.1),
    //                     content: c,
    //                 });
    //
    //                 x += glyph.advance * font_size;
    //             }
    //         }
    //         if let Some(next_missing) = missing_chars.get(i) {
    //             if let Some(glyph) = self
    //                 .sdf
    //                 .glyphs
    //                 .iter()
    //                 .find(|glyph| glyph.unicode == *next_missing)
    //             {
    //                 x += glyph.advance * font_size;
    //             }
    //             i += 1;
    //         }
    //     }
    //
    //     line_widths.push((
    //         x + last_width,
    //         line_starting_index,
    //         positions_and_size.len(),
    //     ));
    //
    //     for (line_width, starting_index, end_index) in line_widths {
    //         let shift_x = match alignment {
    //             Alignment::Start => 0.0,
    //             Alignment::Middle => (max_size.0 - line_width) / 2.0,
    //             Alignment::End => max_size.0 - line_width,
    //         };
    //         for i in starting_index..end_index {
    //             let layout_rect = &mut positions_and_size[i];
    //
    //             layout_rect.position.0 += position.0 + shift_x;
    //             layout_rect.position.1 += position.1;
    //         }
    //     }
    //
    //     positions_and_size
    // }
}
