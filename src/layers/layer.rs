use skia_safe::{Canvas, Image, Matrix, M44};
use std::sync::Arc;
use std::sync::RwLock;

use crate::drawing::layer::draw_layer;
use crate::engine::node::RenderNode;
use crate::engine::node::RenderableFlags;
use crate::engine::rendering::Drawable;
use crate::engine::storage::TreeStorageId;
use crate::engine::ChangeProducer;
use crate::engine::Engine;
use crate::layers::*;
use crate::types::*;

use super::change_attr;

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum BlendMode {
    Normal,
    BackgroundBlur,
}

impl PartialEq for BlendMode {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct SkiaImage {
    pub data: Box<Image>,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Layer {
    pub background_color: PaintColor,
    pub border_color: PaintColor,
    pub border_width: f64,
    pub border_style: BorderStyle,
    pub border_corner_radius: BorderRadius,
    pub size: Point,
    pub shadow_offset: Point,
    pub shadow_radius: f64,
    pub shadow_color: Color,
    pub shadow_spread: f64,
    pub matrix: Matrix,
    pub content: Option<SkiaImage>,
    pub blend_mode: BlendMode,
}
// #[derive(Clone)]
pub struct ModelLayer {
    pub anchor_point: SyncValue<Point>,
    pub position: SyncValue<Point>,
    pub scale: SyncValue<Point>,
    pub rotation: SyncValue<Point3d>,
    pub size: SyncValue<Point>,
    pub background_color: SyncValue<PaintColor>,
    pub border_corner_radius: SyncValue<BorderRadius>,
    pub border_color: SyncValue<PaintColor>,
    pub border_width: SyncValue<f64>,
    pub shadow_offset: SyncValue<Point>,
    pub shadow_radius: SyncValue<f64>,
    pub shadow_spread: SyncValue<f64>,
    pub shadow_color: SyncValue<Color>,
    pub matrix: M44,

    pub content: Option<Image>,
    pub blend_mode: BlendMode,

    pub engine: RwLock<Option<(TreeStorageId, Arc<Engine>)>>,
}

impl ModelLayer {
    fn new() -> Self {
        Default::default()
    }
    pub fn create() -> Arc<ModelLayer> {
        Arc::new(Self::new())
    }
    pub fn add_on_click_handler(&self, _handler: Box<dyn Fn()>) {
        let mut _engine = self.engine.write().unwrap();
        // if let Some((id, engine)) = engine.as_mut() {
        // engine.add_on_click_handler(*id, handler);
        // }
    }

    change_attr!(position, Point, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(
        size,
        Point,
        RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT
    );
    change_attr!(background_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_attr!(scale, Point, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(rotation, Point3d, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(anchor_point, Point, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(
        border_corner_radius,
        BorderRadius,
        RenderableFlags::NEEDS_PAINT
    );
    change_attr!(border_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_attr!(border_width, f64, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_offset, Point, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_radius, f64, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_spread, f64, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_color, Color, RenderableFlags::NEEDS_PAINT);
}

//api_change_attr!(ModelLayer, position, Point);

impl Default for ModelLayer {
    fn default() -> Self {
        let position = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let size = SyncValue::new(Point { x: 100.0, y: 100.0 });
        let anchor_point = SyncValue::new(size.value() * 0.5);
        let scale = SyncValue::new(Point { x: 1.0, y: 1.0 });
        let rotation = SyncValue::new(Point3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let background_color = SyncValue::new(PaintColor::Solid {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        });
        let border_corner_radius = SyncValue::new(BorderRadius::new_single(0.0));
        let border_color = SyncValue::new(PaintColor::Solid {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        });
        let border_width = SyncValue::new(0.0);
        let matrix = M44::new_identity();
        let shadow_offset = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let shadow_radius = SyncValue::new(0.0);
        let shadow_spread = SyncValue::new(0.0);
        let shadow_color = SyncValue::new(Color::new(0.0, 0.0, 0.0, 1.0));
        let content = None;
        let blend_mode = BlendMode::Normal;
        let engine = RwLock::new(None);
        Self {
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
            content,
            matrix,
            blend_mode,
            engine,
        }
    }
}

impl Drawable for ModelLayer {
    fn draw(&self, canvas: &mut Canvas) {
        let layer: Layer = Layer::from(self);
        draw_layer(canvas, &layer);
    }
    fn bounds(&self) -> Rectangle {
        let p = self.position.value();

        let s = self.size.value();

        Rectangle {
            x: p.x,
            y: p.y,
            width: s.x,
            height: s.y,
        }
    }
    fn transform(&self) -> Matrix {
        let s = self.scale.value();
        let p = self.position.value();
        let translate = M44::translate(p.x as f32, p.y as f32, 0.0);
        let scale = M44::scale(s.x as f32, s.y as f32, 1.0);
        // let rotate = M44::rotate(
        //     V3 {
        //         x: 0.0,
        //         y: 1.0,
        //         z: 0.0,
        //     },
        //     (p.x / 100.0) as f32,
        // );
        let transform = skia_safe::M44::concat(&translate, &scale);
        // let transform = skia_safe::M44::concat(&transform, &rotate);

        transform.to_m33()
    }
}

impl ChangeProducer for ModelLayer {
    fn set_engine(&self, engine: Arc<Engine>, id: TreeStorageId) {
        *self.engine.write().unwrap() = Some((id, engine));
    }
}

impl RenderNode for ModelLayer {}

// Convertion helpers

impl From<&ModelLayer> for Layer {
    fn from(model: &ModelLayer) -> Self {
        let size = model.size.value();
        let background_color = model.background_color.value();
        let border_color = model.border_color.value();
        let border_width = model.border_width.value();
        let border_corner_radius = model.border_corner_radius.value();
        let shadow_offset = model.shadow_offset.value();
        let shadow_radius = model.shadow_radius.value();
        let shadow_spread = model.shadow_spread.value();
        let shadow_color = model.shadow_color.value();
        let matrix = model.transform();
        let content = model.content.clone().map(|image| SkiaImage {
            data: Box::new(image),
        });

        Self {
            size,
            background_color,
            border_color,
            border_width,
            border_style: BorderStyle::Solid,
            border_corner_radius,
            shadow_offset,
            shadow_radius,
            shadow_color,
            shadow_spread,
            matrix,
            content,
            blend_mode: model.blend_mode.clone(),
        }
    }
}

impl From<Layer> for ModelLayer {
    fn from(layer: Layer) -> Self {
        let size = layer.size;
        let background_color = layer.background_color;
        let border_color = layer.border_color;
        let border_corner_radius = layer.border_corner_radius;
        let border_width = layer.border_width;
        let shadow_offset = layer.shadow_offset;
        let shadow_radius = layer.shadow_radius;
        let shadow_spread = layer.shadow_spread;
        let shadow_color = layer.shadow_color;
        let matrix = M44::new_identity();
        let content = layer.content.map(|image| {
            let i = image.data;
            *i
        });
        let blend_mode = layer.blend_mode;

        let (x, y) = (
            layer.matrix.translate_x() as f64,
            layer.matrix.translate_y() as f64,
        );
        Self {
            anchor_point: SyncValue::new(Point { x: 0.0, y: 0.0 }),
            position: SyncValue::new(Point { x, y }),
            scale: SyncValue::new(Point { x: 1.0, y: 1.0 }),
            rotation: SyncValue::new(Point3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            size: SyncValue::new(size),
            background_color: SyncValue::new(background_color),
            border_color: SyncValue::new(border_color),
            border_corner_radius: SyncValue::new(border_corner_radius),
            border_width: SyncValue::new(border_width),
            shadow_offset: SyncValue::new(shadow_offset),
            shadow_radius: SyncValue::new(shadow_radius),
            shadow_spread: SyncValue::new(shadow_spread),
            shadow_color: SyncValue::new(shadow_color),
            content,
            matrix,
            blend_mode,
            engine: RwLock::new(None),
        }
    }
}

impl From<ModelLayer> for Arc<dyn RenderNode> {
    fn from(model: ModelLayer) -> Self {
        Arc::new(model)
    }
}
