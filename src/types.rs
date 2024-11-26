//! Types used in the library to describe Layers properties
use oklab::{oklab_to_srgb, srgb_to_oklab, Oklab, RGB};
use serde::Serialize;
use skia_safe::Color4f;
use skia_safe::Vector;

pub use skia_safe::{Image, Matrix, Picture, M44, V3};

#[derive(Clone, Copy, Serialize, Debug)]
#[repr(C)]
pub struct Color {
    pub l: f32,
    pub a: f32,
    pub b: f32,
    pub alpha: f32,
}
impl Color {
    pub fn c4f(&self) -> skia::Color4f {
        Color4f::from(*self)
    }
}
#[derive(Clone, Copy, Serialize, Debug)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}
#[derive(Clone, Copy, Serialize, Debug)]
#[repr(C)]
pub struct Size {
    pub width: taffy::style::Dimension,
    pub height: taffy::style::Dimension,
}
impl From<skia_safe::Size> for Size {
    fn from(size: skia_safe::Size) -> Self {
        Size {
            width: taffy::style::Dimension::Length(size.width),
            height: taffy::style::Dimension::Length(size.height),
        }
    }
}
impl Size {
    pub fn points(width: f32, height: f32) -> Self {
        Size {
            width: taffy::style::Dimension::Length(width),
            height: taffy::style::Dimension::Length(height),
        }
    }
    pub fn percent(width: f32, height: f32) -> Self {
        Size {
            width: taffy::style::Dimension::Percent(width),
            height: taffy::style::Dimension::Percent(height),
        }
    }
    pub fn auto() -> Self {
        Size {
            width: taffy::style::Dimension::Auto,
            height: taffy::style::Dimension::Auto,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Size {
            width: taffy::style::Dimension::Auto,
            height: taffy::style::Dimension::Auto,
        }
    }
}
use core::ops::Sub;
impl Sub for &Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Point3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Clone, Copy, Default, Serialize, Debug)]
#[repr(C)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<skia_safe::Rect> for Rectangle {
    fn from(rect: skia_safe::Rect) -> Self {
        Rectangle {
            x: rect.left,
            y: rect.top,
            width: rect.width(),
            height: rect.height(),
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct GradientLinear {
    pub colors: Vec<Color>,
    pub points: Vec<f32>,
}
#[derive(Clone, Serialize, Debug)]
pub struct GradientRadial {
    pub center: Point,
    pub radius: f32,
    pub colors: Vec<Color>,
    pub points: Vec<Point>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
#[repr(C)]
pub enum PaintColor {
    Solid { color: Color },
    GradientLinear(Box<GradientLinear>),
    GradientRadial(Box<GradientRadial>),
}
impl Default for PaintColor {
    fn default() -> Self {
        PaintColor::Solid {
            color: Color::default(),
        }
    }
}
#[allow(dead_code)]
#[derive(Clone, Copy, Default, Serialize, Debug)]
#[repr(u32)]
pub enum BorderStyle {
    #[default]
    Solid,
    Dotted,
    Dashed,
}

#[derive(Clone, Copy, Debug, diff::Diff, Serialize)]
#[repr(C)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

#[allow(clippy::from_over_into)]
impl Into<[Vector; 4]> for BorderRadius {
    fn into(self) -> [Vector; 4] {
        [
            skia_safe::Point::new(self.top_left, self.top_left),
            skia_safe::Point::new(self.top_right, self.top_right),
            skia_safe::Point::new(self.bottom_left, self.bottom_left),
            skia_safe::Point::new(self.bottom_right, self.bottom_right),
        ]
    }
}

impl BorderRadius {
    pub fn new_single(r: f32) -> Self {
        BorderRadius {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        }
    }
    #[allow(dead_code)]
    fn set(mut self, radius: f32) -> Self {
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
            l: 0.0,
            a: 0.0,
            b: 0.0,
            alpha: 0.0,
        }
    }
}

impl Color {
    // Put in the public domain by Björn Ottosson 2020
    pub fn new_rgba(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        let Oklab { l, a, b } = srgb_to_oklab(RGB {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        });

        Color { l, a, b, alpha }
    }

    pub fn new_rgba255(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new_rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
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

#[derive(Clone, Copy, Default, Serialize, Debug)]
#[repr(u32)]
pub enum BlendMode {
    #[default]
    Normal,
    BackgroundBlur,
}

impl PartialEq for BlendMode {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

// skia conversions

impl From<Color> for Color4f {
    fn from(color: Color) -> Self {
        let Color { l, a, b, alpha } = color;
        let rgb = oklab_to_srgb(Oklab { l, a, b });

        Self {
            r: (rgb.r as f32 / 255.0),
            g: (rgb.g as f32 / 255.0),
            b: (rgb.b as f32 / 255.0),
            a: alpha,
        }
    }
}

impl From<Point> for skia_safe::Point {
    fn from(point: Point) -> Self {
        skia_safe::Point {
            x: point.x,
            y: point.y,
        }
    }
}
impl From<Point> for (f32, f32) {
    fn from(point: Point) -> (f32, f32) {
        (point.x, point.y)
    }
}

impl From<Point3d> for skia_safe::Point3 {
    fn from(point: Point3d) -> Self {
        skia_safe::Point3 {
            x: point.x,
            y: point.y,
            z: point.z,
        }
    }
}

impl From<(u32, u32)> for Point {
    fn from(point: (u32, u32)) -> Self {
        Point {
            x: point.0 as f32,
            y: point.1 as f32,
        }
    }
}

impl From<(usize, usize)> for Point {
    fn from(point: (usize, usize)) -> Self {
        Point {
            x: point.0 as f32,
            y: point.1 as f32,
        }
    }
}
impl From<(f32, f32)> for Point {
    fn from(point: (f32, f32)) -> Self {
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

impl From<f32> for BorderRadius {
    fn from(radius: f32) -> Self {
        BorderRadius::new_single(radius)
    }
}