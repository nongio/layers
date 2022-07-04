use std::{default, sync::atomic::AtomicUsize};

use crate::{easing::Interpolable, ecs::animations::*};
use indexmap::IndexMap;
use skia_safe::{Color4f, Matrix};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub l: f64,
    pub a: f64,
    pub b: f64,
    pub alpha: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub enum PaintColor {
    Solid {
        color: Color,
    },
    GradientLinear {
        colors: Vec<Color>,
        points: Vec<Point>,
    },
    GradientRadial {
        center: Point,
        radius: f64,
        colors: Vec<Color>,
        points: Vec<Point>,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum BorderStyle {
    Solid,
    Dotted,
    Dashed,
}

#[derive(Clone, Copy, Debug)]
pub struct BorderRadius {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

#[derive(Clone, Debug)]
pub struct RenderLayer {
    pub position: Point,
    pub background_color: PaintColor,
    pub border_color: PaintColor,
    pub border_width: f64,
    pub border_style: BorderStyle,
    pub border_corner_radius: BorderRadius,
    pub size: Point,
    pub matrix: Matrix,
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
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
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

#[derive(Debug, Clone)]
pub enum Properties {
    Position(AnimatedValue<Point>),
    BackgroundColor(AnimatedValue<PaintColor>),
    BorderColor(AnimatedValue<PaintColor>),
    BorderWidth(AnimatedValue<f64>),
    BorderStyle(BorderStyle),
    BorderCornerRadius(AnimatedValue<BorderRadius>),
    Size(AnimatedValue<Point>),
}

impl Properties {
    pub fn key(&self) -> String {
        match self {
            Properties::Position(_) => "position".to_string(),
            Properties::BackgroundColor(_) => "background_color".to_string(),
            Properties::BorderColor(_) => "border_color".to_string(),
            Properties::BorderWidth(_) => "border_width".to_string(),
            Properties::BorderStyle(_) => "border_style".to_string(),
            Properties::BorderCornerRadius(_) => "border_corner_radius".to_string(),
            Properties::Size(_) => "size".to_string(),
        }
    }
}
#[derive(Debug)]
pub struct ModelLayer {
    pub id: usize,
    pub properties: IndexMap<String, Properties>,
}

#[derive(Clone, Debug)]
pub struct ValueChange<V: Interpolable + Sync> {
    pub from: V,
    pub to: V,
    pub target: AnimatedValue<V>,
    pub transition: Option<Transition<Easing>>,
}

pub enum ModelChanges {
    Point(usize, ValueChange<Point>, bool),
    F64(usize, ValueChange<f64>, bool),
    BorderCornerRadius(usize, ValueChange<BorderRadius>, bool),
    PaintColor(usize, ValueChange<PaintColor>, bool),
}

impl ModelLayer {
    pub fn new() -> Self {
        Self {
            ..default::Default::default()
        }
    }

    pub fn position(&self) -> AnimatedValue<Point> {
        if let Properties::Position(p) = self.properties.get("position").unwrap() {
            p.clone()
        } else {
            panic!("position not found")
        }
    }

    pub fn position_to(&self, p: Point, transition: Option<Transition<Easing>>) -> ModelChanges {
        let position = self.position();
        ModelChanges::Point(
            self.id,
            ValueChange {
                from: position.value(),
                to: p,
                target: position,
                transition,
            },
            false,
        )
    }

    pub fn background_color(&self) -> AnimatedValue<PaintColor> {
        if let Properties::BackgroundColor(p) = self.properties.get("background_color").unwrap() {
            p.clone()
        } else {
            panic!("background_color not found")
        }
    }

    pub fn background_color_to(
        &self,
        c: PaintColor,
        transition: Option<Transition<Easing>>,
    ) -> ModelChanges {
        let color = self.background_color();
        ModelChanges::PaintColor(
            self.id,
            ValueChange {
                from: color.value(),
                to: c,
                target: color,
                transition,
            },
            true,
        )
    }

    pub fn border_color(&self) -> AnimatedValue<PaintColor> {
        if let Properties::BorderColor(p) = self.properties.get("border_color").unwrap() {
            p.clone()
        } else {
            panic!("border_color not found")
        }
    }

    pub fn border_color_to(
        &self,
        c: PaintColor,
        transition: Option<Transition<Easing>>,
    ) -> ModelChanges {
        let color = self.border_color();
        ModelChanges::PaintColor(
            self.id,
            ValueChange {
                from: color.value(),
                to: c,
                target: color,
                transition,
            },
            true,
        )
    }

    pub fn border_width(&self) -> AnimatedValue<f64> {
        if let Properties::BorderWidth(p) = self.properties.get("border_width").unwrap() {
            p.clone()
        } else {
            panic!("border_width not found")
        }
    }

    pub fn border_width_to(&self, w: f64, transition: Option<Transition<Easing>>) -> ModelChanges {
        let width = self.border_width();
        ModelChanges::F64(
            self.id,
            ValueChange {
                from: width.value(),
                to: w,
                target: width,
                transition,
            },
            true,
        )
    }

    pub fn border_style(&self) -> BorderStyle {
        if let Properties::BorderStyle(p) = self.properties.get("border_style").unwrap() {
            p.clone()
        } else {
            panic!("border_style not found")
        }
    }

    pub fn border_corner_radius(&self) -> AnimatedValue<BorderRadius> {
        if let Properties::BorderCornerRadius(p) =
            self.properties.get("border_corner_radius").unwrap()
        {
            p.clone()
        } else {
            panic!("border_corner_radius not found")
        }
    }

    pub fn border_corner_radius_to(
        &self,
        r: BorderRadius,
        transition: Option<Transition<Easing>>,
    ) -> ModelChanges {
        let radius = self.border_corner_radius();
        ModelChanges::BorderCornerRadius(
            self.id,
            ValueChange {
                from: radius.value(),
                to: r,
                target: radius,
                transition,
            },
            true,
        )
    }

    pub fn size(&self) -> AnimatedValue<Point> {
        if let Properties::Size(p) = self.properties.get("size").unwrap() {
            p.clone()
        } else {
            panic!("size not found")
        }
    }

    pub fn size_to(&self, s: Point, transition: Option<Transition<Easing>>) -> ModelChanges {
        let size = self.size();
        ModelChanges::Point(
            self.id,
            ValueChange {
                from: size.value(),
                to: s,
                target: size,
                transition,
            },
            true,
        )
    }

    pub fn render_layer(&self) -> RenderLayer {
        let position = self.position().value();
        let matrix = Matrix::translate((position.x as f32, position.y as f32));
        RenderLayer {
            position,
            background_color: self.background_color().value(),
            border_color: self.border_color().value(),
            border_width: self.border_width().value(),
            border_style: self.border_style().clone(),
            border_corner_radius: self.border_corner_radius().value(),
            size: self.size().value(),
            matrix,
        }
    }
}

impl RenderLayer {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let RenderLayer { position, size, .. } = self;
        let x = x - position.x;
        let y = y - position.y;
        x >= 0.0 && x < size.x && y >= 0.0 && y < size.y
    }
}
// implement the trait Clone for ModelLayer
impl Clone for ModelLayer {
    fn clone(&self) -> Self {
        ModelLayer {
            id: self.id,
            properties: self.properties.clone(),
        }
    }
}

static OBJECT_COUNTER: AtomicUsize = AtomicUsize::new(1);

// implement Default for ModelLayer
impl Default for ModelLayer {
    fn default() -> Self {
        let id = OBJECT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut map = IndexMap::new();
        let position = Properties::Position(AnimatedValue::new(Point { x: 0.0, y: 0.0 }));
        let background_color = Properties::BackgroundColor(AnimatedValue::new(PaintColor::Solid {
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        }));
        let border_color = Properties::BorderColor(AnimatedValue::new(PaintColor::Solid {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        }));
        let border_width = Properties::BorderWidth(AnimatedValue::new(0.0));
        let border_style = Properties::BorderStyle(BorderStyle::Solid);
        let border_corner_radius =
            Properties::BorderCornerRadius(AnimatedValue::new(BorderRadius::new_single(25.0)));
        let size = Properties::Size(AnimatedValue::new(Point { x: 50.0, y: 50.0 }));

        map.insert(position.key(), position);
        map.insert(background_color.key(), background_color);
        map.insert(border_color.key(), border_color);
        map.insert(border_width.key(), border_width);
        map.insert(border_style.key(), border_style);
        map.insert(border_corner_radius.key(), border_corner_radius);
        map.insert(size.key(), size);

        Self {
            id,
            properties: map,
        }
    }
}
