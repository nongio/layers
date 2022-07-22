use skia_safe::M44;

use crate::types::{BorderStyle, PaintColor, Point};

#[derive(Clone, Debug)]
pub struct Text {
    pub background_color: PaintColor,
    pub border_color: PaintColor,
    pub border_width: f64,
    pub border_style: BorderStyle,
    pub matrix: M44,
    pub size: Point,
    pub text: String,
}
