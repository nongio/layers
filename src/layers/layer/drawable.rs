use indextree::Arena;
use skia_safe::Canvas;

use crate::{drawing::layer::draw_layer, engine::{rendering::Drawable, SceneNode}, types::*};

use super::render_layer::RenderLayer;

impl Drawable for RenderLayer {
    fn draw(&self, canvas: &Canvas, arena: &Arena<SceneNode>) -> skia_safe::Rect {
        draw_layer(canvas, self, 1.0, arena)
    }
    fn bounds(&self) -> skia_safe::Rect {
        self.bounds
    }
    fn transform(&self) -> Matrix {
        self.transform.to_m33()
    }
    fn opacity(&self) -> f32 {
        self.opacity
    }
    fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }
    fn border_corner_radius(&self) -> BorderRadius {
        self.border_corner_radius
    }
}
