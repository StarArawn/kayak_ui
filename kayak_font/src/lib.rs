#![allow(dead_code)]
mod color_flags;
mod contour;
mod font;
mod msdf;
mod path_collector;
mod path_element;
mod recolor;
mod sdf;
mod ttf_parser;
mod utils;

pub use color_flags::ColorFlags;
pub use contour::{rescale_contours, Contour};
pub use font::{Font, FontCache};
pub use lyon_geom::math::{Angle, Point, Rect, Vector};
pub use lyon_path::builder::FlatPathBuilder;
pub use msdf::compute_msdf;
pub use path_collector::PathCollector;
pub use path_element::PathElement;
pub use recolor::recolor_contours;
pub use sdf::compute_sdf;

pub(crate) fn median(a: f32, b: f32, c: f32) -> f32 {
    let min = |a: f32, b: f32| a.min(b);
    let max = |a: f32, b: f32| a.max(b);
    max(min(a, b), min(max(a, b), c))
}
