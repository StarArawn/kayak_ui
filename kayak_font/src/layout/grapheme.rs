use std::cmp::Ordering;

use bevy::reflect::Reflect;

/// A representation of a grapheme cluster, as defined by [Unicode UAX #29].
///
/// [Unicode UAX #29]: https://unicode.org/reports/tr29/
#[derive(Default, Debug, Reflect, Copy, Clone, PartialEq)]
pub struct Grapheme {
    /// The index of the starting char within this grapheme, relative to the entire text content.
    pub char_index: usize,
    /// The total number of chars in this grapheme.
    pub char_total: usize,
    /// The index of the starting glyph within this grapheme, relative to the entire text content.
    pub glyph_index: usize,
    /// The total number of glyphs in this grapheme.
    pub glyph_total: usize,
    /// The position of this grapheme, relative to the entire text content.
    pub position: (f32, f32),
    /// The size of this grapheme.
    pub size: (f32, f32),
}

impl PartialOrd for Grapheme {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.char_index.partial_cmp(&other.char_index)
    }
}
