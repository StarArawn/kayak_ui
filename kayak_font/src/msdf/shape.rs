#![allow(dead_code)]

use crate::msdf::contour::Contour;

#[derive(Debug, Default, Clone)]
pub struct Shape {
    pub contours: Vec<Contour>,
    pub inverse_y_axis: bool,
}

impl Shape {
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
            inverse_y_axis: false,
        }
    }

    pub fn normalized(&mut self) {
        for contour in self.contours.iter_mut() {
            let (e0, e1, e2) = contour.edges[0].split_in_thirds();
            contour.edges.clear();
            contour.edges.push(e0);
            contour.edges.push(e1);
            contour.edges.push(e2);
        }
    }

    fn find_bounds(&mut self, left: &mut f64, bottom: &mut f64, right: &mut f64, top: &mut f64) {
        for contour in self.contours.iter_mut() {
            contour.find_bounds(left, bottom, right, top);
        }
    }

    pub fn bound_miters(
        &self,
        l: &mut f64,
        b: &mut f64,
        r: &mut f64,
        t: &mut f64,
        border: f64,
        miter_limit: f64,
        polarity: i32,
    ) {
        for contour in self.contours.iter() {
            contour.bound_miters(l, b, r, t, border, miter_limit, polarity);
        }
    }

    pub fn get_bounds(&mut self) -> (f64, f64, f64, f64) {
        const LARGE_VALUE: f64 = 1e240;
        let mut left = -LARGE_VALUE;
        let mut bottom = LARGE_VALUE;
        let mut right = LARGE_VALUE;
        let mut top = -LARGE_VALUE;
        self.find_bounds(&mut left, &mut bottom, &mut right, &mut top);
        return (left, bottom, right, top);
    }
}
