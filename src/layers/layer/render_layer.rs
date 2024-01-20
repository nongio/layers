use crate::types::{BlendMode, Color, Point, *};

use super::model::ModelLayer;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct RenderLayer {
    pub bounds: Rectangle,
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
    pub transform: Matrix,
    pub content: Option<Picture>,
    pub blend_mode: BlendMode,
    pub opacity: f32,
}

impl RenderLayer {
    #![allow(unused_variables)]
    pub fn from_model_and_layout(model: &ModelLayer, layout: &taffy::layout::Layout) -> Self {
        let layout_position = layout.location;
        let model_position = model.position.value();
        let position = Point {
            x: layout_position.x + model_position.x,
            y: layout_position.y + model_position.y,
        };
        let size = Point {
            x: layout.size.width,
            y: layout.size.height,
        };
        let bounds = Rectangle {
            x: position.x,
            y: position.y,
            width: size.x,
            height: size.y,
        };

        let rotation = model.rotation.value();
        let anchor_point = model.anchor_point.value();
        let scale = model.scale.value();
        let anchor_translate =
            M44::translate(-anchor_point.x * size.x, -anchor_point.y * size.y, 0.0);
        let identity = M44::new_identity();
        let translate = M44::translate(position.x, position.y, 0.0);
        let scale = M44::scale(scale.x, scale.y, 1.0);
        let rotate_x = M44::rotate(
            V3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            rotation.x,
        );
        let rotate_y = M44::rotate(
            V3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            rotation.y,
        );
        let rotate_z = M44::rotate(
            V3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            rotation.z,
        );
        // merge all transforms keeping into account the anchor point
        let transform = M44::concat(&translate, &identity);
        // let transform = M44::concat(&transform, &scale);
        // let transform = M44::concat(&transform, &rotate_x);
        // let transform = M44::concat(&transform, &rotate_y);
        // let transform = M44::concat(&transform, &rotate_z);
        // let transform = M44::concat(&transform, &anchor_translate);

        let matrix = transform.to_m33();

        let background_color = model.background_color.value();
        let border_color = model.border_color.value();
        let border_width = model.border_width.value();
        let border_corner_radius = model.border_corner_radius.value();
        let shadow_offset = model.shadow_offset.value();
        let shadow_radius = model.shadow_radius.value();
        let shadow_spread = model.shadow_spread.value();
        let shadow_color = model.shadow_color.value();

        let mut content = None;
        if let Some(draw_func) = model.draw_content.read().unwrap().as_ref() {
            let mut recorder = skia_safe::PictureRecorder::new();
            let canvas = recorder.begin_recording(skia_safe::Rect::from_wh(size.x, size.y), None);
            let caller = draw_func.0.clone();
            caller(canvas, size.x, size.y);
            content = recorder.finish_recording_as_picture(None);
        }

        let opacity = model.opacity.value();
        let blend_mode = model.blend_mode.value();

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
            transform: matrix,
            content,
            blend_mode,
            opacity,
            bounds,
        }
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self {
            background_color: PaintColor::Solid {
                color: Color::new_rgba(0.0, 0.0, 0.0, 0.0),
            },
            border_color: PaintColor::Solid {
                color: Color::new_rgba(0.0, 0.0, 0.0, 0.0),
            },
            border_width: 0.0,
            border_style: BorderStyle::Solid,
            border_corner_radius: BorderRadius::new_single(0.0),
            size: Point { x: 0.0, y: 0.0 },
            shadow_offset: Point { x: 0.0, y: 0.0 },
            shadow_radius: 0.0,
            shadow_color: Color::new_rgba(0.0, 0.0, 0.0, 0.0),
            shadow_spread: 0.0,
            transform: Matrix::new_identity(),
            content: None,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
            bounds: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        }
    }
}
