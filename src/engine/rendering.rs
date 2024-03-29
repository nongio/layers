use crate::prelude::BlendMode;
use crate::prelude::BorderRadius;
use crate::types::Rectangle;

use skia_safe::{Canvas, Matrix};

/// A trait for objects that can be drawn to a canvas.
pub trait Drawable {
    /// Draws the entity on the canvas.
    fn draw(&self, canvas: &mut Canvas);
    /// Returns the area that this drawable occupies.
    fn bounds(&self) -> Rectangle;
    /// Returns the transformation matrix for this drawable.
    fn transform(&self) -> Matrix;
    /// Returns the opacity of this drawable.
    fn opacity(&self) -> f32;
    /// Returns the blend mode of this drawable.
    fn blend_mode(&self) -> BlendMode;
    /// Returns the border corner radius of this drawable.
    fn border_corner_radius(&self) -> BorderRadius;
}
