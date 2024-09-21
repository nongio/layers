use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use taffy::style::Display;

use crate::{
    engine::command::Attribute,
    types::{BlendMode, Color, Point, *},
};

// pub type ContentDrawFunction = Box<dyn Fn(&mut skia_safe::Canvas, f32, f32)>;

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct ContentDrawFunction(
    pub Arc<dyn 'static + Send + Sync + Fn(&skia_safe::Canvas, f32, f32) -> skia_safe::Rect>,
);

impl<F: Fn(&skia_safe::Canvas, f32, f32) -> skia_safe::Rect + Send + Sync + 'static> From<F>
    for ContentDrawFunction
{
    fn from(f: F) -> Self {
        ContentDrawFunction(Arc::new(f))
    }
}

impl std::fmt::Debug for ContentDrawFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContentDrawFunction").finish()
    }
}
impl PartialEq for ContentDrawFunction {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
use std::fmt;

use super::Layer;

#[derive(Debug)]
pub struct ContentDrawError {
    pub message: String,
}

impl fmt::Display for ContentDrawError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ContentDrawError {}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct PointerHandlerFunction(pub Arc<dyn 'static + Send + Sync + Fn(Layer, f32, f32)>);

impl<F: Fn(Layer, f32, f32) + Send + Sync + 'static> From<F> for PointerHandlerFunction {
    fn from(f: F) -> Self {
        PointerHandlerFunction(Arc::new(f))
    }
}

pub(crate) struct ModelLayer {
    pub key: RwLock<String>,
    pub display: Attribute<Display>,
    pub anchor_point: Attribute<Point>,
    pub position: Attribute<Point>,
    pub scale: Attribute<Point>,
    pub rotation: Attribute<Point3d>,
    pub size: Attribute<Size>,
    pub background_color: Attribute<PaintColor>,
    pub border_corner_radius: Attribute<BorderRadius>,
    pub border_color: Attribute<PaintColor>,
    pub border_width: Attribute<f32>,
    pub shadow_offset: Attribute<Point>,
    pub shadow_radius: Attribute<f32>,
    pub shadow_spread: Attribute<f32>,
    pub shadow_color: Attribute<Color>,
    pub draw_content: Arc<RwLock<Option<ContentDrawFunction>>>,
    pub blend_mode: Attribute<BlendMode>,
    pub opacity: Attribute<f32>,
}

impl Default for ModelLayer {
    fn default() -> Self {
        let position = Attribute::new(Point { x: 0.0, y: 0.0 });
        let size = Attribute::new(Size {
            width: taffy::style::Dimension::Length(0.0),
            height: taffy::style::Dimension::Length(0.0),
        });
        let anchor_point = Attribute::new(Point { x: 0.0, y: 0.0 });
        let scale = Attribute::new(Point { x: 1.0, y: 1.0 });
        let rotation = Attribute::new(Point3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let background_color = Attribute::new(PaintColor::Solid {
            color: Color::new_rgba(1.0, 1.0, 1.0, 0.0),
        });
        let border_corner_radius = Attribute::new(BorderRadius::new_single(0.0));
        let border_color = Attribute::new(PaintColor::Solid {
            color: Color::new_rgba(0.0, 0.0, 0.0, 1.0),
        });
        let border_width = Attribute::new(0.0);
        let shadow_offset = Attribute::new(Point { x: 0.0, y: 0.0 });
        let shadow_radius = Attribute::new(0.0);
        let shadow_spread = Attribute::new(0.0);
        let shadow_color = Attribute::new(Color::new_rgba(0.0, 0.0, 0.0, 0.0));
        let content = Arc::new(RwLock::new(None));
        let blend_mode = Attribute::new(BlendMode::Normal);
        let opacity = Attribute::new(1.0);
        let display = Attribute::new(Display::None);
        Self {
            key: RwLock::new(String::new()),
            display,
            anchor_point,
            position,
            scale,
            rotation,
            size,
            background_color,
            border_corner_radius,
            border_color,
            border_width,
            shadow_offset,
            shadow_radius,
            shadow_spread,
            shadow_color,
            draw_content: content,
            blend_mode,
            opacity,
        }
    }
}
