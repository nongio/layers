use std::{default, ops::Deref};

use indexmap::IndexMap;
use skia_safe::{Color4f};
use crate::ecs::animations::*;


#[derive(Clone, Copy, Debug)]
pub struct Color{
    pub r:f64,
    pub g:f64,
    pub b:f64, 
    pub a:f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x:f64,
    pub y:f64,
}

#[derive(Clone, Debug)]
pub enum PaintColor {
    Solid {color:Color},
    GradientLinear {
        colors: Vec<Color>,
        points: Vec<Point>,
    },
    GradientRadial {
        center: Point,
        radius: f64,
        colors: Vec<Color>,
        points: Vec<Point>,
    }
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
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Color {
    pub fn new(r:f64, g:f64, b:f64, a:f64) -> Self {
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
        
        Self { r: r as f32, g: g as f32, b: b as f32, a: a as f32 }
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
    pub properties: IndexMap<String, Properties>,
}

impl ModelLayer {
    pub fn new() -> Self {
        default::Default::default()
    }

    pub fn position(&self) -> AnimatedValue<Point> {
        // self.properties.get("position").unwrap().clone()
        if let Properties::Position(p) = self.properties.get("position").unwrap() {
            p.clone()
        } else {
            panic!("position not found")
        }
    }

    pub fn background_color(&self) -> AnimatedValue<PaintColor> {
        if let Properties::BackgroundColor(p) = self.properties.get("background_color").unwrap() {
            p.clone()
        } else {
            panic!("background_color not found")
        }
    }

    pub fn border_color(&self) -> AnimatedValue<PaintColor> {
        if let Properties::BorderColor(p) = self.properties.get("border_color").unwrap() {
            p.clone()
        } else {
            panic!("border_color not found")
        }
    }

    pub fn border_width(&self) -> AnimatedValue<f64> {
        if let Properties::BorderWidth(p) = self.properties.get("border_width").unwrap() {
            p.clone()
        } else {
            panic!("border_width not found")
        }
    }

    pub fn border_style(&self) -> BorderStyle {
        if let Properties::BorderStyle(p) = self.properties.get("border_style").unwrap() {
            p.clone()
        } else {
            panic!("border_style not found")
        }
    }

    pub fn border_corner_radius(&self) -> AnimatedValue<BorderRadius> {
        if let Properties::BorderCornerRadius(p) = self.properties.get("border_corner_radius").unwrap() {
            p.clone()
        } else {
            panic!("border_corner_radius not found")
        }
    }

    pub fn size(&self) -> AnimatedValue<Point> {
        if let Properties::Size(p) = self.properties.get("size").unwrap() {
            p.clone()
        } else {
            panic!("size not found")
        }
    }

    pub fn render_layer(&self) -> RenderLayer {
        RenderLayer {
            position: self.position().value(),
            background_color: self.background_color().value(),
            border_color: self.border_color().value(),
            border_width: self.border_width().value(),
            border_style: self.border_style().clone(),
            border_corner_radius: self.border_corner_radius().value(),
            size: self.size().value(),
        }
    }

    pub fn to(&self, layer: RenderLayer, transition: Option<Transition<Easing>>) -> (Vec<ValueChanges>, Option<Transition<Easing>>){
        let mut changes = Vec::<ValueChanges>::new();
        
        
        changes.push(self.position().to(layer.position, None));
        changes.push(self.border_width().to(layer.border_width, None));
        changes.push(self.border_corner_radius().to(layer.border_corner_radius, None));
        changes.push(self.size().to(layer.size, None));
        (changes, transition)
    }

}
impl RenderLayer {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let RenderLayer{position, size, ..} = self;
        let x = x - position.x;
        let y = y - position.y;
        x >= 0.0 && x < size.x && y >= 0.0 && y < size.y
    }
}
// implement the trait Clone for ModelLayer
impl Clone for ModelLayer {
    fn clone(&self) -> Self {
        ModelLayer {
            properties: self.properties.clone(),
        }
    }
}

// implement Default for ModelLayer
impl Default for ModelLayer {
    fn default() -> Self {
        let mut map = IndexMap::new();
        let position = Properties::Position(AnimatedValue::new(Point{x:0.0, y:0.0}));
        let background_color = Properties::BackgroundColor(AnimatedValue::new(PaintColor::Solid {color: Color::new(1.0, 0.0, 0.0, 1.0)}));
        let border_color = Properties::BorderColor(AnimatedValue::new(PaintColor::Solid {color: Color::new(0.0, 0.0, 0.0, 1.0)}));
        let border_width = Properties::BorderWidth(AnimatedValue::new(0.0));
        let border_style = Properties::BorderStyle(BorderStyle::Solid);
        let border_corner_radius = Properties::BorderCornerRadius(AnimatedValue::new(BorderRadius::new_single(25.0)));
        let size = Properties::Size(AnimatedValue::new(Point{x:50.0, y:50.0}));

        map.insert(position.key(), position);
        map.insert(background_color.key(), background_color);
        map.insert(border_color.key(), border_color);
        map.insert(border_width.key(), border_width);
        map.insert(border_style.key(), border_style);
        map.insert(border_corner_radius.key(), border_corner_radius);
        map.insert(size.key(), size);
        Self { properties: map }
    }
}