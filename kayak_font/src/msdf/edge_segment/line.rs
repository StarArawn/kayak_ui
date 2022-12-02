use crate::msdf::{signed_distance::SignedDistance, vector::Vector2, EdgeColor};

use super::{mix, non_zero_sign, EdgeSegment};

pub fn direction(p0: Vector2, p1: Vector2, _param: f64) -> Vector2 {
    p1 - p0
}

pub fn point(p0: Vector2, p1: Vector2, param: f64) -> Vector2 {
    mix(p0, p1, param)
}

pub fn find_bounds(p0: Vector2, p1: Vector2, l: &mut f64, b: &mut f64, r: &mut f64, t: &mut f64) {
    Vector2::point_bounds(p0, l, b, r, t);
    Vector2::point_bounds(p1, l, b, r, t);
}

pub fn split_in_thirds(
    p0: Vector2,
    p1: Vector2,
    color: EdgeColor,
) -> (EdgeSegment, EdgeSegment, EdgeSegment) {
    (
        EdgeSegment::new_linear(p0, point(p0, p1, 1.0 / 3.0), color),
        EdgeSegment::new_linear(point(p0, p1, 1.0 / 3.0), point(p0, p1, 2.0 / 3.0), color),
        EdgeSegment::new_linear(point(p0, p1, 2.0 / 3.0), p1, color),
    )
}

pub fn signed_distance(p0: Vector2, p1: Vector2, origin: Vector2) -> (SignedDistance, f64) {
    let aq = origin - p0;
    let ab = p1 - p0;
    let param = Vector2::dot_product(aq, ab) / Vector2::dot_product(ab, ab);

    let eq = (if param > 0.5 { p1 } else { p0 }) - origin;
    let endpoint_distance = eq.length();
    if param > 0.0 && param < 1.0 {
        let ortho_distance = Vector2::dot_product(ab.get_ortho_normal(false, false), aq);
        if ortho_distance.abs() < endpoint_distance {
            return (SignedDistance::new(ortho_distance, 0.0), param);
        }
    }
    return (
        SignedDistance::new(
            non_zero_sign(Vector2::cross_product(aq, ab)) as f64 * endpoint_distance,
            Vector2::dot_product(ab.normalize(false), eq.normalize(false)).abs(),
        ),
        param,
    );
}
