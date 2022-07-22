use skia_safe::{Canvas, Image, Matrix, M44};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::easing::Interpolable;
use crate::ecs::animations::*;
use crate::ecs::entities::HasId;
use crate::layers::*;
use crate::rendering::{draw_layer, Drawable};
use crate::types::*;

#[derive(Clone, PartialEq, Debug)]
pub enum BlendMode {
    Normal,
    BackgroundBlur,
}
#[derive(Clone, Debug)]
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
    pub content: Option<Image>,
    pub blend_mode: BlendMode,
}

static OBJECT_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub struct ModelLayer {
    id: usize,
    pub anchor_point: AnimatedValue<Point>,
    pub position: AnimatedValue<Point>,
    pub scale: AnimatedValue<Point>,
    pub rotation: AnimatedValue<Point3d>,
    pub size: AnimatedValue<Point>,
    pub background_color: AnimatedValue<PaintColor>,
    pub border_corner_radius: AnimatedValue<BorderRadius>,
    pub border_color: AnimatedValue<PaintColor>,
    pub border_width: AnimatedValue<f64>,
    pub shadow_offset: AnimatedValue<Point>,
    pub shadow_radius: AnimatedValue<f64>,
    pub shadow_spread: AnimatedValue<f64>,
    pub shadow_color: AnimatedValue<Color>,
    pub matrix: M44,

    pub content: Option<Image>,
    pub blend_mode: BlendMode,
}
macro_rules! change_attr {
    ($variable_name:ident, $type:ty) => {
        pub fn $variable_name(
            &self,
            value: $type,
            transition: Option<Transition<Easing>>,
        ) -> Arc<ModelChange<$type>> {
            Arc::new(self.change(self.$variable_name.to(value, transition)))
        }
    };
}

impl ModelLayer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn change<T: Interpolable + Sync>(&self, change: ValueChange<T>) -> ModelChange<T> {
        ModelChange {
            id: self.id,
            value_change: change,
            need_repaint: true,
        }
    }

    change_attr!(position, Point);
    change_attr!(size, Point);
    change_attr!(scale, Point);
}

impl Default for ModelLayer {
    fn default() -> Self {
        let id = OBJECT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let position = AnimatedValue::new(Point { x: 0.0, y: 0.0 });
        let size = AnimatedValue::new(Point { x: 100.0, y: 100.0 });
        let anchor_point = AnimatedValue::new(size.value() * 0.5);
        let scale = AnimatedValue::new(Point { x: 1.0, y: 1.0 });
        let rotation = AnimatedValue::new(Point3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let background_color = AnimatedValue::new(PaintColor::Solid {
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        });
        let border_corner_radius = AnimatedValue::new(BorderRadius::new_single(20.0));
        let border_color = AnimatedValue::new(PaintColor::Solid {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        });
        let border_width = AnimatedValue::new(0.0);
        let matrix = M44::new_identity();
        let shadow_offset = AnimatedValue::new(Point { x: 0.0, y: 0.0 });
        let shadow_radius = AnimatedValue::new(0.0);
        let shadow_spread = AnimatedValue::new(0.0);
        let shadow_color = AnimatedValue::new(Color::new(0.0, 0.0, 0.0, 1.0));
        let content = None;
        let blend_mode = BlendMode::Normal;
        Self {
            id,
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
        }
    }
}

impl Drawable for ModelLayer {
    fn draw(&self, ctx: &mut Canvas) {
        let layer: Layer = Layer::from(self.clone());
        draw_layer(ctx, &layer)
    }
    fn bounds(&self) -> Rectangle {
        let p = self.position.value.clone();
        let p = p.read().unwrap();
        let s = self.size.value.clone();
        let s = s.read().unwrap();
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

impl HasId for ModelLayer {
    fn id(&self) -> usize {
        self.id
    }
}

impl From<ModelLayer> for Layer {
    fn from(model: ModelLayer) -> Self {
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
        let content = model.content;

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
            blend_mode: model.blend_mode,
        }
    }
}

impl From<Layer> for ModelLayer {
    fn from(layer: Layer) -> Self {
        let id = OBJECT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
        let content = layer.content;
        let blend_mode = layer.blend_mode;

        let (x, y) = (
            layer.matrix.translate_x() as f64,
            layer.matrix.translate_y() as f64,
        );
        Self {
            id,
            anchor_point: AnimatedValue::new(Point { x: 0.0, y: 0.0 }),
            position: AnimatedValue::new(Point { x, y }),
            scale: AnimatedValue::new(Point { x: 1.0, y: 1.0 }),
            rotation: AnimatedValue::new(Point3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            size: AnimatedValue::new(size),
            background_color: AnimatedValue::new(background_color),
            border_color: AnimatedValue::new(border_color),
            border_corner_radius: AnimatedValue::new(border_corner_radius),
            border_width: AnimatedValue::new(border_width),
            shadow_offset: AnimatedValue::new(shadow_offset),
            shadow_radius: AnimatedValue::new(shadow_radius),
            shadow_spread: AnimatedValue::new(shadow_spread),
            shadow_color: AnimatedValue::new(shadow_color),
            content,
            matrix,
            blend_mode,
        }
    }
}
