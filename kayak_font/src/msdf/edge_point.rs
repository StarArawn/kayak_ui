use crate::msdf::{edge_segment::EdgeSegment, signed_distance::SignedDistance};

#[derive(Debug, Clone, Copy)]
pub struct EdgePoint {
    pub min_distance: SignedDistance,
    pub near_edge: Option<EdgeSegment>,
    pub near_param: f64,
}

impl EdgePoint {
    // pub fn calculate_contour_color(&mut self, p: Vector2) -> f64 {
    //     if let Some(near_edge) = self.near_edge {
    //         near_edge.distance_to_pseudo_distance(&mut self.min_distance, p, self.near_param);
    //     }
    //     return self.min_distance.distance;
    // }
}
