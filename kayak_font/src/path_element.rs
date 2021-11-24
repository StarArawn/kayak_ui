use crate::{color_flags::ColorFlags, utils::AugmentedDistance};
use lyon_geom::math::{Point, Vector};
use lyon_path::Segment;

#[derive(Clone, Debug, Copy)]
pub struct PathElement {
    pub segment: Segment,
    pub color: ColorFlags,
}

impl PathElement {
    pub fn new(segment: Segment, color: ColorFlags) -> PathElement {
        Self { segment, color }
    }

    pub fn sample(&self, t: f32) -> Point {
        match self.segment {
            Segment::Line(s) => s.sample(t),
            Segment::Quadratic(s) => s.sample(t),
            Segment::Cubic(s) => s.sample(t),
            Segment::Arc(s) => s.sample(t),
        }
    }

    pub fn direction(&self, f: f32) -> Vector {
        use lyon_geom::Segment as SegmentTrait;
        let f = f.min(1.0).max(0.0);
        match self.segment {
            Segment::Line(s) => s.derivative(f),
            Segment::Quadratic(s) => s.derivative(f),
            Segment::Cubic(s) => s.derivative(f),
            Segment::Arc(s) => s.derivative(f),
        }
    }

    /// Split a path element into 3rds
    pub fn split_in_thirds(&self) -> [PathElement; 3] {
        macro_rules! segment_case {
            ($i:expr, $s:expr) => {{
                let (a, b) = ($i).split(1.0 / 3.0);
                let (b, c) = b.split(1.0 / 2.0);

                [a, b, c]
                    .into_iter()
                    .map(|x| PathElement::new(($s)(x), self.color))
                    .collect()
            }};
        }
        let segments: arrayvec::ArrayVec<PathElement, 3> = match self.segment {
            Segment::Line(s) => segment_case!(s, Segment::Line),
            Segment::Quadratic(s) => segment_case!(s, Segment::Quadratic),
            Segment::Cubic(s) => segment_case!(s, Segment::Cubic),
            Segment::Arc(s) => segment_case!(s, Segment::Arc),
        };

        segments
            .into_inner()
            .expect("We should have precisely the right capacity")
    }

    /// Computes the distance from p to this path element
    /// Returns the distance from the point to this path element,
    /// and the distance along this element to the closest point.
    pub fn distance(&self, p: Point) -> (AugmentedDistance, f32) {
        use lyon_geom::{LineSegment, QuadraticBezierSegment};
        match self.segment {
            Segment::Line(LineSegment { from: s, to: e }) => {
                let aq = p - s;
                let ab = e - s;
                let f = aq.dot(ab) / ab.dot(ab);
                let eq = if f >= 0.5 { p - e } else { p - s };

                let dist_to_endpoint = eq.length();
                let endpoint_sd = AugmentedDistance::new(
                    aq.cross(ab).signum() * dist_to_endpoint,
                    // ab.normalize().cross(eq.normalize()),
                    ab.normalize().dot(eq.normalize()).abs(),
                );

                if 0.0 < f && f < 1.0 {
                    let ortho = Vector::new(ab.y, -ab.x).normalize();
                    let ortho_dist = ortho.dot(aq);
                    if ortho_dist.abs() < endpoint_sd.distance.abs() {
                        (AugmentedDistance::new(ortho_dist, 0.0), f)
                    } else {
                        (endpoint_sd, f)
                    }
                } else {
                    (endpoint_sd, f)
                }
            }

            Segment::Quadratic(QuadraticBezierSegment {
                from: p0,
                ctrl: p1,
                to: p2,
            }) => {
                use lyon_geom::utils::cubic_polynomial_roots;
                let qa = p0 - p;
                let ab = p1 - p0;
                let br = (p0 - p1) + (p2 - p1);
                let a = br.dot(br);
                let b = 3.0 * ab.dot(br);
                let c = 2.0 * ab.dot(ab) + qa.dot(br);
                let d = qa.dot(ab);
                let solutions = cubic_polynomial_roots(a, b, c, d);

                let mut min_dist = ab.cross(qa).signum() * qa.length();

                let mut f = -qa.dot(ab) / ab.dot(ab);
                {
                    let ec = p2 - p1;
                    let ep = p2 - p;
                    let dist = ec.cross(ep).signum() * ep.length();
                    if dist.abs() < min_dist.abs() {
                        min_dist = dist;
                        f = (p - p1).dot(ec) / ec.dot(ec);
                    }
                }
                for t in solutions {
                    if t <= 0.0 || 1.0 <= t {
                        continue;
                    }
                    let endpoint = p0 + (ab * 2.0 * t) + (br * t * t);
                    let delta = endpoint - p;
                    let dist = (p2 - p0).cross(delta).signum() * delta.length();

                    if dist.abs() < min_dist.abs() {
                        min_dist = dist;
                        f = t;
                    }
                }

                if 0.0 <= f && f <= 1.0 {
                    (AugmentedDistance::new(min_dist, 0.0), f)
                // (AugmentedDistance::new(200f32, 0.0), f)
                } else if f < 0.5 {
                    (
                        AugmentedDistance::new(min_dist, ab.normalize().dot(qa.normalize()).abs()),
                        f,
                    )
                } else {
                    (
                        AugmentedDistance::new(
                            min_dist,
                            (p2 - p1).normalize().dot((p2 - p).normalize()).abs(),
                        ),
                        f,
                    )
                }
            }

            _ => unimplemented!(),
        }
    }

    pub(crate) fn to_psuedodistance(
        &self,
        dist: AugmentedDistance,
        p: Vector,
        near: f32,
    ) -> AugmentedDistance {
        if near <= 0.0 {
            let dir = self.direction(0.0).normalize();
            let aq = p - self.sample(0.0).to_vector();
            let ts = aq.dot(dir);
            if ts < 0.0 {
                let ds = aq.cross(dir);
                if ds.abs() <= dist.distance.abs() {
                    return AugmentedDistance::new(ds, 0.0);
                }
            }

            dist
        } else if near >= 1.0 {
            let dir = self.direction(1.0).normalize();
            let aq = p - self.sample(1.0).to_vector();
            let ts = aq.dot(dir);
            if ts > 0.0 {
                let ds = aq.cross(dir);
                if ds.abs() <= dist.distance.abs() {
                    return AugmentedDistance::new(ds, 0.0);
                }
            }

            dist
        } else {
            dist
        }
    }
}
