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
    pub fn new_rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        let l_ = libm::cbrt(l);
        let m_ = libm::cbrt(m);
        let s_ = libm::cbrt(s);

        Color {
            l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            b: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            a: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
            alpha: a,
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

// skia conversions

impl From<Color> for Color4f {
    fn from(color: Color) -> Self {
        let l_ = color.l + 0.3963377774 * color.a + 0.2158037573 * color.b;
        let m_ = color.l - 0.1055613458 * color.a - 0.0638541728 * color.b;
        let s_ = color.l - 0.0894841775 * color.a - 1.2914855480 * color.b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        Self {
            r: (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s) as f32,
            g: (-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s) as f32,
            b: (-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s) as f32,
            a: color.alpha as f32,
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
