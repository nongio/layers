use skia_safe::{Bitmap, Pixmap};
// use skia_safe::gpu::BackendTexture;
// use skia_safe::image::CachingHint;
// use skia_safe::Bitmap;
// use skia_safe::Pixmap;
use skia_safe::{Canvas, Image, Matrix, M44, V3};
use taffy::prelude::Node;
use taffy::style::Style;
// use stretch::node::Node;

use std::sync::Arc;
use std::sync::RwLock;
// use std::sync::RwLock;

use crate::drawing::layer::draw_layer;
use crate::engine::node::RenderNode;
use crate::engine::node::RenderableFlags;
use crate::engine::rendering::Drawable;
use crate::engine::Engine;
use crate::engine::NodeRef;
// use crate::engine::{Engine};
use crate::layers::*;
use crate::types::*;

use super::change_model;

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum BlendMode {
    Normal,
    BackgroundBlur,
}

impl PartialEq for BlendMode {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Clone, Debug)]
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
}
// #[derive(Clone)]
pub(crate) struct ModelLayer {
    pub anchor_point: SyncValue<Point>,
    pub position: SyncValue<Point>,
    pub scale: SyncValue<Point>,
    pub rotation: SyncValue<Point3d>,
    pub size: SyncValue<Point>,
    pub background_color: SyncValue<PaintColor>,
    pub border_corner_radius: SyncValue<BorderRadius>,
    pub border_color: SyncValue<PaintColor>,
    pub border_width: SyncValue<f32>,
    pub shadow_offset: SyncValue<Point>,
    pub shadow_radius: SyncValue<f32>,
    pub shadow_spread: SyncValue<f32>,
    pub shadow_color: SyncValue<Color>,

    pub content: SyncValue<Option<Image>>,

    pub blend_mode: BlendMode,
}

impl ModelLayer {}

impl Default for ModelLayer {
    fn default() -> Self {
        let position = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let size = SyncValue::new(Point { x: 100.0, y: 100.0 });
        let anchor_point = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let scale = SyncValue::new(Point { x: 1.0, y: 1.0 });
        let rotation = SyncValue::new(Point3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let background_color = SyncValue::new(PaintColor::Solid {
            color: Color::new_rgba(1.0, 1.0, 1.0, 1.0),
        });
        let border_corner_radius = SyncValue::new(BorderRadius::new_single(0.0));
        let border_color = SyncValue::new(PaintColor::Solid {
            color: Color::new_rgba(0.0, 0.0, 0.0, 1.0),
        });
        let border_width = SyncValue::new(0.0);
        let shadow_offset = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let shadow_radius = SyncValue::new(0.0);
        let shadow_spread = SyncValue::new(0.0);
        let shadow_color = SyncValue::new(Color::new_rgba(0.0, 0.0, 0.0, 1.0));
        let content = SyncValue::new(None);
        let blend_mode = BlendMode::Normal;

        Self {
            anchor_point,
            position,
            scale,
            rotation,
            size,
            background_color,
            border_corner_radius,
            border_color,
            border_width,
            shadow_offset,
            shadow_radius,
            shadow_spread,
            shadow_color,
            content,
            blend_mode,
        }
    }
}

impl Drawable for ModelLayer {
    fn draw(&self, canvas: &mut Canvas) {
        let layer: RenderLayer = RenderLayer::from(self);
        draw_layer(canvas, &layer);
    }
    fn bounds(&self) -> Rectangle {
        let s = self.size.value();
        let p = self.position.value();
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
        let rotation = self.rotation.value();
        let anchor_point = self.anchor_point.value();
        let size = self.size.value();
        let anchor_translate =
            M44::translate(-anchor_point.x * size.x, -anchor_point.y * size.y, 0.0);
        let identity = M44::new_identity();
        let translate = M44::translate(p.x, p.y, 0.0);
        let _scale = M44::scale(s.x, s.y, 1.0);
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
        let transform = M44::concat(&transform, &rotate_x);
        let transform = M44::concat(&transform, &rotate_y);
        let transform = M44::concat(&transform, &rotate_z);
        let transform = M44::concat(&transform, &anchor_translate);

        transform.to_m33()
    }
    fn scale(&self) -> (f32, f32) {
        let s = self.scale.value();
        (s.x, s.y)
    }
}

impl RenderNode for ModelLayer {}

// Convertion helpers

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
        }
    }
}

impl From<RenderLayer> for ModelLayer {
    fn from(layer: RenderLayer) -> Self {
        let size = layer.size;
        let background_color = layer.background_color;
        let border_color = layer.border_color;
        let border_corner_radius = layer.border_corner_radius;
        let border_width = layer.border_width;
        let shadow_offset = layer.shadow_offset;
        let shadow_radius = layer.shadow_radius;
        let shadow_spread = layer.shadow_spread;
        let shadow_color = layer.shadow_color;
        let content = match layer.content {
            None => SyncValue::new(None),
            Some(image) => SyncValue::new(Some(image)),
        };

        let blend_mode = layer.blend_mode;

        let (x, y) = (layer.matrix.translate_x(), layer.matrix.translate_y());
        Self {
            anchor_point: SyncValue::new(Point { x: 0.0, y: 0.0 }),
            position: SyncValue::new(Point { x, y }),
            scale: SyncValue::new(Point { x: 1.0, y: 1.0 }),
            rotation: SyncValue::new(Point3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            size: SyncValue::new(size),
            background_color: SyncValue::new(background_color),
            border_color: SyncValue::new(border_color),
            border_corner_radius: SyncValue::new(border_corner_radius),
            border_width: SyncValue::new(border_width),
            shadow_offset: SyncValue::new(shadow_offset),
            shadow_radius: SyncValue::new(shadow_radius),
            shadow_spread: SyncValue::new(shadow_spread),
            shadow_color: SyncValue::new(shadow_color),
            content,
            blend_mode,
        }
    }
}

impl From<ModelLayer> for Arc<dyn RenderNode> {
    fn from(model: ModelLayer) -> Self {
        Arc::new(model)
    }
}

#[derive(Clone)]
pub struct Layer {
    pub(crate) engine: Arc<Engine>,
    pub id: Arc<RwLock<Option<NodeRef>>>,
    pub(crate) model: Arc<ModelLayer>,
    pub layout: Node,
}

impl Layer {
    pub fn set_id(&self, id: NodeRef) {
        self.id.write().unwrap().replace(id);
    }
    pub fn id(&self) -> Option<NodeRef> {
        let id = *self.id.read().unwrap();
        id
    }
    change_model!(position, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(background_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_model!(scale, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(rotation, Point3d, RenderableFlags::NEEDS_LAYOUT);
    change_model!(anchor_point, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(
        border_corner_radius,
        BorderRadius,
        RenderableFlags::NEEDS_PAINT
    );
    change_model!(border_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_model!(border_width, f32, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_offset, Point, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_radius, f32, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_spread, f32, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_color, Color, RenderableFlags::NEEDS_PAINT);
    change_model!(content, Option<Image>, RenderableFlags::NEEDS_PAINT);

    // change_model!(
    //     size,
    //     Point,
    //     RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT
    // );

    pub fn set_size(
        &self,
        value: impl Into<Point>,
        transition: Option<Transition<Easing>>,
    ) -> &Self {
        let value: Point = value.into();
        let flags = RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT;

        let change: Arc<ModelChange<Point>> = Arc::new(ModelChange {
            value_change: self.model.size.to(value, transition),
            flag: flags,
        });
        let id: Option<NodeRef> = *self.id.read().unwrap();
        if let Some(id) = id {
            self.engine.schedule_change(id, change);
        } else {
            self.model.size.set(value);
            self.engine.set_node_layout_size(self.layout, value);
        }
        self
    }

    pub fn set_layout_style(&self, style: Style) {
        self.engine.set_node_layout_style(self.layout, style);
    }

    pub fn into_render_layer(&self) -> RenderLayer {
        let model = &*self.model.clone();
        model.into()
    }
    // pub fn set_content_from_file(&self, file_path: &str) {
    //     // read jpg file
    //     let data = std::fs::read(file_path).unwrap();
    //     unsafe {
    //         let data = skia_safe::Data::new_bytes(data.as_slice());
    //         let content = Image::from_encoded(data);

    //         self.set_content(content.clone(), None);
    //     }
    // }
    pub fn set_content_from_data_raster_rgba8(
        &self,
        data: &Vec<u8>,
        width: impl Into<i32>,
        height: impl Into<i32>,
    ) {
        let width = width.into();
        let height = height.into();
        unsafe {
            let image_info = skia_safe::ImageInfo::new(
                skia_safe::ISize::new(width, height),
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Premul,
                None,
            );

            let data = skia_safe::Data::new_bytes(data.as_slice());
            let row_bytes = width as usize * image_info.bytes_per_pixel();
            let pixmap = Pixmap::new(&image_info, data.as_bytes(), row_bytes);

            let mut bitmap = Bitmap::new();
            bitmap.install_pixels(&image_info, pixmap.writable_addr(), row_bytes);
            let content = Image::from_bitmap(&bitmap);

            if let Some(image) = content {
                self.set_content(Some(image), None);
            }
        }
    }

    pub fn set_content_from_data_encoded(&self, data: &Vec<u8>) {
        unsafe {
            let data = skia_safe::Data::new_bytes(data.as_slice());

            let content = Image::from_encoded(data);
            if let Some(image) = content {
                let image = image.to_raster_image(None);
                println!("image size: {:?}", image);
                self.set_content(image, None);
            }
        }
    }
    // // set content from gl texture
    // pub fn set_content_from_texture(
    //     &self,
    //     texture_id: u32,
    //     target: skia_safe::gpu::gl::Enum,
    //     _format: skia_safe::gpu::gl::Enum,
    //     size: impl Into<Point>,
    // ) {
    //     let size = size.into();
    //     unsafe {
    //         let mut gr_context: skia_safe::gpu::DirectContext =
    //             skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();

    //         let mut texture_info =
    //             skia_safe::gpu::gl::TextureInfo::from_target_and_id(target, texture_id);
    //         texture_info.format = skia_safe::gpu::gl::Format::RGBA8.into();

    //         let texture = BackendTexture::new_gl(
    //             (size.x as i32, size.y as i32),
    //             skia_safe::gpu::MipMapped::Yes,
    //             texture_info,
    //         );

    //         let image = Image::from_texture(
    //             &mut gr_context,
    //             &texture.clone(),
    //             skia_safe::gpu::SurfaceOrigin::TopLeft,
    //             skia_safe::ColorType::RGBA8888,
    //             skia_safe::AlphaType::Opaque,
    //             None,
    //         )
    //         .unwrap()
    //         .clone();

    //         let image = image.to_raster_image(CachingHint::Allow).unwrap();

    //         self.set_content(Some(image), None);
    //     }
    // }

    pub fn bounds(&self) -> Rectangle {
        self.model.bounds()
    }

    pub fn add_sublayer(&self, layer: Layer) -> NodeRef {
        self.engine.scene_add_layer(layer, self.id())
    }
}

impl From<Layer> for Arc<dyn RenderNode> {
    fn from(layer: Layer) -> Self {
        layer.model
    }
}
