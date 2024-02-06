use crate::types::{BlendMode, Color, Point, *};

use super::model::{ContentDrawFunction, ModelLayer};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct RenderLayer {
    pub bounds: skia_safe::Rect,
    pub transformed_bounds: skia_safe::Rect,
    pub bounds_with_children: skia_safe::Rect,
    pub background_color: PaintColor,
    pub border_color: PaintColor,
    pub border_width: f32,
    pub border_style: BorderStyle,
    pub border_corner_radius: BorderRadius,
    pub size: skia_safe::Size,
    pub shadow_offset: Point,
    pub shadow_radius: f32,
    pub shadow_color: Color,
    pub shadow_spread: f32,
    pub transform: M44,
    pub blend_mode: BlendMode,
    pub opacity: f32,
    content_draw_func: Option<ContentDrawFunction>,
    pub content_damage: skia_safe::Rect,
    pub content: Option<Picture>,
}

impl RenderLayer {
    #![allow(unused_variables, dead_code)]
    pub fn update_with_model_and_layout(
        &mut self,
        model: &ModelLayer,
        layout: &taffy::layout::Layout,
        matrix: Option<&M44>,
    ) {
        let layout_position = layout.location;
        let model_position = model.position.value();
        let position = Point {
            x: layout_position.x + model_position.x,
            y: layout_position.y + model_position.y,
        };
        // self.position = position;

        let size = skia_safe::Size {
            width: layout.size.width,
            height: layout.size.height,
        };

        let border_width = model.border_width.value();

        let bounds = skia_safe::Rect::from_xywh(0.0, 0.0, size.width, size.height);

        let rotation = model.rotation.value();
        let anchor_point = model.anchor_point.value();
        let scale = model.scale.value();
        let anchor_translate = M44::translate(
            -anchor_point.x * size.width,
            -anchor_point.y * size.height,
            0.0,
        );
        let identity = M44::new_identity();
        let matrix = matrix.unwrap_or(&identity);
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
        let transform = M44::concat(matrix, &translate);
        let transform = M44::concat(&transform, &scale);
        // let transform = M44::concat(&transform, &rotate_x);
        // let transform = M44::concat(&transform, &rotate_y);
        // let transform = M44::concat(&transform, &rotate_z);
        // let transform = M44::concat(&transform, &anchor_translate);

        // let matrix = transform.to_m33();
        let (transformed_bounds, _) = transform.to_m33().map_rect(bounds);
        let bounds_with_children = transformed_bounds;
        let background_color = model.background_color.value();
        let border_color = model.border_color.value();

        let border_corner_radius = model.border_corner_radius.value();
        let shadow_offset = model.shadow_offset.value();
        let shadow_radius = model.shadow_radius.value();
        let shadow_spread = model.shadow_spread.value();
        let shadow_color = model.shadow_color.value();

        let opacity = model.opacity.value();
        let blend_mode = model.blend_mode.value();

        let content_draw_func = model.draw_content.read().unwrap();
        let content_draw_func = content_draw_func.as_ref();
        if self.size != size || self.content_draw_func.as_ref() != content_draw_func {
            if let Some(draw_func) = content_draw_func {
                let mut recorder = skia_safe::PictureRecorder::new();
                let canvas = recorder
                    .begin_recording(skia_safe::Rect::from_wh(size.width, size.height), None);
                let caller = draw_func.0.clone();
                let content_damage = caller(canvas, size.width, size.height);
                self.content_damage = content_damage;
                self.content = recorder.finish_recording_as_picture(None);
                self.content_draw_func = Some(draw_func.clone());
            }
        }
        self.size = size;
        self.background_color = background_color;
        self.border_color = border_color;
        self.border_width = border_width;

        self.border_corner_radius = border_corner_radius;
        self.shadow_offset = shadow_offset;
        self.shadow_radius = shadow_radius;
        self.shadow_color = shadow_color;
        self.shadow_spread = shadow_spread;
        self.transform = transform;
        self.blend_mode = blend_mode;
        self.opacity = opacity;
        self.bounds = bounds;
        self.transformed_bounds = transformed_bounds;
        self.bounds_with_children = bounds_with_children;
    }

    pub fn from_model_and_layout(
        model: &ModelLayer,
        layout: &taffy::layout::Layout,
        matrix: Option<&M44>,
    ) -> Self {
        let layout_position = layout.location;
        let model_position = model.position.value();
        let position = Point {
            x: layout_position.x + model_position.x,
            y: layout_position.y + model_position.y,
        };
        let size = skia_safe::Size {
            width: layout.size.width,
            height: layout.size.height,
        };

        let border_width = model.border_width.value();

        let bounds = skia_safe::Rect::from_xywh(0.0, 0.0, size.width, size.height);

        let rotation = model.rotation.value();
        let anchor_point = model.anchor_point.value();
        let scale = model.scale.value();
        let anchor_translate = M44::translate(
            -anchor_point.x * size.width,
            -anchor_point.y * size.height,
            0.0,
        );
        let identity = M44::new_identity();
        let matrix = matrix.unwrap_or(&identity);
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
        let transform = M44::concat(&translate, matrix);
        let transform = M44::concat(&transform, &scale);
        // let transform = M44::concat(&transform, &rotate_x);
        // let transform = M44::concat(&transform, &rotate_y);
        // let transform = M44::concat(&transform, &rotate_z);
        // let transform = M44::concat(&transform, &anchor_translate);

        // let matrix = transform.to_m33();
        let (transformed_bounds, _) = transform.to_m33().map_rect(bounds);
        let bounds_with_children = transformed_bounds;
        let background_color = model.background_color.value();
        let border_color = model.border_color.value();

        let border_corner_radius = model.border_corner_radius.value();
        let shadow_offset = model.shadow_offset.value();
        let shadow_radius = model.shadow_radius.value();
        let shadow_spread = model.shadow_spread.value();
        let shadow_color = model.shadow_color.value();

        let mut content = None;
        let mut content_draw_func = None;
        if let Some(draw_func) = model.draw_content.read().unwrap().as_ref() {
            let mut recorder = skia_safe::PictureRecorder::new();
            let canvas =
                recorder.begin_recording(skia_safe::Rect::from_wh(size.width, size.height), None);
            let caller = draw_func.0.clone();
            caller(canvas, size.width, size.height);
            content = recorder.finish_recording_as_picture(None);
            content_draw_func = Some(draw_func.clone());
        }

        let opacity = model.opacity.value();
        let blend_mode = model.blend_mode.value();
        let content_damage = skia_safe::Rect::default();
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
            transform,
            content,
            blend_mode,
            opacity,
            bounds,
            transformed_bounds,
            bounds_with_children,
            content_draw_func,
            content_damage,
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
            size: skia_safe::Size::default(),
            shadow_offset: Point { x: 0.0, y: 0.0 },
            shadow_radius: 0.0,
            shadow_color: Color::new_rgba(0.0, 0.0, 0.0, 0.0),
            shadow_spread: 0.0,
            transform: M44::new_identity(),
            content: None,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
            bounds: skia_safe::Rect::default(),
            transformed_bounds: skia_safe::Rect::default(),
            bounds_with_children: skia_safe::Rect::default(),
            content_draw_func: None,
            content_damage: skia_safe::Rect::default(),
        }
    }
}
