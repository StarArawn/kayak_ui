use num_derive::FromPrimitive;

pub mod bitmap;
pub mod contour;
pub mod edge_coloring;
pub mod edge_point;
pub mod edge_segment;
pub mod gen;
pub mod msdf_params;
pub mod shape;
pub mod signed_distance;
pub mod ttf_parser;
pub mod vector;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
pub enum EdgeColor {
    BLACK = 0,
    RED = 1,
    GREEN = 2,
    YELLOW = 3,
    BLUE = 4,
    MAGENTA = 5,
    CYAN = 6,
    WHITE = 7,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MultiDistance {
    pub b: f64,
    pub g: f64,
    pub med: f64,
    pub r: f64,
}
