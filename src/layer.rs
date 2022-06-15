use skia_safe::{Color4f};


#[derive(Clone, Copy, Debug)]
pub struct Color{
    pub r:f32,
    pub g:f32,
    pub b:f32, 
    pub a:f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x:f64,
    pub y:f64,
}

pub enum PaintColor {
    Solid {color:Color},
    GradientLinear {
        colors: Vec<Color>,
        points: Vec<Point>,
    },
    GradientRadial {
        center: Point,
        radius: f32,
        colors: Vec<Color>,
        points: Vec<Point>,
    }
}
pub enum BorderStyle {
    Solid,
    Dotted,
    Dashed,
}

pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

pub struct RenderLayer {
    pub position: Point,
    pub background_color: PaintColor,
    pub border_color: PaintColor,
    pub border_width: f64,
    pub border_style: BorderStyle,
    pub border_corner_radius: BorderRadius,
    pub size: Point,
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
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Color {
    pub fn new(r:f32, g:f32, b:f32, a:f32) -> Self {
        Color {
            r,
            g,
            b,
            a,
        }
    }
}

// skia conversions 

impl From<Color> for Color4f {
    fn from(color: Color) -> Self {
        let Color{r, g, b, a} = color;
        
        Self { r, g, b, a }
    }
}