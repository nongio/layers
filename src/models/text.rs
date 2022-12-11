use skia_safe::{Canvas, Matrix, M44};
use std::sync::{Arc, RwLock};

use crate::{
    drawing::layer::draw_text,
    engine::{
        animations::SyncValue,
        node::{RenderNode, RenderableFlags},
        rendering::Drawable,
        ChangeProducer, Engine,
    },
    types::{Color, PaintColor, Point, Rectangle},
};

use super::change_attr;
use crate::engine::{animations::*, command::*};
use crate::engine::{NodeRef, TransactionRef};
#[derive(Clone, Debug)]
pub struct Text {
    pub matrix: Matrix,
    pub size: Point,
    pub text: String,
    pub text_color: Color,
    pub background_color: PaintColor,
    pub font_size: f64,
    pub font_family: String,
    pub font_weight: f64,
    pub font_style: String,
    pub font_letter_spacing: f64,
}

pub struct ModelText {
    pub position: SyncValue<Point>,
    pub scale: SyncValue<Point>,
    pub size: SyncValue<Point>,
    pub background_color: SyncValue<PaintColor>,
    pub text_color: SyncValue<Color>,
    pub font_size: SyncValue<f64>,

    pub font_weight: SyncValue<f64>,
    pub font_letter_spacing: SyncValue<f64>,
    pub text: RwLock<String>,
    pub engine: RwLock<Option<(NodeRef, Arc<Engine>)>>,
}

impl ModelText {
    fn new() -> Self {
        Default::default()
    }
    pub fn create() -> Arc<Self> {
        Arc::new(Self::new())
    }

    change_attr!(position, Point, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(
        size,
        Point,
        RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT
    );

    change_attr!(
        font_size,
        f64,
        RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT
    );
}
impl Default for ModelText {
    fn default() -> Self {
        let position = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let size = SyncValue::new(Point { x: 100.0, y: 100.0 });
        let scale = SyncValue::new(Point { x: 1.0, y: 1.0 });

        let background_color = SyncValue::new(PaintColor::Solid {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        });

        let text_color = SyncValue::new(Color::new(0.0, 0.0, 0.0, 1.0));
        let font_size = SyncValue::new(22.0);
        let font_weight = SyncValue::new(400.0);
        let font_letter_spacing = SyncValue::new(0.0);
        let text = RwLock::new(String::from("Hello World"));

        let engine = RwLock::new(None);
        Self {
            position,
            scale,
            size,
            background_color,
            text_color,
            font_size,
            font_weight,
            font_letter_spacing,
            text,
            engine,
        }
    }
}

impl Drawable for ModelText {
    fn draw(&self, canvas: &mut Canvas) {
        let text: Text = Text::from(self);
        draw_text(canvas, &text);
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

impl ChangeProducer for ModelText {
    fn set_engine(&self, engine: Arc<Engine>, id: NodeRef) {
        *self.engine.write().unwrap() = Some((id, engine));
    }
}

impl RenderNode for ModelText {}

// Conversion helpers

impl From<&ModelText> for Text {
    fn from(mt: &ModelText) -> Self {
        let matrix = mt.transform();
        let size = mt.size.value();
        let text = mt.text.read().unwrap().clone();
        let text_color = mt.text_color.value();
        let background_color = mt.background_color.value();
        let font_size = mt.font_size.value();
        let font_family = "Noto Sans".to_string();
        let font_weight = mt.font_weight.value();
        let font_style = "normal".to_string();
        let font_letter_spacing = mt.font_letter_spacing.value();

        Self {
            matrix,
            size,
            text,
            text_color,
            background_color,
            font_size,
            font_family,
            font_weight,
            font_style,
            font_letter_spacing,
        }
    }
}
