const EPSILON: f64 = 1.0e-14;

pub fn fabs(v: f64) -> f64 {
    v.abs()
}

pub fn solve_quadratic(a: f64, b: f64, c: f64) -> (i32, [f64; 3]) {
    let mut result = [0.0; 3];

    if fabs(a) < EPSILON {
        if fabs(b) < EPSILON {
            if c == 0.0 {
                return (-1, result);
            }
            return (0, result);
        }
        result[0] = -c / b;
        return (1, result);
    }
    let mut dscr = b * b - 4.0 * a * c;
    if dscr > 0.0 {
        dscr = dscr.sqrt();
        result[0] = (-b + dscr) / (2.0 * a);
        result[1] = (-b - dscr) / (2.0 * a);
        return (2, result);
    } else if dscr == 0.0 {
        result[0] = -b / (2.0 * a);
        return (1, result);
    } else {
        return (0, result);
    }
}

pub fn solve_cubic_norm(mut a: f64, b: f64, c: f64) -> (i32, [f64; 3]) {
    let mut result = [0.0; 3];
    let a2 = a * a;
    let mut q = (a2 - 3.0 * b) / 9.0;
    let r = (a * (2.0 * a2 - 9.0 * b) + 27.0 * c) / 54.0;
    let r2 = r * r;
    let q3 = q * q * q;
    let mut result_a;
    let result_b;
    if r2 < q3 {
        let mut t = r / q3.sqrt();
        if t < -1.0 {
            t = -1.0;
        }
        if t > 1.0 {
            t = 1.0;
        }
        t = t.acos();
        a /= 3.0;
        q = -2.0 * q.sqrt();
        result[0] = q * (t / 3.0).cos() - a;
        result[1] = q * ((t + 2.0 * std::f64::consts::PI) / 3.0).cos() - a;
        result[2] = q * ((t - 2.0 * std::f64::consts::PI) / 3.0).cos() - a;
        return (3, result);
    } else {
        result_a = -(fabs(r) + (r2 - q3).sqrt()).powf(1.0 / 3.0);
        if r < 0.0 {
            result_a = -result_a
        };
        result_b = if result_a == 0.0 { 0.0 } else { q / result_a };
        a /= 3.0;
        result[0] = (result_a + result_b) - a;
        result[1] = -0.5 * (result_a + result_b) - a;
        result[2] = 0.5 * 3.0f64.sqrt() * (result_a - result_b);
        if fabs(result[2]) < EPSILON {
            return (2, result);
        }
        return (1, result);
    }
}

pub fn solve_cubic(a: f64, b: f64, c: f64, d: f64) -> (i32, [f64; 3]) {
    if fabs(a) < EPSILON {
        solve_quadratic(b, c, d)
    } else {
        solve_cubic_norm(b / a, c / a, d / a)
    }
}
