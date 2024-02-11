use crate::msdf::{signed_distance::SignedDistance, vector::Vector2, EdgeColor};

use super::{
    equation_solver::{self, fabs},
    mix, non_zero_sign, EdgeSegment,
};

pub const MSDFGEN_CUBIC_SEARCH_STARTS: usize = 4;
pub const MSDFGEN_CUBIC_SEARCH_STEPS: usize = 4;

pub fn direction(p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, param: f64) -> Vector2 {
    let tangent = mix(
        mix(p1 - p0, p2 - p1, param),
        mix(p2 - p1, p3 - p2, param),
        param,
    );
    if !tangent.is_zero() {
        if param == 0.0 {
            return p2 - p0;
        }
        if param == 1.0 {
            return p3 - p1;
        }
    }
    tangent
}

pub fn point(p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, param: f64) -> Vector2 {
    let p12 = mix(p1, p2, param);
    mix(
        mix(mix(p0, p1, param), p12, param),
        mix(p12, mix(p2, p3, param), param),
        param,
    )
}

pub fn find_bounds(
    p0: Vector2,
    p1: Vector2,
    p2: Vector2,
    p3: Vector2,
    l: &mut f64,
    b: &mut f64,
    r: &mut f64,
    t: &mut f64,
) {
    Vector2::point_bounds(p0, l, b, r, t);
    Vector2::point_bounds(p3, l, b, r, t);

    let a0 = p1 - p0;
    let a1 = 2.0 * (p2 - p1 - a0);
    let a2 = p3 - 3.0 * p2 + 3.0 * p1 - p0;

    let (solutions, result) = equation_solver::solve_quadratic(a2.x, a1.x, a0.x);
    for i in 0..solutions {
        let par = result[i as usize];
        if par > 0.0 && par < 1.0 {
            Vector2::point_bounds(point(p0, p1, p2, p3, par), l, b, r, t);
        }
    }

    let (solutions, result) = equation_solver::solve_quadratic(a2.y, a1.y, a0.y);

    for i in 0..solutions {
        let par = result[i as usize];
        if par > 0.0 && par < 1.0 {
            Vector2::point_bounds(point(p0, p1, p2, p3, par), l, b, r, t);
        }
    }
}

pub fn split_in_thirds(
    p0: Vector2,
    p1: Vector2,
    p2: Vector2,
    p3: Vector2,
    color: EdgeColor,
) -> (EdgeSegment, EdgeSegment, EdgeSegment) {
    (
        EdgeSegment::new_cubic(
            p0,
            if p0 == p1 { p0 } else { mix(p0, p1, 1.0 / 3.0) },
            mix(mix(p0, p1, 1.0 / 3.0), mix(p1, p2, 1.0 / 3.0), 1.0 / 3.0),
            point(p0, p1, p2, p3, 1.0 / 3.0),
            color,
        ),
        EdgeSegment::new_cubic(
            point(p0, p1, p2, p3, 1.0 / 3.0),
            mix(
                mix(mix(p0, p1, 1.0 / 3.0), mix(p1, p2, 1.0 / 3.0), 1.0 / 3.0),
                mix(mix(p1, p2, 1.0 / 3.0), mix(p2, p3, 1.0 / 3.0), 1.0 / 3.0),
                2.0 / 3.0,
            ),
            mix(
                mix(mix(p0, p1, 2.0 / 3.0), mix(p1, p2, 2.0 / 3.0), 2.0 / 3.0),
                mix(mix(p1, p2, 2.0 / 3.0), mix(p2, p3, 2.0 / 3.0), 2.0 / 3.0),
                1.0 / 3.0,
            ),
            point(p0, p1, p2, p3, 2.0 / 3.0),
            color,
        ),
        EdgeSegment::new_cubic(
            point(p0, p1, p2, p3, 2.0 / 3.0),
            mix(mix(p1, p2, 2.0 / 3.0), mix(p2, p3, 2.0 / 3.0), 2.0 / 3.0),
            if p2 == p3 { p3 } else { mix(p2, p3, 2.0 / 3.0) },
            p3,
            color,
        ),
    )
}

pub fn signed_distance(
    p0: Vector2,
    p1: Vector2,
    p2: Vector2,
    p3: Vector2,
    origin: Vector2,
) -> (SignedDistance, f64) {
    let qa = p0 - origin;
    let ab = p1 - p0;
    let br = p2 - p1 - ab;
    let as_ = (p3 - p2) - (p2 - p1) - br;
    let mut ep_dir = direction(p0, p1, p2, p3, 0.0);

    let mut min_distance = non_zero_sign(Vector2::cross_product(ep_dir, qa)) as f64 * qa.length();
    let mut param = -Vector2::dot_product(qa, ep_dir) / Vector2::dot_product(ep_dir, ep_dir);
    {
        ep_dir = direction(p0, p1, p2, p3, 1.0);
        let distance = (p3 - origin).length();
        if distance.abs() < min_distance.abs() {
            min_distance =
                non_zero_sign(Vector2::cross_product(ep_dir, p3 - origin)) as f64 * distance;
            param = Vector2::dot_product(ep_dir - (p3 - origin), ep_dir)
                / Vector2::dot_product(ep_dir, ep_dir);
        }
    }

    for i in 0..MSDFGEN_CUBIC_SEARCH_STARTS {
        let mut t = (i / MSDFGEN_CUBIC_SEARCH_STARTS) as f64;
        let mut qe = qa + 3.0 * t * ab + 3.0 * t * t * br + t * t * t * as_;
        for _ in 0..MSDFGEN_CUBIC_SEARCH_STEPS {
            let d1 = 3.0 * ab + 6.0 * t * br + 3.0 * t * t * as_;
            let d2 = 6.0 * br + 6.0 * t * as_;
            t -= Vector2::dot_product(qe, d1)
                / (Vector2::dot_product(d1, d1) + Vector2::dot_product(qe, d2));

            if !(0.0..=1.0).contains(&t) {
                break;
            }

            qe = qa + 3.0 * t * ab + 3.0 * t * t * br + t * t * t * as_;
            let distance = qe.length();
            if distance < min_distance.abs() {
                min_distance = non_zero_sign(Vector2::cross_product(d1, qe)) as f64 * distance;
                param = t;
            }

            // let qpt = point(p0, p1, p2, p3, t) - origin;
            // let distance = non_zero_sign(Vector2::cross_product(direction(p0, p1, p2, p3, t), qpt))
            //     as f64
            //     * qpt.length();

            // if fabs(distance) < fabs(min_distance) {
            //     min_distance = distance;
            //     param = t;
            // }
            // if step == MSDFGEN_CUBIC_SEARCH_STEPS {
            //     break;
            // }
            // let d1 = 3.0 * as_ * t * t + 6.0 * br * t + 3.0 * ab;
            // let d2 = 6.0 * as_ * t + 6.0 * br;
            // t -= Vector2::dot_product(qpt, d1)
            //     / (Vector2::dot_product(d1, d1) + Vector2::dot_product(qpt, d2));
            // if t < 0.0 || t > 1.0 {
            //     break;
            // }
        }
    }

    if (0.0..=1.0).contains(&param) {
        (SignedDistance::new(min_distance, 0.0), param)
    } else if param < 0.5 {
        (
            SignedDistance::new(
                min_distance,
                fabs(Vector2::dot_product(
                    direction(p0, p1, p2, p3, 0.0),
                    qa.normalize(false),
                )),
            ),
            param,
        )
    } else {
        (
            SignedDistance::new(
                min_distance,
                fabs(Vector2::dot_product(
                    direction(p0, p1, p2, p3, 1.0).normalize(false),
                    (p3 - origin).normalize(false),
                )),
            ),
            param,
        )
    }
}
