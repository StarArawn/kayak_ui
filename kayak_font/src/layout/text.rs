use bevy::reflect::Reflect;

use crate::{GlyphRect, Line, RowCol};
use std::cmp::Ordering;

/// The text alignment.
#[derive(Copy, Clone, Reflect, Debug, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Middle,
    End,
}

/// Properties to control text layout.
#[derive(Copy, Clone, Reflect, Debug, PartialEq)]
pub struct TextProperties {
    /// The font size (in pixels).
    pub font_size: f32,
    /// The line height (in pixels).
    pub line_height: f32,
    /// The maximum width and height a block of text can take up (in pixels).
    pub max_size: (f32, f32),
    /// The text alignment.
    pub alignment: Alignment,
    /// The size of a tab (`'\t'`) character in equivalent spaces.
    pub tab_size: u8,
}

impl Default for TextProperties {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            line_height: 14.0 * 1.2,
            max_size: (f32::MAX, f32::MAX),
            tab_size: 4,
            alignment: Alignment::Start,
        }
    }
}

/// Calculated text layout.
///
/// This can be retrieved using [`measure`](crate::KayakFont::measure).
#[derive(Clone, Reflect, Debug, Default, PartialEq)]
pub struct TextLayout {
    glyphs: Vec<GlyphRect>,
    lines: Vec<Line>,
    size: (f32, f32),
    properties: TextProperties,
}

impl TextLayout {
    /// Create a new [`TextLayout`].
    ///
    /// The given lists of [lines] and [glyphs] should be in their appropriate order
    /// (i.e. Line 1 should come before Line 2, etc.).
    ///
    /// [lines]: Line
    /// [glyphs]: GlyphRect
    pub fn new(
        glyphs: Vec<GlyphRect>,
        lines: Vec<Line>,
        size: (f32, f32),
        properties: TextProperties,
    ) -> Self {
        Self {
            glyphs,
            lines,
            size,
            properties,
        }
    }

    /// Returns the calculated lines for the text content.
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// Returns the calculated glyph rects for the text content.
    pub fn glyphs(&self) -> &[GlyphRect] {
        &self.glyphs
    }

    /// Returns the total width and height of the text content (in pixels).
    pub fn size(&self) -> (f32, f32) {
        self.size
    }

    /// Returns the properties used to calculate this layout.
    pub fn properties(&self) -> TextProperties {
        self.properties
    }

    /// The total number of lines.
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// The total number of graphemes.
    pub fn total_graphemes(&self) -> usize {
        self.lines
            .last()
            .map(|line| line.grapheme_index() + line.total_graphemes())
            .unwrap_or_default()
    }

    /// The total number of glyphs.
    pub fn total_glyphs(&self) -> usize {
        self.glyphs.len()
    }

    /// The total number of chars.
    pub fn total_chars(&self) -> usize {
        self.lines
            .last()
            .map(|line| line.char_index() + line.total_chars())
            .unwrap_or_default()
    }

    /// Performs a binary search to find the grapheme at the given index.
    ///
    /// If the grapheme could not be found, `None` is returned.
    pub fn find_grapheme(&self, index: usize) -> Option<RowCol> {
        self.lines
            .binary_search_by(|line| {
                if index < line.grapheme_index() {
                    // Current line comes after line containing grapheme
                    Ordering::Greater
                } else if index >= line.grapheme_index() + line.total_graphemes() {
                    // Current line comes before line containing grapheme
                    Ordering::Less
                } else {
                    // Current line contains grapheme
                    Ordering::Equal
                }
            })
            .map(|row| {
                let line = &self.lines[row];
                let col = index - line.grapheme_index();
                let grapheme = line[col];

                RowCol { row, col, grapheme }
            })
            .ok()
    }
}
