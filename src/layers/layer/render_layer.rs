use derive_builder::*;

use crate::{
    engine::rendering::Drawable,
    types::{BlendMode, Color, Point, *},
};

use super::model::ModelLayer;

#[derive(Clone, Debug, Builder)]
#[builder(public, default)]
#[repr(C)]
pub struct RenderLayer {
    pub background_color: PaintColor,
    pub border_color: PaintColor,
    pub border_width: f32,
    pub border_style: BorderStyle,
    pub border_corner_radius: BorderRadius,
    pub size: Point,
    pub shadow_offset: Point,
    pub shadow_radius: f32,
    pub shadow_color: Color,
    pub shadow_spread: f32,
    pub matrix: Matrix,
    pub content: Option<Image>,
    pub blend_mode: BlendMode,
    pub opacity: f32,
}

impl From<&ModelLayer> for RenderLayer {
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
        let content = model.content.value();
        let opacity = model.opacity.value();
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
            opacity,
        }
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self {
            background_color: PaintColor::Solid {
                color: Color::new_rgba(1.0, 1.0, 1.0, 1.0),
            },
            border_color: PaintColor::Solid {
                color: Color::new_rgba(0.0, 0.0, 0.0, 1.0),
            },
            border_width: 0.0,
            border_style: BorderStyle::Solid,
            border_corner_radius: BorderRadius::new_single(0.0),
            size: Point { x: 100.0, y: 100.0 },
            shadow_offset: Point { x: 0.0, y: 0.0 },
            shadow_radius: 0.0,
            shadow_color: Color::new_rgba(0.0, 0.0, 0.0, 1.0),
            shadow_spread: 0.0,
            matrix: Matrix::new_identity(),
            content: None,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
        }
    }
}
