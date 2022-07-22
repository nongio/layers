use std::fmt::Debug;

use crate::types::{BorderRadius, Color, PaintColor, Point, Point3d};

#[allow(dead_code)]
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
impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
// implementation of Mul<f64> trait for Point
impl std::ops::Mul<f64> for Point {
    type Output = Point;

    fn mul(self, other: f64) -> Point {
        Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

// implementation of Add trait for Point
impl std::ops::Add for Point3d {
    type Output = Point3d;

    fn add(self, other: Point3d) -> Point3d {
        Point3d {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// implementation of Mul<f64> trait for Point
impl std::ops::Mul<f64> for Point3d {
    type Output = Point3d;

    fn mul(self, other: f64) -> Point3d {
        Point3d {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

// implementation of PartialEq trait for Point
impl std::cmp::PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

// implementation of PartialEq trait for Point
impl std::cmp::PartialEq for Point3d {
    fn eq(&self, other: &Point3d) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

// implementation of PartialEq trait for BorderRadius
impl std::cmp::PartialEq for BorderRadius {
    fn eq(&self, other: &BorderRadius) -> bool {
        self.top_left == other.top_left
            && self.top_right == other.top_right
            && self.bottom_left == other.bottom_left
            && self.bottom_right == other.bottom_right
    }
}
// implementation of Add trait for BorderRadius
impl std::ops::Add for BorderRadius {
    type Output = BorderRadius;

    fn add(self, other: BorderRadius) -> BorderRadius {
        BorderRadius {
            top_left: self.top_left + other.top_left,
            top_right: self.top_right + other.top_right,
            bottom_left: self.bottom_left + other.bottom_left,
            bottom_right: self.bottom_right + other.bottom_right,
        }
    }
}
// implementation of Mul<f64> trait for BorderRadius
impl std::ops::Mul<f64> for BorderRadius {
    type Output = BorderRadius;

    fn mul(self, other: f64) -> BorderRadius {
        BorderRadius {
            top_left: self.top_left * other,
            top_right: self.top_right * other,
            bottom_left: self.bottom_left * other,
            bottom_right: self.bottom_right * other,
        }
    }
}
// implementation of Mul<f64> trait for Color
impl std::ops::Mul<f64> for Color {
    type Output = Color;

    fn mul(self, other: f64) -> Color {
        Color {
            l: self.l * other,
            a: self.a * other,
            b: self.b * other,
            alpha: self.alpha * other,
        }
    }
}
// implementation of Add trait for Color
impl std::ops::Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
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
                points,
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
                points,
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
                PaintColor::GradientLinear { colors, points } => {
                    PaintColor::GradientLinear { colors, points }
                }
                PaintColor::GradientRadial {
                    center,
                    radius,
                    colors,
                    points,
                } => PaintColor::GradientRadial {
                    center,
                    radius,
                    colors,
                    points,
                },
            },
            PaintColor::GradientLinear { .. } => match other {
                PaintColor::Solid { color: other_color } => {
                    PaintColor::Solid { color: other_color }
                }
                PaintColor::GradientLinear {
                    colors: other_colors,
                    points: other_points,
                } => PaintColor::GradientLinear {
                    colors: other_colors,
                    points: other_points,
                },
                PaintColor::GradientRadial {
                    center,
                    radius,
                    colors,
                    points,
                } => PaintColor::GradientRadial {
                    center,
                    radius,
                    colors,
                    points,
                },
            },
            PaintColor::GradientRadial { .. } => match other {
                PaintColor::Solid { color: other_color } => {
                    PaintColor::Solid { color: other_color }
                }
                PaintColor::GradientLinear {
                    colors: other_colors,
                    points: other_points,
                } => PaintColor::GradientLinear {
                    colors: other_colors,
                    points: other_points,
                },
                PaintColor::GradientRadial {
                    center: other_center,
                    radius: other_radius,
                    colors: other_colors,
                    points: other_points,
                } => PaintColor::GradientRadial {
                    center: other_center,
                    radius: other_radius,
                    colors: other_colors,
                    points: other_points,
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
impl Interpolable for crate::types::Point {}
impl Interpolable for crate::types::Point3d {}
impl Interpolable for crate::types::BorderRadius {}
impl Interpolable for crate::types::PaintColor {}
impl Interpolable for crate::types::Color {}

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
    f64::NAN
}

pub fn bezier_easing_function(x1: f64, y1: f64, x2: f64, y2: f64, f: f64) -> f64 {
    assert!((0.0..=1.0).contains(&x1));
    assert!((0.0..=1.0).contains(&x1));
    assert!((0.0..=1.0).contains(&f));
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
