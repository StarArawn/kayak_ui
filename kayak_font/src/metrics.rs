use nanoserde::DeJson;

#[derive(DeJson, Debug, Copy, Clone, PartialEq)]
pub struct Metrics {
    #[nserde(rename = "emSize")]
    em_size: f32,
    #[nserde(rename = "lineHeight")]
    line_height: f32,
    ascender: f32,
    descender: f32,
    #[nserde(rename = "underlineY")]
    underline_y: f32,
    #[nserde(rename = "underlineThickness")]
    underline_thickness: f32,
}
