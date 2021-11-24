use lyon_geom::math::Point;
use lyon_path::Segment;

use crate::{ColorFlags, Contour, PathCollector, PathElement};

impl ttf_parser::OutlineBuilder for PathCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = Point::new(x, y);
        self.pen = to;
        self.contour_start = to;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = Point::new(x, y) * self.scale;
        self.elements.push(PathElement::new(
            Segment::Line(lyon_geom::LineSegment { from: self.pen, to }),
            ColorFlags::WHITE,
        ));
        self.pen = to;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let ctrl = Point::new(x1, y1) * self.scale;
        let to = Point::new(x, y) * self.scale;
        self.elements.push(PathElement::new(
            Segment::Quadratic(lyon_geom::QuadraticBezierSegment {
                from: self.pen,
                to,
                ctrl,
            }),
            ColorFlags::WHITE,
        ));
        self.pen = to;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let ctrl1 = Point::new(x1, y1) * self.scale;
        let ctrl2 = Point::new(x2, y2) * self.scale;
        let to = Point::new(x, y) * self.scale;
        self.elements.push(PathElement::new(
            Segment::Cubic(lyon_geom::CubicBezierSegment {
                from: self.pen,
                to,
                ctrl1,
                ctrl2,
            }),
            ColorFlags::WHITE,
        ));

        self.pen = to;
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
}
