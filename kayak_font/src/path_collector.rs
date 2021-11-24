use lyon_geom::math::{Angle, Point, Vector};
use lyon_path::{
    builder::{FlatPathBuilder, PathBuilder},
    Segment,
};

use crate::{color_flags::ColorFlags, contour::Contour, path_element::PathElement};

/// This is a path collector which produces our custom contour type.
pub struct PathCollector {
    /// The start point of the last contour
    pub(crate) contour_start: Point,
    /// The current pen location
    pub(crate) pen: Point,
    /// in-flight path elements
    pub(crate) elements: Vec<PathElement>,
    /// Completed contours
    pub(crate) contours: Vec<Contour>,
    pub scale: f32,
}

impl PathCollector {
    pub fn new() -> Self {
        Self {
            contour_start: Point::new(0.0, 0.0),
            pen: Point::new(0.0, 0.0),
            elements: Vec::new(),
            contours: Vec::new(),
            scale: 1.0,
        }
    }
}

impl PathBuilder for PathCollector {
    fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point) {
        self.elements.push(PathElement::new(
            Segment::Quadratic(lyon_geom::QuadraticBezierSegment {
                from: self.pen * self.scale,
                to: to * self.scale,
                ctrl: ctrl * self.scale,
            }),
            ColorFlags::WHITE,
        ));
        self.pen = to;
    }

    fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.elements.push(PathElement::new(
            Segment::Cubic(lyon_geom::CubicBezierSegment {
                from: self.pen * self.scale,
                to: to * self.scale,
                ctrl1: ctrl1 * self.scale,
                ctrl2: ctrl2 * self.scale,
            }),
            ColorFlags::WHITE,
        ));

        self.pen = to;
    }

    fn arc(&mut self, _center: Point, _radii: Vector, _sweep_angle: Angle, _x_rotation: Angle) {
        unimplemented!()
    }
}

impl FlatPathBuilder for PathCollector {
    type PathType = Vec<Contour>;

    fn move_to(&mut self, to: Point) {
        self.pen = to * self.scale;
        self.contour_start = to * self.scale;
    }

    fn line_to(&mut self, to: Point) {
        self.elements.push(PathElement::new(
            Segment::Line(lyon_geom::LineSegment {
                from: self.pen * self.scale,
                to: to * self.scale,
            }),
            ColorFlags::WHITE,
        ));
        self.pen = to * self.scale;
    }

    fn close(&mut self) {
        if (self.pen - self.contour_start).length() > 1E-14 {
            self.elements.push(PathElement::new(
                Segment::Line(lyon_geom::LineSegment {
                    from: self.pen * self.scale,
                    to: self.contour_start * self.scale,
                }),
                ColorFlags::WHITE,
            ));
        }

        self.pen = self.contour_start;
        let elements = std::mem::replace(&mut self.elements, Vec::new());

        self.contours.push(Contour { elements });
    }

    fn build(self) -> Self::PathType {
        let mut contours = self.contours;
        if self.elements.len() > 0 {
            let final_contour = Contour {
                elements: self.elements,
            };

            contours.push(final_contour);
        }

        contours
    }

    fn build_and_reset(&mut self) -> Self::PathType {
        let elements = std::mem::replace(&mut self.elements, Vec::new());
        if elements.len() > 0 {
            let final_contour = Contour { elements };

            self.contours.push(final_contour);
        }

        let tr = std::mem::replace(&mut self.contours, Vec::new());

        self.contour_start = Point::new(0.0, 0.0);
        self.pen = Point::new(0.0, 0.0);

        tr
    }

    fn current_position(&self) -> Point {
        self.pen
    }
}
