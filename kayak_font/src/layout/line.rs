use bevy::reflect::Reflect;

use crate::layout::grapheme::Grapheme;
use std::cmp::Ordering;
use std::ops::Index;
use std::slice::SliceIndex;

/// Contains details for a calculated line of text.
#[derive(Clone, Reflect, Debug, PartialEq)]
pub struct Line {
    grapheme_index: usize,
    graphemes: Vec<Grapheme>,
    width: f32,
}

/// A reference to the grapheme at a specific row and column of a given line of text.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RowCol {
    /// The row this line belongs to (zero-indexed).
    pub row: usize,
    /// The column this grapheme belongs to (zero-indexed).
    ///
    /// This is the same as the grapheme index localized to within a line.
    pub col: usize,
    /// The grapheme at this row and column.
    pub grapheme: Grapheme,
}

impl Line {
    /// Creates a new [`Line`] starting at the given grapheme cluster index.
    pub fn new(grapheme_index: usize) -> Self {
        Self {
            grapheme_index,
            graphemes: Vec::new(),
            width: 0.0,
        }
    }

    /// Creates a new [`Line`] following the given one.
    ///
    /// This essentially means that it starts out pointing to the next [grapheme index].
    ///
    /// [grapheme index]: Self::grapheme_index
    pub fn new_after(previous: &Self) -> Self {
        Self::new(previous.grapheme_index + previous.total_graphemes())
    }

    /// The total width of this line (in pixels).
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Returns the grapheme at the given index within this line, if any.
    ///
    /// If the grapheme does
    pub fn get_grapheme<I: SliceIndex<[Grapheme]>>(&self, index: I) -> Option<&I::Output> {
        self.graphemes.get(index)
    }

    /// Returns the grapheme at the given index within this line.
    ///
    /// # Panics
    ///
    /// Will panic if the given index is out of bounds of this line.
    pub fn grapheme<I: SliceIndex<[Grapheme]>>(&self, index: I) -> &I::Output {
        &self.graphemes[index]
    }

    /// The list of grapheme clusters in this line.
    pub fn graphemes(&self) -> &[Grapheme] {
        &self.graphemes
    }

    /// The index of the starting grapheme cluster within this line, relative to the entire text content.
    pub fn grapheme_index(&self) -> usize {
        self.grapheme_index
    }

    /// The total number of graphemes in this line.
    pub fn total_graphemes(&self) -> usize {
        self.graphemes.len()
    }

    /// The index of the starting glyph within this line, relative to the entire text content.
    pub fn glyph_index(&self) -> usize {
        self.graphemes
            .first()
            .map(|grapheme| grapheme.glyph_index)
            .unwrap_or_default()
    }

    /// The total number of glyphs in this line.
    pub fn total_glyphs(&self) -> usize {
        let end = self
            .graphemes
            .last()
            .map(|grapheme| grapheme.glyph_index + grapheme.glyph_total);

        match end {
            Some(index) if index > 0 => index - self.glyph_index(),
            _ => 0,
        }
    }

    /// The index of the starting char within this line, relative to the entire text content.
    pub fn char_index(&self) -> usize {
        self.graphemes
            .first()
            .map(|grapheme| grapheme.char_index)
            .unwrap_or_default()
    }

    /// The total number of chars in this line.
    pub fn total_chars(&self) -> usize {
        let end = self
            .graphemes
            .last()
            .map(|grapheme| grapheme.char_index + grapheme.char_total);

        match end {
            Some(index) if index > 0 => index - self.char_index(),
            _ => 0,
        }
    }

    /// Add a new grapheme to this line.
    pub fn add_grapheme(&mut self, grapheme: Grapheme) {
        self.width += grapheme.size.0;
        self.graphemes.push(grapheme)
    }
}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.grapheme_index.partial_cmp(&other.grapheme_index)
    }
}

impl<I: SliceIndex<[Grapheme]>> Index<I> for Line {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.grapheme(index)
    }
}
