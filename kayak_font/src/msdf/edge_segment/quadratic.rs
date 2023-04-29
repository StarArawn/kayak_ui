use crate::msdf::{signed_distance::SignedDistance, vector::Vector2, EdgeColor};

use super::{equation_solver, mix, non_zero_sign, EdgeSegment};

pub fn direction(p0: Vector2, p1: Vector2, p2: Vector2, param: f64) -> Vector2 {
    mix(p1 - p0, p2 - p1, param)
}

pub fn point(p0: Vector2, p1: Vector2, p2: Vector2, param: f64) -> Vector2 {
    mix(mix(p0, p1, param), mix(p1, p2, param), param)
}

pub fn find_bounds(
    p0: Vector2,
    p1: Vector2,
    p2: Vector2,
    l: &mut f64,
    b: &mut f64,
    r: &mut f64,
    t: &mut f64,
) {
    Vector2::point_bounds(p0, l, b, r, t);
    Vector2::point_bounds(p2, l, b, r, t);
    let bot = (p1 - p0) - (p2 - p1);
    if bot.x != 0.0 {
        let param = (p1.x - p0.x) / bot.x;
        if param > 0.0 && param < 1.0 {
            Vector2::point_bounds(point(p0, p1, p2, param), l, b, r, t);
        }
    }
    if bot.y != 0.0 {
        let param = (p1.y - p0.y) / bot.y;
        if param > 0.0 && param < 1.0 {
            Vector2::point_bounds(point(p0, p1, p2, param), l, b, r, t);
        }
    }
}

pub fn split_in_thirds(
    p0: Vector2,
    p1: Vector2,
    p2: Vector2,
    color: EdgeColor,
) -> (EdgeSegment, EdgeSegment, EdgeSegment) {
    (
        EdgeSegment::new_quadratic(
            p0,
            mix(p0, p1, 1.0 / 3.0),
            point(p0, p1, p2, 1.0 / 3.0),
            color,
        ),
        EdgeSegment::new_quadratic(
            point(p0, p1, p2, 1.0 / 3.0),
            mix(mix(p0, p1, 5.0 / 9.0), mix(p1, p2, 4.0 / 9.0), 0.5),
            point(p0, p1, p2, 2.0 / 3.0),
            color,
        ),
        EdgeSegment::new_quadratic(
            point(p0, p1, p2, 2.0 / 3.0),
            mix(p1, p2, 2.0 / 3.0),
            p2,
            color,
        ),
    )
}

pub fn signed_distance(
    p0: Vector2,
    p1: Vector2,
    p2: Vector2,
    origin: Vector2,
) -> (SignedDistance, f64) {
    let qa = p0 - origin;
    let ab = p1 - p0;
    // let br = p0 + p2 - p1 - p1;
    let br = p2 - p1 - ab;
    let a = Vector2::dot_product(br, br);
    let b = 3.0 * Vector2::dot_product(ab, br);
    let c = 2.0 * Vector2::dot_product(ab, ab) + Vector2::dot_product(qa, br);
    let d = Vector2::dot_product(qa, ab);

    let (solutions, t) = equation_solver::solve_cubic(a, b, c, d);

    let mut min_distance = non_zero_sign(Vector2::cross_product(ab, qa)) as f64 * qa.length();
    let mut param = -Vector2::dot_product(qa, ab) / Vector2::dot_product(ab, ab);
    {
        let distance = non_zero_sign(Vector2::cross_product(p2 - p1, p2 - origin)) as f64
            * (p2 - origin).length();
        if distance.abs() < min_distance.abs() {
            min_distance = distance;
            param =
                Vector2::dot_product(origin - p1, p2 - p1) / Vector2::dot_product(p2 - p1, p2 - p1);
        }
    }

    for i in 0..solutions {
        let ti = t[i as usize];

        if ti > 0.0 && ti < 1.0 {
            let endpoint = p0 + 2.0 * ti * ab + ti * ti * br;
            let distance = non_zero_sign(Vector2::cross_product(p2 - p0, endpoint - origin)) as f64
                * (endpoint - origin).length();
            if distance.abs() <= min_distance.abs() {
                min_distance = distance;
                param = ti;
            }
        }
    }

    if (0.0..=1.0).contains(&param) {
        (SignedDistance::new(min_distance, 0.0), param)
    } else if param < 0.5 {
        (
            SignedDistance::new(
                min_distance,
                (Vector2::dot_product(ab.normalize(false), qa.normalize(false))).abs(),
            ),
            param,
        )
    } else {
        (
            SignedDistance::new(
                min_distance,
                (Vector2::dot_product((p2 - p1).normalize(false), (p2 - origin).normalize(false)))
                    .abs(),
            ),
            param,
        )
    }
}
