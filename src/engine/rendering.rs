use crate::prelude::BlendMode;
use crate::prelude::BorderRadius;

use indextree::Arena;
use skia_safe::{Canvas, Matrix};

use super::SceneNode;

/// A trait for objects that can be drawn to a canvas.
pub(crate) trait Drawable {
    /// Draws the entity on the canvas.
    fn draw(&self, canvas: &Canvas, arena: &Arena<SceneNode>) -> skia_safe::Rect;
    /// Returns the area that this drawable occupies.
    fn bounds(&self) -> skia_safe::Rect;
    /// Returns the transformation matrix for this drawable.
    fn transform(&self) -> Matrix;
    /// Returns the opacity of this drawable.
    fn opacity(&self) -> f32;
    /// Returns the blend mode of this drawable.
    fn blend_mode(&self) -> BlendMode;
    /// Returns the border corner radius of this drawable.
    fn border_corner_radius(&self) -> BorderRadius;
}
