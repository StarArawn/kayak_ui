use std::collections::HashMap;

#[cfg(feature = "bevy_renderer")]
use bevy::{prelude::Handle, reflect::TypeUuid, render::texture::Image};
use unicode_segmentation::UnicodeSegmentation;

use crate::utility::{BreakableWord, MISSING, SPACE};
use crate::{
    utility, Alignment, Glyph, GlyphRect, Grapheme, Line, Sdf, TextLayout, TextProperties,
};

#[cfg(feature = "bevy_renderer")]
#[derive(Debug, Clone, TypeUuid, PartialEq)]
#[uuid = "4fe4732c-6731-49bb-bafc-4690d636b848"]
pub struct KayakFont {
    pub sdf: Sdf,
    pub atlas_image: Handle<Image>,
    pub missing_glyph: Option<char>,
    char_ids: HashMap<char, u32>,
    max_glyph_size: (f32, f32),
}

#[cfg(not(feature = "bevy_renderer"))]
#[derive(Debug, Clone)]
pub struct KayakFont {
    pub sdf: Sdf,
    pub missing_glyph: Option<char>,
    char_ids: HashMap<char, u32>,
    max_glyph_size: (f32, f32),
}

impl KayakFont {
    pub fn new(sdf: Sdf, #[cfg(feature = "bevy_renderer")] atlas_image: Handle<Image>) -> Self {
        let max_glyph_size = sdf.max_glyph_size();
        assert!(
            sdf.glyphs.len() < u32::MAX as usize,
            "SDF contains too many glyphs"
        );

        let char_ids: HashMap<char, u32> = sdf
            .glyphs
            .iter()
            .enumerate()
            .map(|(idx, glyph)| (glyph.unicode, idx as u32))
            .collect();

        let missing_glyph = if char_ids.contains_key(&MISSING) {
            Some(MISSING)
        } else if char_ids.contains_key(&SPACE) {
            Some(SPACE)
        } else {
            None
        };

        Self {
            sdf,
            #[cfg(feature = "bevy_renderer")]
            atlas_image,
            missing_glyph,
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

        // This is the normalized glyph bounds for all glyphs in the atlas.
        // It's needed to ensure all glyphs render proportional to each other.
        let norm_glyph_bounds = self.calc_glyph_size(properties.font_size);

        // The current line being calculated
        let mut line = Line::new(0);
        let mut glyph_index = 0;
        let mut char_index = 0;

        // The word index to break a line before
        let mut break_index = None;
        // The word index until attempting to find another line break
        let mut skip_until_index = None;

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

        let words = utility::split_breakable_words(content).collect::<Vec<_>>();
        for (index, word) in words.iter().enumerate() {
            // Check if this is the last word of the line.
            let mut will_break = break_index.map(|idx| index + 1 == idx).unwrap_or_default();

            // === Line Break === //
            // If the `break_index` is set, see if it applies.
            if let Some(idx) = break_index {
                if idx == index {
                    let next_line = Line::new_after(&line);
                    lines.push(line);
                    line = next_line;
                    break_index = None;
                }
            }

            if break_index.is_none() {
                match skip_until_index {
                    Some(idx) if index < idx => {
                        // Skip finding a line break since we're guaranteed not to find one until `idx`
                    }
                    _ => {
                        let (next_break, next_skip) =
                            self.find_next_break(index, line.width(), properties, &words);
                        break_index = next_break;
                        skip_until_index = next_skip;
                        will_break |= break_index.map(|idx| index + 1 == idx).unwrap_or_default();
                    }
                }
            }

            // === Iterate Grapheme Clusters === //
            for grapheme_content in word.content.graphemes(true) {
                let mut grapheme = Grapheme {
                    position: (line.width(), properties.line_height * lines.len() as f32),
                    glyph_index,
                    char_index,
                    ..Default::default()
                };

                for c in grapheme_content.chars() {
                    char_index += 1;
                    grapheme.char_total += 1;

                    if utility::is_newline(c) {
                        // Newlines (hard breaks) are already accounted for by the line break algorithm
                        continue;
                    }

                    if utility::is_space(c) {
                        if !will_break {
                            // Don't add the space if we're about to break the line
                            grapheme.size.0 += space_width;
                        }
                    } else if utility::is_tab(c) {
                        grapheme.size.0 += tab_width;
                    } else {
                        let glyph = self.get_glyph(c).or_else(|| {
                            if let Some(missing) = self.missing_glyph {
                                self.get_glyph(missing)
                            } else {
                                None
                            }
                        });

                        if let Some(glyph) = glyph {
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
                            let pos_x = (grapheme.position.0 + grapheme.size.0)
                                + left * properties.font_size;
                            let pos_y = (grapheme.position.1 + grapheme.size.1)
                                - top * properties.font_size;

                            glyph_rects.push(GlyphRect {
                                position: (pos_x, pos_y),
                                size: norm_glyph_bounds,
                                content: glyph.unicode,
                            });

                            glyph_index += 1;
                            grapheme.glyph_total += 1;
                            grapheme.size.0 += glyph.advance * properties.font_size;
                        }
                    }
                }

                line.add_grapheme(grapheme);
                size.0 = size.0.max(line.width());
            }
        }

        // Push the final line
        lines.push(line);
        size.1 = properties.line_height * lines.len() as f32;

        // === Shift Lines & Glyphs === //
        for line in lines.iter() {
            let shift_x = match properties.alignment {
                Alignment::Start => 0.0,
                Alignment::Middle => (properties.max_size.0 - line.width()) / 2.0,
                Alignment::End => properties.max_size.0 - line.width(),
            };

            let start = line.glyph_index();
            let end = line.glyph_index() + line.total_glyphs();

            for index in start..end {
                let rect = &mut glyph_rects[index];
                rect.position.0 += shift_x;
            }
        }

        TextLayout::new(glyph_rects, lines, size, properties)
    }

    /// Attempts to find the next line break for a given set of [breakable words](BreakableWord).
    ///
    /// Each line break returned is guaranteed to be a _future_ index. That is, a line break will
    /// never occur before the given index. This ensures you can always prepare for a line break
    /// (e.g. remove extraneous trailing spaces) ahead of time.
    ///
    /// # Returns
    ///
    /// A tuple. The first field of the tuple indicates which word index to break _before_, if any.
    /// The second field indicates which word index to wait _until_ before calling this method again,
    /// if any. The reason for the second field is that there are cases where the line break behavior
    /// can be accounted for ahead of time.
    ///
    /// It's important that the skip index is used. Aside from it being inefficient, it may also result
    /// in unexpected behavior.
    ///
    /// # Arguments
    ///
    /// * `curr_index`: The current word index
    /// * `line_width`: The current line's current width
    /// * `properties`: The associated text properties
    /// * `words`: The list of breakable words
    ///
    fn find_next_break(
        &self,
        curr_index: usize,
        line_width: f32,
        properties: TextProperties,
        words: &[BreakableWord],
    ) -> (Option<usize>, Option<usize>) {
        // Line Break Rules:
        //
        // Break before Next if...
        // 1. Current is hard break.
        // 2. Next (end-trimmed) width > Max width.
        // 3. Next (end-trimmed) width + Current width > Max width.
        // 4. Next (end-trimmed) width + Current width + Line width > Max width.
        //
        // Break after Next if...
        // 5. Next is hard break.
        //
        // No break if...
        // 6. Next ends in whitespace.
        //
        // Collect joined Chain of words.
        //
        // No break if...
        // 7. Chain width + Current width + Line width <= Max width.
        //
        // Add Current width to Chain width if Current does not end in whitespace.
        //
        // Break before Next if...
        // 8. Chain width <= Max width.
        //
        // Otherwise...
        // 9. Break after Best point in Chain.

        let next_index = curr_index + 1;

        let curr = if let Some(curr) = words.get(curr_index) {
            curr
        } else {
            return (None, None);
        };

        // 1.
        if curr.hard_break {
            return (Some(next_index), None);
        }

        let next = if let Some(next) = words.get(next_index) {
            next
        } else {
            return (None, None);
        };

        let next_trimmed_width = self.get_word_width(next.content.trim_end(), properties);

        // 2.
        if next_trimmed_width > properties.max_size.0 {
            return (Some(next_index), None);
        }

        let curr_width = self.get_word_width(curr.content, properties);

        // 3.
        if next_trimmed_width + curr_width > properties.max_size.0 {
            return (Some(next_index), None);
        }

        // 4.
        if next_trimmed_width + curr_width + line_width > properties.max_size.0 {
            return (Some(next_index), None);
        }

        // 5.
        if next.hard_break {
            return (Some(next_index + 1), None);
        }

        // 6.
        if next.content.ends_with(char::is_whitespace) {
            return (None, None);
        }

        let mut peek_index = next_index;
        let mut chain_width = 0.0;
        let mut best_break_index = next_index;

        while let Some(peek) = words.get(peek_index) {
            chain_width += self.get_word_width(peek.content, properties);

            if peek.content.ends_with(char::is_whitespace) {
                // End of joined chain
                break;
            }

            if chain_width + curr_width + line_width < properties.max_size.0 {
                // Still within confines of line -> break line after here if needed
                best_break_index = peek_index + 1;
            }

            peek_index += 1;
        }

        // 7.
        if chain_width + curr_width + line_width <= properties.max_size.0 {
            return (None, Some(peek_index));
        }

        if !curr.content.ends_with(char::is_whitespace) {
            // Include the current word as part of the chain (if it is a part of it).
            // This is only for checking if the entire chain can fit on its own line.
            chain_width += curr_width;
        }

        // 8.
        if chain_width <= properties.max_size.0 {
            return (Some(next_index), Some(peek_index));
        }

        // 9.
        return (Some(best_break_index), Some(best_break_index));
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
        self.char_ids
            .get(&c)
            .and_then(|index| self.sdf.glyphs.get(*index as usize))
    }

    /// Calculates the appropriate glyph size for a desired font size.
    ///
    /// This glyph size can then be used to provide a normalized size across all glyphs
    /// in the atlas.
    fn calc_glyph_size(&self, font_size: f32) -> (f32, f32) {
        let font_scale = font_size / self.sdf.atlas.font_size;
        (
            self.max_glyph_size.0 * font_scale,
            self.max_glyph_size.1 * font_scale,
        )
    }
}
