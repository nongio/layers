use super::model::{ContentDrawFunctionInternal, ModelLayer};
use crate::types::{BlendMode, Color, Point, *};

use serde::{ser::SerializeStruct, Serialize};
use skia::{ColorFilter, ImageFilter};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct RenderLayer {
    /// A unique identifier for the layer
    pub key: String,
    /// The rectangle representing the bounds of the layer
    pub bounds: skia_safe::Rect,
    /// The rounded rectangle representing the bounds of the layer
    pub rbounds: skia_safe::RRect,
    /// The transformed bounds of the layer, relative to the parent
    pub local_transformed_bounds: skia_safe::Rect,
    /// The transformed bounds of the layer, relative to the parent, including children bounds
    pub local_transformed_bounds_with_children: skia_safe::Rect,
    /// The bounds of the layers, including children bounds
    pub bounds_with_children: skia_safe::Rect,
    /// The transformed bounds of the layer, relative to the root
    pub global_transformed_bounds: skia_safe::Rect,
    /// The transformed rounded bounds of the layer, relative to the root
    pub global_transformed_rbounds: skia_safe::RRect,
    /// The transformed bounds of the layer, including children bounds, relative to the root
    pub global_transformed_bounds_with_children: skia_safe::Rect,
    /// The background color of the layer
    pub background_color: PaintColor,
    /// The border color of the layer
    pub border_color: PaintColor,
    /// The border width of the layer
    pub border_width: f32,
    /// The border style of the layer
    pub border_style: BorderStyle,
    /// The border corner radius of the layer
    pub border_corner_radius: BorderRadius,
    /// The size of the layer
    pub size: skia_safe::Size,
    /// The shadow offset of the layer
    pub shadow_offset: Point,
    /// The shadow radius of the layer
    pub shadow_radius: f32,
    /// The shadow color of the layer
    pub shadow_color: Color,
    /// The shadow spread of the layer
    pub shadow_spread: f32,
    /// The transform of the layer relative to the root (4x4)
    pub transform: M44,
    /// The transform of the layer relative to the root (3x3)
    pub transform_33: Matrix,
    /// The transform of the layer relative to the parent (4x4)
    pub local_transform: M44,
    /// The blend mode of the layer
    pub blend_mode: BlendMode,
    /// The opacity of the layer, 0.0 is transparent, 1.0 is opaque
    pub opacity: f32,
    /// The premultiplied opacity of the layer, 0.0 is transparent, 1.0 is opaque
    pub premultiplied_opacity: f32,
    /// Is the content drawn clipped to the bounds of the layer
    pub clip_content: bool,
    /// Are the children drawn clipped to the bounds of the layer
    pub clip_children: bool,
    /// Are the pointer events enabled for the layer
    pub pointer_events: bool,
    /// Whether the layer should be rendered (not hidden and has drawable content)
    pub visible: bool,
    pub content_draw_func: Option<ContentDrawFunctionInternal>,
    pub content: Option<Picture>,
    pub image_filter: Option<ImageFilter>,
    pub image_filter_bounds: Option<skia::Rect>,
    pub color_filter: Option<ColorFilter>,
}

impl RenderLayer {
    pub(crate) fn update_with_model_and_layout(
        &mut self,
        model: &ModelLayer,
        layout: &taffy::tree::Layout,
        context_transform: Option<&M44>,
        context_opacity: f32,
    ) {
        let key = model.key.read().unwrap().clone();
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

        let anchor_point = model.anchor_point.value();
        let scale = model.scale.value();
        let anchor_translate = M44::translate(
            -anchor_point.x * size.width,
            -anchor_point.y * size.height,
            0.0,
        );
        let identity = M44::new_identity();
        let matrix = context_transform.unwrap_or(&identity);
        let translate = M44::translate(position.x, position.y, 0.0);
        let scale = M44::scale(scale.x, scale.y, 1.0);

        let rotation = model.rotation.value();
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
        let mut local_transform = M44::new_identity();
        local_transform = M44::concat(&local_transform, &translate);
        local_transform = M44::concat(&local_transform, &scale);
        let transform = M44::concat(&local_transform, &rotate_x);
        let transform = M44::concat(&transform, &rotate_y);
        let transform = M44::concat(&transform, &rotate_z);
        local_transform = M44::concat(&transform, &anchor_translate);

        let global_transform = M44::concat(matrix, &local_transform);
        let (transformed_bounds, _) = global_transform.to_m33().map_rect(bounds);
        let (local_transformed_bounds, _) = local_transform.to_m33().map_rect(bounds);
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

        // FIXME: cache content
        // if cache_content {
        // if content_draw_func.is_some()
        //     && ((self.size != size) || (self.content_draw_func.as_ref() != content_draw_func))
        // {
        //     let mut recorder = skia_safe::PictureRecorder::new();
        //     let canvas =
        //         recorder.begin_recording(skia_safe::Rect::from_wh(size.width, size.height), None);
        //     let draw_func = content_draw_func.unwrap();
        //     let caller = draw_func.0.as_ref();
        //     let content_damage = caller(canvas, size.width, size.height);
        //     self.content_damage = content_damage;
        //     self.content = recorder.finish_recording_as_picture(None);
        // }
        // } else {
        //     self.content = None;
        if let Some(draw_func) = content_draw_func {
            // let caller = draw_func.0.as_ref();
            self.content_draw_func = Some(draw_func.clone());
        }
        // }

        self.key = key;
        self.size = size;
        self.background_color = background_color;
        self.border_color = border_color;
        self.border_width = border_width;

        self.border_corner_radius = border_corner_radius;
        self.shadow_offset = shadow_offset;
        self.shadow_radius = shadow_radius;
        self.shadow_color = shadow_color;
        self.shadow_spread = shadow_spread;
        self.transform = global_transform;
        self.transform_33 = self.transform.to_m33();
        self.local_transform = local_transform;
        self.blend_mode = blend_mode;
        self.opacity = opacity;
        self.premultiplied_opacity = opacity * context_opacity;
        self.bounds = bounds;
        self.rbounds = skia_safe::RRect::new_rect_radii(bounds, &border_corner_radius.into());
        self.bounds_with_children = bounds;
        self.local_transformed_bounds = local_transformed_bounds;
        self.local_transformed_bounds_with_children = local_transformed_bounds;
        self.global_transformed_bounds = transformed_bounds;
        self.global_transformed_bounds_with_children = transformed_bounds;
        self.global_transformed_rbounds =
            skia_safe::RRect::new_rect_radii(transformed_bounds, &border_corner_radius.into());

        self.clip_content = model.clip_content.value();
        self.clip_children = model.clip_children.value();
        self.pointer_events = model
            .pointer_events
            .load(std::sync::atomic::Ordering::Relaxed);

        self.image_filter = model.image_filter.value();
        self.image_filter_bounds = *model.filter_bounds.read().unwrap();
        self.color_filter = model.color_filter.value();
    }

    pub(crate) fn has_visible_drawables(&self) -> bool {
        if self.premultiplied_opacity <= 0.0 {
            return false;
        }

        let draws_background = Self::paint_color_max_alpha(&self.background_color) > 0.0;
        let draws_border =
            self.border_width > 0.0 && Self::paint_color_max_alpha(&self.border_color) > 0.0;
        let draws_shadow = self.shadow_color.alpha > 0.0;
        let draws_content = self.content.is_some() || self.content_draw_func.is_some();

        draws_background || draws_border || draws_shadow || draws_content
    }

    fn paint_color_max_alpha(color: &PaintColor) -> f32 {
        match color {
            PaintColor::Solid { color } => color.alpha,
            PaintColor::GradientLinear(gradient) => {
                gradient.colors.iter().fold(0.0, |acc, c| acc.max(c.alpha))
            }
            PaintColor::GradientRadial(gradient) => {
                gradient.colors.iter().fold(0.0, |acc, c| acc.max(c.alpha))
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_model_and_layout(
        model: &ModelLayer,
        layout: &taffy::tree::Layout,
        matrix: Option<&M44>,
        context_opacity: f32,
    ) -> Self {
        let key = model.key.read().unwrap().clone();
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
        let border_corner_radius = model.border_corner_radius.value();
        let rbounds = skia_safe::RRect::new_rect_radii(bounds, &border_corner_radius.into());
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
        let mut local_transform = translate;
        local_transform = M44::concat(&local_transform, &scale);
        let transform = M44::concat(matrix, &local_transform);
        let transform = M44::concat(&transform, &rotate_x);
        let transform = M44::concat(&transform, &rotate_y);
        let transform = M44::concat(&transform, &rotate_z);
        let transform = M44::concat(&transform, &anchor_translate);

        let transform_33 = transform.to_m33();
        // let matrix = transform.to_m33();
        let transform = M44::concat(matrix, &local_transform);
        let (transformed_bounds, _) = transform.to_m33().map_rect(bounds);
        let (local_transformed_bounds, _) = local_transform.to_m33().map_rect(bounds);
        let transformed_rbounds =
            skia_safe::RRect::new_rect_radii(transformed_bounds, &border_corner_radius.into());
        let background_color = model.background_color.value();
        let border_color = model.border_color.value();

        let shadow_offset = model.shadow_offset.value();
        let shadow_radius = model.shadow_radius.value();
        let shadow_spread = model.shadow_spread.value();
        let shadow_color = model.shadow_color.value();

        let content = None;
        let mut content_draw_func = None;
        if let Some(draw_func) = model.draw_content.read().unwrap().as_ref() {
            // let mut recorder = skia_safe::PictureRecorder::new();
            // let canvas =
            // recorder.begin_recording(skia_safe::Rect::from_wh(size.width, size.height), None);
            // let caller = draw_func.0.clone();
            // caller(canvas, size.width, size.height, arena);
            // content = recorder.finish_recording_as_picture(None);
            content_draw_func = Some(draw_func.clone());
        }

        let opacity = model.opacity.value();
        let premultiplied_opacity = opacity * context_opacity;
        let blend_mode = model.blend_mode.value();
        let clip_content = model.clip_content.value();
        let clip_children = model.clip_children.value();

        let mut render_layer = Self {
            key,
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
            local_transform,
            transform,
            transform_33,
            content,
            blend_mode,
            opacity,
            premultiplied_opacity,
            bounds,
            bounds_with_children: bounds,
            local_transformed_bounds,
            local_transformed_bounds_with_children: local_transformed_bounds,
            global_transformed_bounds: transformed_bounds,
            global_transformed_bounds_with_children: transformed_bounds,
            content_draw_func,
            rbounds,
            global_transformed_rbounds: transformed_rbounds,
            clip_content,
            clip_children,
            image_filter: model.image_filter.value(),
            image_filter_bounds: *model.filter_bounds.read().unwrap(),
            color_filter: model.color_filter.value(),
            pointer_events: model
                .pointer_events
                .load(std::sync::atomic::Ordering::Relaxed),
            visible: true,
        };

        render_layer.visible = render_layer.has_visible_drawables();
        render_layer
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self {
            key: "".to_string(),
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
            transform_33: Matrix::default(),
            local_transform: M44::new_identity(),
            content: None,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
            premultiplied_opacity: 1.0,
            bounds: skia_safe::Rect::default(),
            rbounds: skia_safe::RRect::default(),
            local_transformed_bounds: skia_safe::Rect::default(),
            local_transformed_bounds_with_children: skia_safe::Rect::default(),
            bounds_with_children: skia_safe::Rect::default(),
            global_transformed_bounds: skia_safe::Rect::default(),
            global_transformed_bounds_with_children: skia_safe::Rect::default(),
            global_transformed_rbounds: skia_safe::RRect::default(),
            content_draw_func: None,
            clip_content: false,
            clip_children: false,
            image_filter: None,
            image_filter_bounds: None,
            color_filter: None,
            pointer_events: false,
            visible: false,
        }
    }
}

impl Serialize for RenderLayer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_struct("RenderLayer", 17)?;
        // let mut seq = serializer.serialize_seq(Some(15))?;
        // seq.serialize_element(&Rectangle::from(self.rbounds))?;
        // seq.serialize_element(&self.transformed_rbounds.into())?;
        seq.serialize_field("key", &self.key)?;
        seq.serialize_field("bounds", &Rectangle::from(self.bounds))?;
        seq.serialize_field(
            "transformed_bounds",
            &Rectangle::from(self.global_transformed_bounds),
        )?;
        seq.serialize_field(
            "bounds_with_children",
            &Rectangle::from(self.global_transformed_bounds_with_children),
        )?;
        seq.serialize_field("background_color", &self.background_color)?;
        seq.serialize_field("border_color", &self.border_color)?;
        seq.serialize_field("border_width", &self.border_width)?;
        seq.serialize_field("border_style", &self.border_style)?;
        seq.serialize_field("border_corner_radius", &self.border_corner_radius)?;
        seq.serialize_field("size", &crate::types::Size::from(self.size))?;
        seq.serialize_field("shadow_offset", &self.shadow_offset)?;
        seq.serialize_field("shadow_radius", &self.shadow_radius)?;
        seq.serialize_field("shadow_color", &self.shadow_color)?;
        seq.serialize_field("shadow_spread", &self.shadow_spread)?;
        seq.serialize_field("blend_mode", &self.blend_mode)?;
        seq.serialize_field("opacity", &self.opacity)?;
        seq.serialize_field("visible", &self.visible)?;
        // seq.serialize_element(&self.content)?;
        // seq.serialize_element(&self.transform)?;
        seq.end()
    }
}
