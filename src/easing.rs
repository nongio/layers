use std::fmt::Debug;

use crate::layer::{Color, PaintColor, Point};

fn linspace(steps: u64, step: u64) -> f64 {
    step as f64 / (steps as f64 - 1.0)
}

pub trait Interpolable:
    std::ops::Mul<f64, Output = Self>
    + std::ops::Add<Output = Self>
    + std::cmp::PartialEq
    + Debug
    + Sized
{
}

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
// implementation of PartialEq trait for Point
impl std::cmp::PartialEq for crate::layer::Point {
    fn eq(&self, other: &crate::layer::Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}
// implementation of PartialEq trait for BorderRadius
impl std::cmp::PartialEq for crate::layer::BorderRadius {
    fn eq(&self, other: &crate::layer::BorderRadius) -> bool {
        self.top_left == other.top_left
            && self.top_right == other.top_right
            && self.bottom_left == other.bottom_left
            && self.bottom_right == other.bottom_right
    }
}
// implementation of Add trait for BorderRadius
impl std::ops::Add for crate::layer::BorderRadius {
    type Output = crate::layer::BorderRadius;

    fn add(self, other: crate::layer::BorderRadius) -> crate::layer::BorderRadius {
        crate::layer::BorderRadius {
            top_left: self.top_left + other.top_left,
            top_right: self.top_right + other.top_right,
            bottom_left: self.bottom_left + other.bottom_left,
            bottom_right: self.bottom_right + other.bottom_right,
        }
    }
}
// implementation of Mul<f64> trait for BorderRadius
impl std::ops::Mul<f64> for crate::layer::BorderRadius {
    type Output = crate::layer::BorderRadius;

    fn mul(self, other: f64) -> crate::layer::BorderRadius {
        crate::layer::BorderRadius {
            top_left: self.top_left * other,
            top_right: self.top_right * other,
            bottom_left: self.bottom_left * other,
            bottom_right: self.bottom_right * other,
        }
    }
}
// implementation of Mul<f64> trait for Color
impl std::ops::Mul<f64> for crate::layer::Color {
    type Output = crate::layer::Color;

    fn mul(self, other: f64) -> crate::layer::Color {
        crate::layer::Color {
            l: self.l * other,
            a: self.a * other,
            b: self.b * other,
            alpha: self.alpha * other,
        }
    }
}
// implementation of Add trait for Color
impl std::ops::Add for crate::layer::Color {
    type Output = crate::layer::Color;

    fn add(self, other: crate::layer::Color) -> crate::layer::Color {
        crate::layer::Color {
            l: self.l + other.l,
            a: self.a + other.a,
            b: self.b + other.b,
            alpha: self.alpha + other.alpha,
        }
    }
}
// implementation of Mul<f64> trait for PaintColor
impl std::ops::Mul<f64> for PaintColor {
    type Output = PaintColor;

    fn mul(self, other: f64) -> PaintColor {
        match self {
            PaintColor::Solid { color } => PaintColor::Solid {
                color: color * other,
            },
            PaintColor::GradientLinear { colors, points } => PaintColor::GradientLinear {
                colors: colors.iter().map(|c| *c * other).collect(),
                points: points.clone(),
            },
            PaintColor::GradientRadial {
                center,
                radius,
                colors,
                points,
            } => PaintColor::GradientRadial {
                center,
                radius,
                colors: colors.iter().map(|c| *c * other).collect(),
                points: points.clone(),
            },
        }
    }
}

// implementation of Add trait for PaintColor
impl std::ops::Add for PaintColor {
    type Output = PaintColor;

    fn add(self, other: PaintColor) -> PaintColor {
        match self {
            PaintColor::Solid { color } => match other {
                PaintColor::Solid { color: other_color } => PaintColor::Solid {
                    color: color + other_color,
                },
                PaintColor::GradientLinear { colors, points } => PaintColor::GradientLinear {
                    colors: colors.clone(),
                    points: points.clone(),
                },
                PaintColor::GradientRadial {
                    center,
                    radius,
                    colors,
                    points,
                } => PaintColor::GradientRadial {
                    center: center,
                    radius: radius,
                    colors: colors.clone(),
                    points: points.clone(),
                },
            },
            PaintColor::GradientLinear { colors, points } => match other {
                PaintColor::Solid { color: other_color } => {
                    PaintColor::Solid { color: other_color }
                }
                PaintColor::GradientLinear {
                    colors: other_colors,
                    points: other_points,
                } => PaintColor::GradientLinear {
                    colors: other_colors.clone(),
                    points: other_points.clone(),
                },
                PaintColor::GradientRadial {
                    center,
                    radius,
                    colors,
                    points,
                } => PaintColor::GradientRadial {
                    center,
                    radius,
                    colors: colors.clone(),
                    points: points.clone(),
                },
            },
            PaintColor::GradientRadial {
                center,
                radius,
                colors,
                points,
            } => match other {
                PaintColor::Solid { color: other_color } => {
                    PaintColor::Solid { color: other_color }
                }
                PaintColor::GradientLinear {
                    colors: other_colors,
                    points: other_points,
                } => PaintColor::GradientLinear {
                    colors: other_colors.clone(),
                    points: other_points.clone(),
                },
                PaintColor::GradientRadial {
                    center: other_center,
                    radius: other_radius,
                    colors: other_colors,
                    points: other_points,
                } => PaintColor::GradientRadial {
                    center: other_center.clone(),
                    radius: other_radius,
                    colors: other_colors.clone(),
                    points: other_points.clone(),
                },
            },
        }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.l == other.l && self.a == other.a && self.b == other.b && self.alpha == other.alpha
    }
}
// implementation of PartiallyEq trait for PaintColor
impl std::cmp::PartialEq for PaintColor {
    fn eq(&self, other: &PaintColor) -> bool {
        match (self, other) {
            (PaintColor::Solid { color: c1 }, PaintColor::Solid { color: c2 }) => c1 == c2,
            (
                PaintColor::GradientLinear {
                    colors: c1,
                    points: p1,
                },
                PaintColor::GradientLinear {
                    colors: c2,
                    points: p2,
                },
            ) => c1 == c2 && p1 == p2,
            (
                PaintColor::GradientRadial {
                    center: c1,
                    radius: r1,
                    colors: cols1,
                    points: p2,
                },
                PaintColor::GradientRadial {
                    center: c2,
                    radius: r2,
                    colors: cols2,
                    points: p3,
                },
            ) => c1 == c2 && r1 == r2 && cols1 == cols2 && p2 == p3,
            _ => false,
        }
    }
}

impl Interpolable for f64 {}
impl Interpolable for crate::layer::Point {}
impl Interpolable for crate::layer::BorderRadius {}
impl Interpolable for crate::layer::PaintColor {}

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
