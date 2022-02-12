use std::ops::{Mul, MulAssign};

/// A struct for defining properties related to the edges of widgets
///
/// This is useful for things like borders, padding, etc.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Edge<T> where T: Copy + Default + PartialEq {
    /// The value of the top edge
    pub top: T,
    /// The value of the right edge
    pub right: T,
    /// The value of the bottom edge
    pub bottom: T,
    /// The value of the left edge
    pub left: T,
}

impl<T> Edge<T> where T: Copy + Default + PartialEq {
    /// Creates a new `Edge` with values individually specified for each edge
    ///
    /// # Arguments
    ///
    /// * `top`: The top edge value
    /// * `right`: The right edge value
    /// * `bottom`: The bottom edge value
    /// * `left`: The left edge value
    ///
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Creates a new `Edge` with matching vertical edges and matching horizontal edges
    ///
    /// # Arguments
    ///
    /// * `vertical`: The value of the vertical edges
    /// * `horizontal`: The value of the horizontal edges
    ///
    pub fn axis(vertical: T, horizontal: T) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Creates a new `Edge` with all edges having the same value
    ///
    /// # Arguments
    ///
    /// * `value`: The value of all edges
    ///
    pub fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Converts this `Edge` into a tuple matching `(Top, Right, Bottom, Left)`
    pub fn into_tuple(self) -> (T, T, T, T) {
        (self.top, self.right, self.bottom, self.left)
    }
}

impl<T> From<Edge<T>> for (T, T, T, T) where T: Copy + Default + PartialEq {
    fn from(edge: Edge<T>) -> Self {
        edge.into_tuple()
    }
}

impl<T> From<T> for Edge<T> where T: Copy + Default + PartialEq {
    fn from(value: T) -> Self {
        Edge::all(value)
    }
}

impl<T> From<(T, T)> for Edge<T> where T: Copy + Default + PartialEq {
    fn from(value: (T, T)) -> Self {
        Edge::axis(value.0, value.1)
    }
}

impl<T> From<(T, T, T, T)> for Edge<T> where T: Copy + Default + PartialEq {
    fn from(value: (T, T, T, T)) -> Self {
        Edge::new(value.0, value.1, value.2, value.3)
    }
}

impl<T> Mul<T> for Edge<T> where T: Copy + Default + PartialEq + Mul<Output=T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            top: self.top * rhs,
            right: self.right * rhs,
            bottom: self.bottom * rhs,
            left: self.left * rhs,
        }
    }
}

impl<T> Mul<Edge<T>> for Edge<T> where T: Copy + Default + PartialEq + Mul<Output=T> {
    type Output = Self;

    fn mul(self, rhs: Edge<T>) -> Self::Output {
        Self {
            top: rhs.top * self.top,
            right: rhs.right * self.right,
            bottom: rhs.bottom * self.bottom,
            left: rhs.left * self.left,
        }
    }
}

impl<T> MulAssign<T> for Edge<T> where T: Copy + Default + PartialEq + MulAssign {
    fn mul_assign(&mut self, rhs: T) {
        self.top *= rhs;
        self.right *= rhs;
        self.bottom *= rhs;
        self.left *= rhs;
    }
}

impl<T> MulAssign<Edge<T>> for Edge<T> where T: Copy + Default + PartialEq + MulAssign {
    fn mul_assign(&mut self, rhs: Edge<T>) {
        self.top *= rhs.top;
        self.right *= rhs.right;
        self.bottom *= rhs.bottom;
        self.left *= rhs.left;
    }
}

#[cfg(test)]
mod tests {
    use super::Edge;

    #[test]
    fn tuples_should_convert_to_edge() {
        let expected = (1.0, 2.0, 3.0, 4.0);
        let edge: Edge<f32> = expected.into();
        assert_eq!(expected, edge.into_tuple());

        let expected = (1.0, 2.0, 1.0, 2.0);
        let edge: Edge<f32> = (expected.0, expected.1).into();
        assert_eq!(expected, edge.into_tuple());

        let expected = (1.0, 1.0, 1.0, 1.0);
        let edge: Edge<f32> = (expected.0).into();
        assert_eq!(expected, edge.into_tuple());

        let expected = (1.0, 1.0, 1.0, 1.0);
        let edge: Edge<f32> = expected.0.into();
        assert_eq!(expected, edge.into_tuple());
    }
}