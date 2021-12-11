use bevy::math::Vec2;
use serde::{Deserialize, Deserializer};

fn from_u32<'de, D>(deserializer: D) -> Result<char, D::Error>
where
    D: Deserializer<'de>,
{
    let number: u32 = Deserialize::deserialize(deserializer)?;
    match char::from_u32(number) {
        Some(c) => Ok(c),
        None => Err(serde::de::Error::custom("Can't deserialize char from u32!")),
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Glyph {
    #[serde(deserialize_with = "from_u32")]
    pub unicode: char,
    pub advance: f32,
    #[serde(alias = "atlasBounds")]
    pub atlas_bounds: Option<Rect>,
    #[serde(alias = "planeBounds")]
    pub plane_bounds: Option<Rect>,
}

#[derive(Deserialize, Default, Clone, Copy, Debug)]
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

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }
}
