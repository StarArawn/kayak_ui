use crate::styles::Style;

/// A trait used to allow reading a value as an `Option<&T>`
pub trait AsRefOption<T> {
    fn as_ref_option(&self) -> Option<&T>;
}

impl AsRefOption<Style> for Style {
    fn as_ref_option(&self) -> Option<&Style> {
        Some(&self)
    }
}

impl AsRefOption<Style> for &Style {
    fn as_ref_option(&self) -> Option<&Style> {
        Some(self)
    }
}

impl AsRefOption<Style> for Option<Style> {
    fn as_ref_option(&self) -> Option<&Style> {
        self.as_ref()
    }
}

impl AsRefOption<Style> for &Option<Style> {
    fn as_ref_option(&self) -> Option<&Style> {
        self.as_ref()
    }
}