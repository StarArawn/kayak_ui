use crate::msdf::{signed_distance::SignedDistance, vector::Vector2, EdgeColor};

mod cubic;
mod equation_solver;
mod line;
mod quadratic;

pub fn non_zero_sign(n: f64) -> i32 {
    return 2 * (if n > 0.0 { 1 } else { 0 }) - 1;
}
pub fn mix(a: Vector2, b: Vector2, weight: f64) -> Vector2 {
    Vector2::new(
        (1.0 - weight) * a.x + (weight * b.x),
        (1.0 - weight) * a.y + (weight * b.y),
    )
}

#[derive(Debug, Clone, Copy)]
pub enum EdgeSegment {
    LineSegment {
        color: EdgeColor,
        p0: Vector2,
        p1: Vector2,
    },
    QuadraticSegment {
        color: EdgeColor,
        p0: Vector2,
        p1: Vector2,
        p2: Vector2,
    },
    CubicSegment {
        color: EdgeColor,
        p0: Vector2,
        p1: Vector2,
        p2: Vector2,
        p3: Vector2,
    },
}

impl Default for EdgeSegment {
    fn default() -> Self {
        EdgeSegment::LineSegment {
            color: EdgeColor::WHITE,
            p0: Vector2::default(),
            p1: Vector2::default(),
        }
    }
}

impl EdgeSegment {
    pub fn new_linear(p0: Vector2, p1: Vector2, color: EdgeColor) -> Self {
        Self::LineSegment { p0, p1, color }
    }

    pub fn new_quadratic(p0: Vector2, mut p1: Vector2, p2: Vector2, color: EdgeColor) -> Self {
        if p1 == p0 || p1 == p2 {
            p1 = 0.5*(p0+p2);
        }
        Self::QuadraticSegment { p0, p1, p2, color }
    }

    pub fn new_cubic(p0: Vector2, mut p1: Vector2, mut p2: Vector2, p3: Vector2, color: EdgeColor) -> Self {
        if (p1 == p0 || p1 == p3) && (p2 == p0 || p2 == p3) {
            p1 = mix(p0, p3, 1.0 / 3.0);
            p2 = mix(p0, p3, 2.0 / 3.0);
        }
        Self::CubicSegment {
            p0,
            p1,
            p2,
            p3,
            color,
        }
    }

    pub fn distance_to_pseudo_distance(
        &self,
        distance: &mut SignedDistance,
        origin: Vector2,
        param: f64,
    ) {
        if param < 0.0 {
            let dir = self.direction(0.0).normalize(false);
            let aq = origin - self.point(0.0);
            let ts = Vector2::dot_product(aq, dir);
            if ts < 0.0 {
                let pseudo_distance = Vector2::cross_product(aq, dir);
                if pseudo_distance.abs() <= distance.distance.abs() {
                    *distance = SignedDistance::new(pseudo_distance, 0.0);
                }
            }
        } else if param > 1.0 {
            let dir = self.direction(1.0).normalize(false);
            let bq = origin - self.point(1.0);
            let ts = Vector2::dot_product(bq, dir);
            if ts > 0.0 {
                let pseudo_distance = Vector2::cross_product(bq, dir);
                if pseudo_distance.abs() <= distance.distance.abs() {
                    *distance = SignedDistance::new(pseudo_distance, 0.0);
                }
            }
        }
    }

    pub fn direction(&self, param: f64) -> Vector2 {
        match *self {
            Self::LineSegment { p0, p1, .. } => line::direction(p0, p1, param),
            Self::QuadraticSegment { p0, p1, p2, .. } => quadratic::direction(p0, p1, p2, param),
            Self::CubicSegment { p0, p1, p2, p3, .. } => cubic::direction(p0, p1, p2, p3, param),
        }
    }

    pub fn point(&self, param: f64) -> Vector2 {
        match *self {
            Self::LineSegment { p0, p1, .. } => line::point(p0, p1, param),
            Self::QuadraticSegment { p0, p1, p2, .. } => quadratic::point(p0, p1, p2, param),
            Self::CubicSegment { p0, p1, p2, p3, .. } => cubic::point(p0, p1, p2, p3, param),
        }
    }

    pub fn find_bounds(&self, l: &mut f64, b: &mut f64, r: &mut f64, t: &mut f64) {
        match *self {
            Self::LineSegment { p0, p1, .. } => line::find_bounds(p0, p1, l, b, r, t),
            Self::QuadraticSegment { p0, p1, p2, .. } => {
                quadratic::find_bounds(p0, p1, p2, l, b, r, t)
            }
            Self::CubicSegment { p0, p1, p2, p3, .. } => {
                cubic::find_bounds(p0, p1, p2, p3, l, b, r, t)
            }
        }
    }

    pub fn split_in_thirds(&self) -> (EdgeSegment, EdgeSegment, EdgeSegment) {
        match *self {
            Self::LineSegment { p0, p1, color } => line::split_in_thirds(p0, p1, color),
            Self::QuadraticSegment { p0, p1, p2, color } => {
                quadratic::split_in_thirds(p0, p1, p2, color)
            }
            Self::CubicSegment {
                p0,
                p1,
                p2,
                p3,
                color,
            } => cubic::split_in_thirds(p0, p1, p2, p3, color),
        }
    }

    pub fn signed_distance(&self, origin: Vector2) -> (SignedDistance, f64) {
        match *self {
            Self::LineSegment { p0, p1, .. } => line::signed_distance(p0, p1, origin),
            Self::QuadraticSegment { p0, p1, p2, .. } => {
                quadratic::signed_distance(p0, p1, p2, origin)
            }
            Self::CubicSegment { p0, p1, p2, p3, .. } => {
                cubic::signed_distance(p0, p1, p2, p3, origin)
            }
        }
    }

    pub fn has_color(&self, c: EdgeColor) -> bool {
        match *self {
            Self::LineSegment { color, .. } => color as usize & c as usize != 0,
            Self::QuadraticSegment { color, .. } => color as usize & c as usize != 0,
            Self::CubicSegment { color, .. } => color as usize & c as usize != 0,
        }
    }

    pub fn get_color(&self) -> EdgeColor {
        match self {
            Self::LineSegment { color, .. } => *color,
            Self::QuadraticSegment { color, .. } => *color,
            Self::CubicSegment { color, .. } => *color,
        }
    }

    pub fn set_color(&mut self, c: EdgeColor) {
        match self {
            Self::LineSegment { color, .. } => *color = c,
            Self::QuadraticSegment { color, .. } => *color = c,
            Self::CubicSegment { color, .. } => *color = c,
        }
    }
}
