#![allow(clippy::needless_question_mark, clippy::question_mark)]
use nanoserde::{DeJson, DeJsonErr, DeJsonState, SerJson, SerJsonState};

pub struct UnicodeChar(char);

impl DeJson for UnicodeChar {
    fn de_json(state: &mut DeJsonState, input: &mut std::str::Chars) -> Result<Self, DeJsonErr> {
        u32::de_json(state, input).and_then(|a| {
            if let Some(a) = char::from_u32(a) {
                Ok(Self(a))
            } else {
                Err(state.err_parse("Not unicode"))
            }
        })
    }
}

impl SerJson for UnicodeChar {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        let out = self.0 as u32;
        out.ser_json(d, s)
    }
}

impl From<&char> for UnicodeChar {
    fn from(c: &char) -> Self {
        Self(*c)
    }
}

impl From<&UnicodeChar> for char {
    fn from(uc: &UnicodeChar) -> Self {
        uc.0
    }
}

#[derive(DeJson, Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    #[nserde(proxy = "UnicodeChar")]
    pub unicode: char,
    pub advance: f32,
    #[nserde(rename = "atlasBounds")]
    pub atlas_bounds: Option<Rect>,
    #[nserde(rename = "planeBounds")]
    pub plane_bounds: Option<Rect>,
}

#[derive(DeJson, Default, Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }

    pub fn size(&self) -> (f32, f32) {
        (self.width(), self.height())
    }
}
