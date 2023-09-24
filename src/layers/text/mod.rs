use skia_safe::{Canvas, Matrix, M44};
use std::sync::{Arc, RwLock};
use taffy::prelude::Node;

use crate::{
    drawing::text::draw_text,
    engine::{
        node::{RenderNode, RenderableFlags},
        rendering::Drawable,
        Engine, NodeRef, TransactionRef,
    },
    types::*,
};

use crate::engine::{animation::*, command::*};
#[derive(Clone, Debug)]
pub struct RenderText {
    pub matrix: Matrix,
    pub size: Point,
    pub text: String,
    pub text_color: Color,
    pub background_color: PaintColor,
    pub font_size: f32,
    pub font_family: String,
    pub font_weight: f32,
    pub font_style: String,
    pub font_letter_spacing: f32,
}

pub struct ModelText {
    pub position: Attribute<Point>,
    pub scale: Attribute<Point>,
    pub size: Attribute<Point>,
    pub background_color: Attribute<PaintColor>,
    pub text_color: Attribute<Color>,
    pub font_family: String,
    pub font_size: Attribute<f32>,

    pub font_weight: Attribute<f32>,
    pub font_letter_spacing: Attribute<f32>,
    pub text: RwLock<String>,
}

impl ModelText {
    fn new() -> Self {
        Default::default()
    }
    pub fn create() -> Arc<Self> {
        Arc::new(Self::new())
    }
}
impl Default for ModelText {
    fn default() -> Self {
        let position = Attribute::new(Point { x: 0.0, y: 0.0 });
        let size = Attribute::new(Point { x: 100.0, y: 100.0 });
        let scale = Attribute::new(Point { x: 1.0, y: 1.0 });

        let background_color = Attribute::new(PaintColor::Solid {
            color: Color::new_rgba(1.0, 1.0, 1.0, 1.0),
        });

        let text_color = Attribute::new(Color::new_rgba(0.0, 0.0, 0.0, 1.0));
        let font_family = "Noto Sans".to_string();
        let font_size = Attribute::new(22.0);
        let font_weight = Attribute::new(400.0);
        let font_letter_spacing = Attribute::new(0.0);
        let text = RwLock::new(String::from("Hello World"));

        Self {
            position,
            scale,
            size,
            background_color,
            text_color,
            font_family,
            font_size,
            font_weight,
            font_letter_spacing,
            text,
        }
    }
}

impl Drawable for ModelText {
    fn draw(&self, canvas: &mut Canvas) {
        let text = RenderText::from(self);
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
    fn scaled_bounds(&self) -> Rectangle {
        let s = self.size.value();
        let scale = self.scale.value();

        Rectangle {
            x: 0.0,
            y: 0.0,
            width: s.x * scale.x,
            height: s.y * scale.y,
        }
    }
    fn transform(&self) -> Matrix {
        let s = self.scale.value();
        let p = self.position.value();
        let translate = M44::translate(p.x, p.y, 0.0);
        let scale = M44::scale(s.x, s.y, 1.0);
        // let rotate = M44::rotate(
        //     V3 {
        //         x: 0.0,
        //         y: 1.0,
        //         z: 0.0,
        //     },
        //     (p.x / 100.0),
        // );
        let transform = skia_safe::M44::concat(&translate, &scale);
        // let transform = skia_safe::M44::concat(&transform, &rotate);

        transform.to_m33()
    }
    fn scale(&self) -> (f32, f32) {
        let s = self.scale.value();
        (s.x, s.y)
    }
}

impl RenderNode for ModelText {}

impl From<&ModelText> for RenderText {
    fn from(mt: &ModelText) -> Self {
        let matrix = mt.transform();
        let size = mt.size.value();
        let text = mt.text.read().unwrap().clone();
        let text_color = mt.text_color.value();
        let background_color = mt.background_color.value();
        let font_size = mt.font_size.value();
        let font_family = mt.font_family.clone();
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

#[derive(Clone)]
pub struct TextLayer {
    engine: Arc<Engine>,
    pub id: Arc<RwLock<Option<NodeRef>>>,
    pub model: Arc<ModelText>,
    pub layout: Node,
}

impl TextLayer {
    pub fn set_id(&self, id: NodeRef) {
        self.id.write().unwrap().replace(id);
    }
    change_model!(position, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(
        size,
        Point,
        RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT
    );

    change_model!(
        font_size,
        f32,
        RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT
    );
    change_model!(
        font_weight,
        f32,
        RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT
    );
}
