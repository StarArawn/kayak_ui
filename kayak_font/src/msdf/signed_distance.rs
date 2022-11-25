#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignedDistance {
    pub distance: f64,
    pub dot: f64,
}

impl SignedDistance {
    pub fn infinite() -> Self {
        Self {
            distance: -1e240,
            dot: 1.0,
        }
    }

    pub fn new(distance: f64, dot: f64) -> Self {
        Self { distance, dot }
    }

    // pub fn g(&self, other: &SignedDistance) -> bool {
    //     self.distance.abs() > other.distance.abs()
    //         || (self.distance.abs() == other.distance.abs() && self.dot > other.dot)
    // }

    // pub fn ge(&self, other: &SignedDistance) -> bool {
    //     self.distance.abs() > other.distance.abs()
    //         || (self.distance.abs() == other.distance.abs() && self.dot >= other.dot)
    // }

    pub fn l(&self, other: &SignedDistance) -> bool {
        self.distance.abs() < other.distance.abs()
            || (self.distance.abs() == other.distance.abs() && self.dot < other.dot)
    }

    // pub fn le(&self, other: &SignedDistance) -> bool {
    //     self.distance.abs() < other.distance.abs()
    //         || (self.distance.abs() == other.distance.abs() && self.dot <= other.dot)
    // }
}
