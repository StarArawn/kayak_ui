use std::cmp::Ordering;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GlyphRect {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub content: char,
}

/// The text alignment.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Alignment {
    Start,
    Middle,
    End,
}

/// Properties to control text layout.
#[derive(Copy, Clone, Debug, PartialEq)]
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

/// Contains details for a calculated line of text.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Line {
    /// The index of the starting grapheme cluster within the text content.
    pub grapheme_index: usize,
    /// The index of the starting glyph within the text content.
    pub glyph_index: usize,
    /// The index of the starting char within the text content.
    pub char_index: usize,
    /// The total number of grapheme clusters in this line.
    pub grapheme_len: usize,
    /// The total number of glyphs in this line.
    pub glyph_len: usize,
    /// The total number of chars in this line.
    pub char_len: usize,
    /// The total width of this line (in pixels).
    pub width: f32,
}

impl Line {
    /// Creates a new [`Line`] following the given one.
    pub fn new_after(previous: Self) -> Self {
        Self {
            grapheme_index: previous.grapheme_index + previous.grapheme_len,
            glyph_index: previous.glyph_index + previous.glyph_len,
            char_index: previous.char_index + previous.char_len,
            ..Default::default()
        }
    }
}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.grapheme_index.partial_cmp(&other.grapheme_index)
    }
}

/// Calculated text layout.
///
/// This can be retrieved using [`measure`](crate::KayakFont::measure).
#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    glyphs: Vec<GlyphRect>,
    lines: Vec<Line>,
    size: (f32, f32),
    properties: TextProperties,
}

impl TextLayout {
    /// Create a new [`TextLayout`].
    pub fn new(glyphs: Vec<GlyphRect>, lines: Vec<Line>, size: (f32, f32), properties: TextProperties) -> Self {
        Self { glyphs, lines, size, properties }
    }

    /// Returns the calculated lines for the text content.
    pub fn lines(&self) -> &Vec<Line> {
        &self.lines
    }

    /// Returns the calculated glyph rects for the text content.
    pub fn glyphs(&self) -> &Vec<GlyphRect> {
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
}
