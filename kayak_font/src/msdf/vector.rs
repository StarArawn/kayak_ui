#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    pub fn get_ortho_normal(&self, polarity: bool, allow_zero: bool) -> Vector2 {
        let len = self.length();
        if len == 0.0 {
            let allow_zero = if !allow_zero { 1.0 } else { 0.0 };
            return if polarity {
                Vector2::new(0.0, allow_zero)
            } else {
                Vector2::new(0.0, -allow_zero)
            };
        }
        if polarity {
            Vector2::new(-self.y / len, self.x / len)
        } else {
            Vector2::new(self.y / len, -self.x / len)
        }
    }

    pub fn get_orthogonal(&self, polarity: bool) -> Vector2 {
        if polarity {
            Vector2::new(-self.y, self.x)
        } else {
            Vector2::new(self.y, -self.x)
        }
    }
    pub fn dot_product(a: Vector2, b: Vector2) -> f64 {
        a.x * b.x + a.y * b.y
    }
    pub fn cross_product(a: Vector2, b: Vector2) -> f64 {
        a.x * b.y - a.y * b.x
    }

    pub fn normalize(&self, allow_zero: bool) -> Vector2 {
        let len = self.length();
        if len == 0.0 {
            let allow_zero = if !allow_zero { 1.0 } else { 0.0 };
            return Vector2::new(0.0, allow_zero);
        }
        Vector2::new(self.x / len, self.y / len)
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn clamp(n: i32, b: i32) -> i32 {
        if n > 0 {
            if n <= b {
                n
            } else {
                b
            }
        } else {
            0
        }
    }

    pub fn sign(n: f64) -> f64 {
        if n == 0.0 {
            0.0
        } else if n > 0.0 {
            1.0
        } else {
            -1.0
        }
    }

    pub fn shoelace(a: Vector2, b: Vector2) -> f64 {
        (b.x - a.x) * (a.y + b.y)
    }

    pub fn point_bounds(p: Vector2, l: &mut f64, b: &mut f64, r: &mut f64, t: &mut f64) {
        if p.x < *l {
            *l = p.x;
        }
        if p.y < *b {
            *b = p.y;
        }
        if p.x > *r {
            *r = p.x;
        }
        if p.y > *t {
            *t = p.y;
        }
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Add for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Mul for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl std::ops::Div for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl std::ops::Mul<f64> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector2::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::Div<f64> for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: f64) -> Self::Output {
        Vector2::new(self.x / rhs, self.y / rhs)
    }
}

impl std::ops::Mul<Vector2> for f64 {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2::new(rhs.x * self, rhs.y * self)
    }
}

impl std::ops::Div<Vector2> for f64 {
    type Output = Vector2;

    fn div(self, rhs: Vector2) -> Self::Output {
        Vector2::new(rhs.x / self, rhs.y / self)
    }
}
