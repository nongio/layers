//! Types used in the library to describe Layers properties
use oklab::{oklab_to_srgb, srgb_to_oklab, Oklab, RGB};
use skia_safe::Color4f;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color {
    pub l: f64,
    pub a: f64,
    pub b: f64,
    pub alpha: f64,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[allow(dead_code)]
pub type Size = Point;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Point3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug)]
pub struct GradientLinear {
    pub colors: Vec<Color>,
    pub points: Vec<f64>,
}
#[derive(Clone, Debug)]
pub struct GradientRadial {
    pub center: Point,
    pub radius: f64,
    pub colors: Vec<Color>,
    pub points: Vec<Point>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
#[repr(C)]
pub enum PaintColor {
    Solid { color: Color },
    GradientLinear(Box<GradientLinear>),
    GradientRadial(Box<GradientRadial>),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum BorderStyle {
    Solid,
    Dotted,
    Dashed,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BorderRadius {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

impl BorderRadius {
    pub fn new_single(r: f64) -> Self {
        BorderRadius {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        }
    }
    #[allow(dead_code)]
    fn set(mut self, radius: f64) -> Self {
        self.top_left = radius;
        self.top_right = radius;
        self.bottom_right = radius;
        self.bottom_left = radius;
        self
    }
}
impl Default for BorderRadius {
    fn default() -> Self {
        BorderRadius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_left: 0.0,
            bottom_right: 0.0,
        }
    }
}
impl Default for Color {
    fn default() -> Self {
        Color {
            l: 1.0,
            a: 1.0,
            b: 1.0,
            alpha: 1.0,
        }
    }
}

impl Color {
    // Put in the public domain by Björn Ottosson 2020
    pub fn new_rgba(r: f64, g: f64, b: f64, alpha: f64) -> Self {
        let Oklab { l, a, b } = srgb_to_oklab(RGB {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        });

        Color {
            l: l as f64,
            a: a as f64,
            b: b as f64,
            alpha,
        }
    }

    pub fn new_rgba255(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0,
        )
    }

    pub fn new_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap()
        } else {
            255
        };
        Self::new_rgba255(r, g, b, a)
    }
}

impl Default for Point {
    fn default() -> Self {
        Point { x: 0.0, y: 0.0 }
    }
}
// skia conversions

impl From<Color> for Color4f {
    fn from(color: Color) -> Self {
        let Color { l, a, b, alpha } = color;
        let rgb = oklab_to_srgb(Oklab {
            l: l as f32,
            a: a as f32,
            b: b as f32,
        });

        Self {
            r: (rgb.r as f32 / 255.0),
            g: (rgb.g as f32 / 255.0),
            b: (rgb.b as f32 / 255.0),
            a: alpha as f32,
        }
    }
}

impl From<Point> for skia_safe::Point {
    fn from(point: Point) -> Self {
        skia_safe::Point {
            x: point.x as f32,
            y: point.y as f32,
        }
    }
}

impl From<Point3d> for skia_safe::Point3 {
    fn from(point: Point3d) -> Self {
        skia_safe::Point3 {
            x: point.x as f32,
            y: point.y as f32,
            z: point.z as f32,
        }
    }
}

impl From<(u32, u32)> for Point {
    fn from(point: (u32, u32)) -> Self {
        Point {
            x: point.0 as f64,
            y: point.1 as f64,
        }
    }
}

impl From<(usize, usize)> for Point {
    fn from(point: (usize, usize)) -> Self {
        Point {
            x: point.0 as f64,
            y: point.1 as f64,
        }
    }
}
impl From<(f64, f64)> for Point {
    fn from(point: (f64, f64)) -> Self {
        Point {
            x: point.0,
            y: point.1,
        }
    }
}

impl From<Color> for PaintColor {
    fn from(color: Color) -> Self {
        PaintColor::Solid { color }
    }
}

impl From<f64> for BorderRadius {
    fn from(radius: f64) -> Self {
        BorderRadius::new_single(radius)
    }
}
