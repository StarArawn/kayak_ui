use nanoserde::DeJson;

#[derive(DeJson, Debug, Copy, Clone, PartialEq)]
pub enum SDFType {
    #[nserde(rename = "msdf")]
    Msdf,
}

#[derive(DeJson, Debug, Copy, Clone, PartialEq)]
pub enum Origin {
    #[nserde(rename = "bottom")]
    Bottom,
    #[nserde(rename = "left")]
    Left,
    #[nserde(rename = "right")]
    Right,
    #[nserde(rename = "top")]
    Top,
}

#[derive(DeJson, Debug, Copy, Clone, PartialEq)]
pub struct Atlas {
    #[nserde(rename = "type")]
    pub sdf_type: SDFType,
    #[nserde(rename = "distanceRange")]
    pub distance_range: f32,
    #[nserde(rename = "size")]
    pub font_size: f32,
    pub width: u32,
    pub height: u32,
    #[nserde(rename = "yOrigin")]
    pub y_origin: Origin,
}
