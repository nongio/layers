use std::fmt::Debug;

use crate::layer::Point;

// the widget id is the index in the vectors
// all vectors have the same length
enum EasingFunction {
    Linear,
    BezierEasing(f64, f64, f64, f64),
}

struct ScheduledAnimation {
    target_value: f64,
    duration: u64,
    start_time: u64,
    easing: EasingFunction,
}

struct Animation {
    start_value: f64,
    program: ScheduledAnimation,
}

fn linspace(steps: u64, step: u64) -> f64 {
    step as f64 / (steps as f64 - 1.0)
}

pub trait Interpolable : std::ops::Mul<f64, Output = Self> + std::ops::Add<Output = Self> + Debug + Copy {}

// implementation of Add trait for Point
impl std::ops::Add for crate::layer::Point {
    type Output = Point;

    fn add(self, other: crate::layer::Point) -> crate::layer::Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
// implementation of Mul<f64> trait for Point
impl std::ops::Mul<f64> for crate::layer::Point {
    type Output = crate::layer::Point;

    fn mul(self, other: f64) -> crate::layer::Point {
        crate::layer::Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}
impl Interpolable for f64 {}
impl Interpolable for crate::layer::Point {}

pub fn interpolate<T: Interpolable>(p1: T, p2: T, f: f64) -> T {
    p1 * (1.0 - f) + p2 * f
}

// easing version of the bezier 1d with p0 = 0 and p3 = 1
fn bezier_easing_1d(p1: f64, p2: f64, f: f64) -> f64 {
    let f2 = f * f;
    let f3 = f2 * f;
    f3 + 3.0 * f3 * p1 - 3.0 * f3 * p2 + 3.0 * f2 * p2 - 6.0 * f2 * p1 + 3.0 * f * p1
}

// derivative of the easing version of the bezier 1d with p0 = 0 and p3 = 1
fn bezier_easing_1d_prime(p1: f64, p2: f64, f: f64) -> f64 {
    let f2 = f * f;
    3.0 * f2 + 9.0 * f2 * p1 - 9.0 * f2 * p2 + 6.0 * f * p2 - 12.0 * f * p1 + 3.0 * p1
}

// newthon method to find the roots
fn find_root(p1: f64, p2: f64, target: f64) -> f64 {
    let mut p0 = 0.5;
    let tolerance = 1e-9;
    let epsilon = 1e-14;
    let max_iter = 100;
    for _ in 0..max_iter {
        let y = bezier_easing_1d(p1, p2, p0) - target;
        let y_prime = bezier_easing_1d_prime(p1, p2, p0);
        if y_prime.abs() < epsilon {
            break;
        }
        let p_next = p0 - y / y_prime;
        if (p_next - p0).abs() <= tolerance {
            return p_next;
        }
        p0 = p_next;
    }
    // numerical difficulties
    return f64::NAN;
}

pub fn bezier_easing_function(x1: f64, y1: f64, x2: f64, y2: f64, f: f64) -> f64 {
    assert!(x1 >= 0.0 && x1 <= 1.0);
    assert!(x1 >= 0.0 && x1 <= 1.0);
    assert!(f >= 0.0 && f <= 1.0);
    let curve_fraction = find_root(x1, x2, f);
    bezier_easing_1d(y1, y2, curve_fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_easing_function() {
        let epsilon = 1e-7;
        assert!((bezier_easing_function(0.0, 1.0, 1.0, 0.0, 0.5) - 0.5).abs() < epsilon);
        assert!((bezier_easing_function(0.0, 1.0, 1.0, 0.0, 0.0) - 0.0).abs() < epsilon);
        assert!((bezier_easing_function(0.0, 1.0, 1.0, 0.0, 1.0) - 1.0).abs() < epsilon);
    }
}
