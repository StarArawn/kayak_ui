use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum SDFType {
    #[serde(alias = "msdf")]
    Msdf,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Origin {
    #[serde(alias = "bottom")]
    Bottom,
    #[serde(alias = "left")]
    Left,
    #[serde(alias = "right")]
    Right,
    #[serde(alias = "top")]
    Top,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Atlas {
    #[serde(alias = "type")]
    pub sdf_type: SDFType,
    #[serde(alias = "distanceRange")]
    pub distance_range: f32,
    #[serde(alias = "size")]
    pub font_size: f32,
    pub width: u32,
    pub height: u32,
    #[serde(alias = "yOrigin")]
    pub y_origin: Origin,
}
