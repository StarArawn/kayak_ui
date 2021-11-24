use lyon_geom::math::{Point, Rect};
use lyon_path::Segment;

use crate::path_element::PathElement;

/// A list of path elements forming a closed loop
#[derive(Clone, Debug)]
pub struct Contour {
    pub elements: Vec<PathElement>,
}

impl Contour {
    pub fn winding(&self) -> f32 {
        let shoelace = |a: Point, b: Point| (b.x - a.x) * (a.y + b.y);
        let n = self.elements.len();
        match n {
            0 => 0.0,
            1 => {
                let a = self.elements[0].sample(0.0);
                let b = self.elements[0].sample(1.0 / 3.0);
                let c = self.elements[0].sample(2.0 / 3.0);

                shoelace(a, b) + shoelace(b, c) + shoelace(c, a)
            }
            2 => {
                let a = self.elements[0].sample(0.0);
                let b = self.elements[0].sample(0.5);
                let c = self.elements[1].sample(0.0);
                let d = self.elements[1].sample(0.5);

                shoelace(a, b) + shoelace(b, c) + shoelace(c, d) + shoelace(d, a)
            }
            _ => {
                let mut total = 0.0;
                let mut prev = self.elements[n - 1].sample(0.0);

                for e in &self.elements {
                    let curr = e.sample(0.0);
                    total += shoelace(prev, curr);
                    prev = curr;
                }

                total
            }
        }
        .signum()
    }
}

/// Rescale contours so they fit in the provided rectangle.
/// Returns the scaled contours along with the transformation used to rescale the contours
pub fn rescale_contours(
    mut contours: Vec<Contour>,
    initial_bounds: Rect,
    bounds: Rect,
    _units_per_em: u16,
) -> (Vec<Contour>, lyon_geom::math::Transform2D) {
    // let (new_width, new_height) = if initial_bounds.size.width > initial_bounds.size.height {
    //     let new_width = 1.0;
    //     let new_height = aspect_ratio_height(
    //         initial_bounds.size.height,
    //         initial_bounds.size.width,
    //         new_width,
    //     );

    //     (new_width, new_height)
    // } else {
    //     let new_height = 1.0;
    //     let new_width = aspect_ratio_width(
    //         initial_bounds.size.height,
    //         initial_bounds.size.width,
    //         new_height,
    //     );

    //     (new_width, new_height)
    // };

    // dbg!(new_width, new_height);

    // let x_scale = new_width / initial_bounds.size.width;
    // let y_scale = new_height / initial_bounds.size.height;

    let initial_scale = initial_bounds.size.height.max(initial_bounds.size.width);
    let bounds_scale = bounds.size.width.max(bounds.size.height);

    // let size = 128.0 / units_per_em as f32;
    let transformation = lyon_geom::math::Transform2D::create_translation(
        -initial_bounds.origin.x as f32,
        -initial_bounds.origin.y as f32,
    )
    .post_scale(bounds_scale / initial_scale, bounds_scale / initial_scale)
    .post_translate(bounds.origin.to_vector());
    for contour in &mut contours {
        for mut elem in &mut contour.elements {
            elem.segment = match elem.segment {
                Segment::Line(s) => Segment::Line(s.transform(&transformation)),
                Segment::Quadratic(s) => Segment::Quadratic(s.transform(&transformation)),
                Segment::Cubic(s) => Segment::Cubic(s.transform(&transformation)),
                Segment::Arc(s) => Segment::Arc(lyon_geom::Arc {
                    center: transformation.transform_point(&s.center),
                    ..s
                }),
            }
        }
    }

    (contours, transformation)
}
