use std::fmt::Debug;

use crate::types::{
    BorderRadius, Color, GradientLinear, GradientRadial, PaintColor, Point, Point3d,
};
use skia_safe::Image;

#[allow(dead_code)]
fn linspace(steps: u64, step: u64) -> f64 {
    step as f64 / (steps as f64 - 1.0)
}

pub trait Interpolable:
    std::ops::Mul<f64, Output = Self>
    + std::ops::Add<Output = Self>
    + std::cmp::PartialEq
    + Clone
    + Debug
    + Sized
{
}

pub trait Interpolate {
    fn interpolate(&self, to: &Self, f: f64) -> Self;
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
// TODO incomplete for GradientLinear and GradientRadial
impl std::ops::Mul<f64> for PaintColor {
    type Output = PaintColor;

    fn mul(self, other: f64) -> PaintColor {
        match self {
            PaintColor::Solid { color } => PaintColor::Solid {
                color: color * other,
            },
            PaintColor::GradientLinear(gradient) => PaintColor::GradientLinear(gradient),
            PaintColor::GradientRadial(gradient) => PaintColor::GradientRadial(gradient),
        }
    }
}

// implementation of Add trait for PaintColor
// TODO incomplete for GradientLinear and GradientRadial

impl std::ops::Add for PaintColor {
    type Output = PaintColor;

    fn add(self, other: PaintColor) -> PaintColor {
        match self {
            PaintColor::Solid { color } => match other {
                PaintColor::Solid { color: other_color } => PaintColor::Solid {
                    color: color + other_color,
                },
                PaintColor::GradientLinear { .. } => other,
                PaintColor::GradientRadial { .. } => other,
            },
            PaintColor::GradientLinear { .. } => match other {
                PaintColor::Solid { color: other_color } => {
                    PaintColor::Solid { color: other_color }
                }
                PaintColor::GradientLinear { .. } => other,
                PaintColor::GradientRadial { .. } => other,
            },
            PaintColor::GradientRadial { .. } => match other {
                PaintColor::Solid { color: other_color } => {
                    PaintColor::Solid { color: other_color }
                }
                PaintColor::GradientLinear { .. } => other,
                PaintColor::GradientRadial { .. } => other,
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
// TODO incomplete for GradientLinear and GradientRadial

impl std::cmp::PartialEq for PaintColor {
    fn eq(&self, other: &PaintColor) -> bool {
        match (self, other) {
            (PaintColor::Solid { color: c1 }, PaintColor::Solid { color: c2 }) => c1 == c2,
            (PaintColor::GradientLinear(g1), PaintColor::GradientLinear(g2)) => **g1 == **g2,
            (PaintColor::GradientRadial(g1), PaintColor::GradientRadial(g2)) => **g1 == **g2,
            _ => false,
        }
    }
}
// implementation of PartiallyEq trait for GradientLinear
impl std::cmp::PartialEq for GradientLinear {
    fn eq(&self, other: &GradientLinear) -> bool {
        self.colors == other.colors && self.points == other.points
    }
}

// implementation of PartiallyEq trait for GradientRadial
impl std::cmp::PartialEq for GradientRadial {
    fn eq(&self, other: &GradientRadial) -> bool {
        self.center == other.center && self.radius == other.radius && self.points == other.points
    }
}

impl Interpolable for f64 {}
impl Interpolable for crate::types::Point {}
impl Interpolable for crate::types::Point3d {}
impl Interpolable for crate::types::BorderRadius {}
impl Interpolable for crate::types::Color {}
// this negative impl is needed to avoid the default implementation of Interpolate
// for PaintColor which is not correct
impl !Interpolable for crate::types::PaintColor {}
impl !Interpolable for Option<Image> {}

impl<V: Interpolable> Interpolate for V {
    fn interpolate(&self, other: &Self, f: f64) -> Self {
        let o = other.to_owned();
        let s = self.to_owned();
        s * (1.0 - f) + (o * f)
    }
}

impl Interpolate for PaintColor {
    fn interpolate(&self, other: &PaintColor, f: f64) -> PaintColor {
        match (self, other) {
            (PaintColor::Solid { color: c1 }, PaintColor::Solid { color: c2 }) => {
                PaintColor::Solid {
                    color: c1.interpolate(c2, f),
                }
            }
            _ => {
                // if we are not interpolating between two solid colors, we just return the first
                // or the second based on the value of f
                if f < 0.5 {
                    self.to_owned()
                } else {
                    other.to_owned()
                }
            }
        }
    }
}

impl Interpolate for Option<Image> {
    fn interpolate(&self, other: &Option<Image>, f: f64) -> Option<Image> {
        if f < 0.5 {
            self.clone().to_owned()
        } else {
            other.clone().to_owned()
        }
    }
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
