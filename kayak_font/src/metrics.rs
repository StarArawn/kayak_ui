use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct Metrics {
    #[serde(alias = "emSize")]
    em_size: f32,
    #[serde(alias = "lineHeight")]
    line_height: f32,
    ascender: f32,
    descender: f32,
    #[serde(alias = "underlineY")]
    underline_y: f32,
    #[serde(alias = "underlineThickness")]
    underline_thickness: f32,
}
