use std::ops::{Mul, MulAssign};

/// A struct for defining properties related to the corners of widgets
///
/// This is useful for things like border radii, etc.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Corner<T>
where
    T: Copy + Default + PartialEq,
{
    /// The value of the top-left corner
    pub top_left: T,
    /// The value of the top-right corner
    pub top_right: T,
    /// The value of the bottom-left corner
    pub bottom_left: T,
    /// The value of the bottom-right corner
    pub bottom_right: T,
}

impl<T> Corner<T>
where
    T: Copy + Default + PartialEq,
{
    /// Creates a new `Corner` with values individually specified for each corner
    ///
    /// # Arguments
    ///
    /// * `top_left`: The top-left corner value
    /// * `top_right`: The top_-right corner value
    /// * `bottom_left`: The bottom_-left corner value
    /// * `bottom_right`: The bottom_-right corner value
    ///
    pub fn new(top_left: T, top_right: T, bottom_left: T, bottom_right: T) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Creates a new `Corner` with matching top corners and matching bottom corners
    ///
    /// # Arguments
    ///
    /// * `top`: The value of the top corners
    /// * `bottom`: The value of the bottom corners
    ///
    /// ```
    /// # use kayak_core::styles::Corner;
    /// // Creates a `Corner` with only the top corners rounded
    /// let corner_radius = Corner::vertical(10.0, 0.0);
    ///
    /// // Creates a `Corner` with only the bottom corners rounded
    /// let corner_radius = Corner::vertical(0.0, 10.0);
    /// ```
    pub fn vertical(top: T, bottom: T) -> Self {
        Self {
            top_left: top,
            top_right: top,
            bottom_left: bottom,
            bottom_right: bottom,
        }
    }

    /// Creates a new `Corner` with matching left corners and matching right corners
    ///
    /// # Arguments
    ///
    /// * `left`: The value of the left corners
    /// * `right`: The value of the right corners
    ///
    /// ```
    /// # use kayak_core::styles::Corner;
    /// // Creates a `Corner` with only the left corners rounded
    /// let corner_radius = Corner::horizontal(10.0, 0.0);
    ///
    /// // Creates a `Corner` with only the right corners rounded
    /// let corner_radius = Corner::horizontal(0.0, 10.0);
    /// ```
    pub fn horizontal(left: T, right: T) -> Self {
        Self {
            top_left: left,
            top_right: right,
            bottom_left: left,
            bottom_right: right,
        }
    }

    /// Creates a new `Corner` with all corners having the same value
    ///
    /// # Arguments
    ///
    /// * `value`: The value of all corners
    ///
    pub fn all(value: T) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_left: value,
            bottom_right: value,
        }
    }

    /// Converts this `Corner` into a tuple matching `(Top Left, Top Right, Bottom Left, Bottom Right)`
    pub fn into_tuple(self) -> (T, T, T, T) {
        (
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
        )
    }
}

impl<T> From<Corner<T>> for (T, T, T, T)
where
    T: Copy + Default + PartialEq,
{
    /// Creates a tuple matching the pattern: `(Top Left, Top Right, Bottom Left, Bottom Right)`
    fn from(edge: Corner<T>) -> Self {
        edge.into_tuple()
    }
}

impl<T> From<T> for Corner<T>
where
    T: Copy + Default + PartialEq,
{
    fn from(value: T) -> Self {
        Corner::all(value)
    }
}

impl<T> From<(T, T, T, T)> for Corner<T>
where
    T: Copy + Default + PartialEq,
{
    /// Converts the tuple according to the pattern: `(Top Left, Top Right, Bottom Left, Bottom Right)`
    fn from(value: (T, T, T, T)) -> Self {
        Corner::new(value.0, value.1, value.2, value.3)
    }
}

impl<T> Mul<T> for Corner<T>
where
    T: Copy + Default + PartialEq + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            top_left: self.top_left * rhs,
            top_right: self.top_right * rhs,
            bottom_left: self.bottom_left * rhs,
            bottom_right: self.bottom_right * rhs,
        }
    }
}

impl<T> Mul<Corner<T>> for Corner<T>
where
    T: Copy + Default + PartialEq + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Corner<T>) -> Self::Output {
        Self {
            top_left: rhs.top_left * self.top_left,
            top_right: rhs.top_right * self.top_right,
            bottom_left: rhs.bottom_left * self.bottom_left,
            bottom_right: rhs.bottom_right * self.bottom_right,
        }
    }
}

impl<T> MulAssign<T> for Corner<T>
where
    T: Copy + Default + PartialEq + MulAssign,
{
    fn mul_assign(&mut self, rhs: T) {
        self.top_left *= rhs;
        self.top_right *= rhs;
        self.bottom_left *= rhs;
        self.bottom_right *= rhs;
    }
}

impl<T> MulAssign<Corner<T>> for Corner<T>
where
    T: Copy + Default + PartialEq + MulAssign,
{
    fn mul_assign(&mut self, rhs: Corner<T>) {
        self.top_left *= rhs.top_left;
        self.top_right *= rhs.top_right;
        self.bottom_left *= rhs.bottom_left;
        self.bottom_right *= rhs.bottom_right;
    }
}

#[cfg(test)]
mod tests {
    use super::Corner;

    #[test]
    fn tuples_should_convert_to_corner() {
        let expected = (1.0, 2.0, 3.0, 4.0);
        let corner: Corner<f32> = expected.into();
        assert_eq!(expected, corner.into_tuple());

        let expected = (1.0, 1.0, 1.0, 1.0);
        let corner: Corner<f32> = (expected.0).into();
        assert_eq!(expected, corner.into_tuple());

        let expected = (1.0, 1.0, 1.0, 1.0);
        let corner: Corner<f32> = expected.0.into();
        assert_eq!(expected, corner.into_tuple());
    }

    #[test]
    fn multiplication_should_work_on_corners() {
        let expected = (10.0, 20.0, 30.0, 40.0);
        let mut corner = Corner::new(1.0, 2.0, 3.0, 4.0);

        // Basic multiplication
        let multiplied = corner * 10.0;
        assert_eq!(expected, multiplied.into_tuple());

        // Multiply and assign
        corner *= 10.0;
        assert_eq!(expected, corner.into_tuple());
    }
}
