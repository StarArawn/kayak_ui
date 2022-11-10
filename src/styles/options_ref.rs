use crate::styles::KStyle;

/// A trait used to allow reading a value as an `Option<&T>`
pub trait AsRefOption<T> {
    fn as_ref_option(&self) -> Option<&T>;
}

impl AsRefOption<KStyle> for KStyle {
    fn as_ref_option(&self) -> Option<&KStyle> {
        Some(self)
    }
}

impl AsRefOption<KStyle> for &KStyle {
    fn as_ref_option(&self) -> Option<&KStyle> {
        Some(self)
    }
}

impl AsRefOption<KStyle> for Option<KStyle> {
    fn as_ref_option(&self) -> Option<&KStyle> {
        self.as_ref()
    }
}

impl AsRefOption<KStyle> for &Option<KStyle> {
    fn as_ref_option(&self) -> Option<&KStyle> {
        self.as_ref()
    }
}
