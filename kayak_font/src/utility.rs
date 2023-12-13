use xi_unicode::LineBreakIterator;

pub const NEWLINE: char = '\n';
pub const CARRIAGE: char = '\r';
pub const SPACE: char = ' ';
pub const NBSP: char = '\u{a0}';
pub const TAB: char = '\t';
pub const MISSING: char = '�';

/// Returns true if the given character is a newline.
pub fn is_newline(c: char) -> bool {
    c == NEWLINE || c == CARRIAGE
}

/// Returns true if the given character is a space.
///
/// Includes the non-breaking space ([`NBSP`]).
pub fn is_space(c: char) -> bool {
    c == SPACE || c == NBSP
}

/// Returns true if the given character is a tab.
pub fn is_tab(c: char) -> bool {
    c == TAB
}

/// Split a string into a collection of "words" that may be followed by a line break,
/// according to [UAX #14](https://www.unicode.org/reports/tr14/).
///
/// For example, `"Hello, world!"` would be broken into `["Hello, ", "world!"]`. And
/// `"A-rather-long-word"` would be broken into `["A-", "rather-", "long-", "word"]`.
pub fn split_breakable_words(text: &str) -> BreakableWordIter {
    BreakableWordIter::new(text)
}

/// A "word" (or, rather substring) that may be followed by a line break,
/// according to [UAX #14](https://www.unicode.org/reports/tr14/).
#[derive(Copy, Clone, Debug)]
pub struct BreakableWord<'a> {
    /// The index of the last character in this word.
    pub char_index: usize,
    /// The content of this word.
    pub content: &'a str,
    /// If true, this word __must__ be followed by a line break.
    pub hard_break: bool,
}

/// An iterator over [`BreakableWord`].
#[derive(Copy, Clone)]
pub struct BreakableWordIter<'a> {
    text: &'a str,
    iter: LineBreakIterator<'a>,
    index: usize,
}

impl<'a> BreakableWordIter<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            iter: LineBreakIterator::new(text),
            index: 0,
        }
    }
}

impl<'a> Iterator for BreakableWordIter<'a> {
    type Item = BreakableWord<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (next_idx, is_hard) = self.iter.next()?;
        let word = self.text.get(self.index..next_idx)?;
        self.index = next_idx;

        Some(BreakableWord {
            char_index: next_idx,
            content: word,
            hard_break: is_hard,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
