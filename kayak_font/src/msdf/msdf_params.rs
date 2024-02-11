#![allow(dead_code)]

pub struct MsdfParams {
    pub scale_x: f32,
    pub scale_y: f32,
    pub shape_scale: f32,
    pub min_image_width: usize,
    pub min_image_height: usize,
    pub angle_threshold: f64,
    pub px_range: f64,
    pub edge_threshold: f64,
    pub use_custom_image_size: bool,
    pub custom_width: usize,
    pub custom_height: usize,
    pub custom_border: usize,
}

impl MsdfParams {
    pub fn new() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            shape_scale: 1.0,
            min_image_width: 5,
            min_image_height: 5,
            angle_threshold: 3.0,
            px_range: 4.0,
            edge_threshold: 1.00000001,
            use_custom_image_size: false,
            custom_width: 0,
            custom_height: 0,
            custom_border: 0,
        }
    }
}
