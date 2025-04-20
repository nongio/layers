use std::{
    error::Error,
    hash::{Hash, Hasher},
    sync::{atomic::AtomicBool, Arc, RwLock},
};

use skia::{ColorFilter, ImageFilter};
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
impl Hash for ContentDrawFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0) as *const ();
        ptr.hash(state);
    }
}
impl fmt::Debug for ContentDrawFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContentDrawFunction").finish()
    }
}
#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct ContentDrawFunctionInternal(
    pub Arc<dyn 'static + Send + Sync + Fn(&skia_safe::Canvas, f32, f32) -> skia_safe::Rect>,
);

impl<F: Fn(&skia_safe::Canvas, f32, f32) -> skia_safe::Rect + Send + Sync + 'static> From<F>
    for ContentDrawFunction
{
    fn from(f: F) -> Self {
        ContentDrawFunction(Arc::new(f))
    }
}

impl<F> From<F> for ContentDrawFunctionInternal
where
    F: Fn(&skia_safe::Canvas, f32, f32) -> skia_safe::Rect + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        ContentDrawFunctionInternal(Arc::new(f))
    }
}

impl From<ContentDrawFunction> for ContentDrawFunctionInternal {
    fn from(value: ContentDrawFunction) -> Self {
        ContentDrawFunctionInternal(Arc::new(move |canvas, x, y| (value.0)(canvas, x, y)))
    }
}

impl std::fmt::Debug for ContentDrawFunctionInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContentDrawFunctionInternal").finish()
    }
}

impl PartialEq for ContentDrawFunctionInternal {
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
pub struct PointerHandlerFunction(pub Arc<dyn 'static + Send + Sync + Fn(&Layer, f32, f32)>);

impl<F: Fn(&Layer, f32, f32) + Send + Sync + 'static> From<F> for PointerHandlerFunction {
    fn from(f: F) -> Self {
        PointerHandlerFunction(Arc::new(f))
    }
}

pub(crate) struct ModelLayer {
    pub(crate) key: RwLock<String>,

    pub(crate) pointer_events: Arc<AtomicBool>,

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
    pub draw_content: Arc<RwLock<Option<ContentDrawFunctionInternal>>>,
    pub blend_mode: Attribute<BlendMode>,
    pub opacity: Attribute<f32>,
    pub image_filter: Arc<RwLock<Option<ImageFilter>>>,
    pub color_filter: Arc<RwLock<Option<ColorFilter>>>,
    pub filter_bounds: Arc<RwLock<Option<skia::Rect>>>,
    pub image_filter_progress: Attribute<f32>,
    pub clip_content: Attribute<bool>,
    pub clip_children: Attribute<bool>,

    pub image_cached: Arc<AtomicBool>,
    pub picture_cached: Arc<AtomicBool>,
}

impl Default for ModelLayer {
    fn default() -> Self {
        let position = Attribute::new(Point { x: 0.0, y: 0.0 });
        let size = Attribute::new(Size {
            width: taffy::style::Dimension::Auto,
            height: taffy::style::Dimension::Auto,
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
        let image_filter = Arc::new(RwLock::new(None));
        let color_filter = Arc::new(RwLock::new(None));
        let filter_progress = Attribute::new(0.0);
        let filter_bounds = Arc::new(RwLock::new(None));
        let clip_content = Attribute::new(false);
        let clip_children = Attribute::new(false);
        let pointer_events = Arc::new(AtomicBool::new(true));
        // let hidden = Arc::new(AtomicBool::new(false));
        let image_cached = Arc::new(AtomicBool::new(true));
        let picture_cached = Arc::new(AtomicBool::new(true));

        Self {
            key: RwLock::new(String::new()),
            pointer_events,
            // hidden,
            image_cached,
            picture_cached,
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
            image_filter,
            color_filter,
            image_filter_progress: filter_progress,
            filter_bounds,
            clip_content,
            clip_children,
        }
    }
}
