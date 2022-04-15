use std::cmp::Ordering;

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
    pub max_size: Option<(f32, f32)>,
    /// The text alignment.
    pub alignment: Alignment,
}

/// Contains details for a calculated line of text.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Line {
    /// The index of the starting grapheme cluster within the text content.
    pub index: usize,
    /// The total number of grapheme clusters in this line.
    pub len: usize,
    /// The total width of this line (in pixels).
    pub width: f32,
}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

/// Calculated text layout.
///
/// This can be retrieved using [`measure`](crate::KayakFont::measure).
#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    lines: Vec<Line>,
    size: (f32, f32),
    properties: TextProperties,
}

impl TextLayout {
    /// Create a new [`TextLayout`].
    pub fn new(lines: Vec<Line>, size: (f32, f32), properties: TextProperties) -> Self {
        Self {
            lines,
            size,
            properties,
        }
    }

    /// Returns the calculated lines for the text content.
    pub fn lines(&self) -> &Vec<Line> {
        &self.lines
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
