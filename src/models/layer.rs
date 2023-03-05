use skia_safe::gpu::BackendTexture;
use skia_safe::image::CachingHint;
use skia_safe::{Bitmap, Canvas, Image, Matrix, Pixmap, M44, V3};

use std::sync::Arc;
use std::sync::RwLock;

use crate::drawing::layer::draw_layer;
use crate::engine::node::RenderNode;
use crate::engine::node::RenderableFlags;
use crate::engine::rendering::Drawable;
use crate::engine::{ChangeProducer, Engine};
use crate::engine::{NodeRef, TransactionRef};
use crate::models::*;
use crate::types::*;

use super::change_attr;

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
#[repr(transparent)]
pub struct SkiaImage {
    pub data: Box<Image>,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Layer {
    pub background_color: PaintColor,
    pub border_color: PaintColor,
    pub border_width: f64,
    pub border_style: BorderStyle,
    pub border_corner_radius: BorderRadius,
    pub size: Point,
    pub shadow_offset: Point,
    pub shadow_radius: f64,
    pub shadow_color: Color,
    pub shadow_spread: f64,
    pub matrix: Matrix,
    pub content: Option<Image>,
    pub blend_mode: BlendMode,
}
// #[derive(Clone)]
pub struct ModelLayer {
    pub anchor_point: SyncValue<Point>,
    pub position: SyncValue<Point>,
    pub scale: SyncValue<Point>,
    pub rotation: SyncValue<Point3d>,
    pub size: SyncValue<Point>,
    pub background_color: SyncValue<PaintColor>,
    pub border_corner_radius: SyncValue<BorderRadius>,
    pub border_color: SyncValue<PaintColor>,
    pub border_width: SyncValue<f64>,
    pub shadow_offset: SyncValue<Point>,
    pub shadow_radius: SyncValue<f64>,
    pub shadow_spread: SyncValue<f64>,
    pub shadow_color: SyncValue<Color>,
    pub matrix: M44,

    pub content: SyncValue<Option<Image>>,
    pub _backend_texture: RwLock<Option<BackendTexture>>,

    pub blend_mode: BlendMode,

    pub engine: RwLock<Option<(NodeRef, Arc<Engine>)>>,
}

impl ModelLayer {
    fn new() -> Self {
        Default::default()
    }
    pub fn create() -> Arc<ModelLayer> {
        Arc::new(Self::new())
    }

    change_attr!(position, Point, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(
        size,
        Point,
        RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT
    );
    change_attr!(background_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_attr!(scale, Point, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(rotation, Point3d, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(anchor_point, Point, RenderableFlags::NEEDS_LAYOUT);
    change_attr!(
        border_corner_radius,
        BorderRadius,
        RenderableFlags::NEEDS_PAINT
    );
    change_attr!(border_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_attr!(border_width, f64, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_offset, Point, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_radius, f64, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_spread, f64, RenderableFlags::NEEDS_PAINT);
    change_attr!(shadow_color, Color, RenderableFlags::NEEDS_PAINT);
    change_attr!(content, Option<Image>, RenderableFlags::NEEDS_PAINT);

    pub fn set_content_from_file(&self, file_path: &str) {
        // read jpg file
        let data = std::fs::read(file_path).unwrap();
        unsafe {
            let data = skia_safe::Data::new_bytes(data.as_slice());
            let content = Image::from_encoded(data);

            self.set_content(content.clone(), None);
        }
    }
    pub fn set_content_from_data_raster_rgba8(&self, data: Vec<u8>, width: i32, height: i32) {
        unsafe {
            let image_info = skia_safe::ImageInfo::new(
                skia_safe::ISize::new(width, height),
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Premul,
                None,
            );

            let data = skia_safe::Data::new_bytes(data.as_slice());
            let row_bytes = (width as usize * image_info.bytes_per_pixel()) as usize;
            let pixmap = Pixmap::new(&image_info, data.as_bytes(), row_bytes);

            let mut bitmap = Bitmap::new();
            bitmap.install_pixels(&image_info, pixmap.writable_addr(), row_bytes);
            let content = Image::from_bitmap(&bitmap);

            // let content = Image::from_raster_data(&image_info, data, width as usize * 4);

            if let Some(image) = content {
                self.set_content(Some(image), None);
            }
        }
    }

    pub fn set_content_from_data_encoded(&self, data: Vec<u8>) {
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
    // set content from gl texture
    pub fn set_content_from_texture(
        &self,
        texture_id: u32,
        target: skia_safe::gpu::gl::Enum,
        _format: skia_safe::gpu::gl::Enum,
        size: impl Into<Point>,
    ) {
        let size = size.into();
        unsafe {
            let mut gr_context: skia_safe::gpu::DirectContext =
                skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();

            let mut texture_info =
                skia_safe::gpu::gl::TextureInfo::from_target_and_id(target, texture_id);
            texture_info.format = skia_safe::gpu::gl::Format::RGBA8.into();

            let texture = BackendTexture::new_gl(
                (size.x as i32, size.y as i32),
                skia_safe::gpu::MipMapped::Yes,
                texture_info,
            );

            let image = Image::from_texture(
                &mut gr_context,
                &texture.clone(),
                skia_safe::gpu::SurfaceOrigin::TopLeft,
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Opaque,
                None,
            )
            .unwrap()
            .clone();

            let image = image.to_raster_image(CachingHint::Allow).unwrap();

            self.set_content(Some(image), None);
        }
    }
}

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
        let matrix = M44::new_identity();
        let shadow_offset = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let shadow_radius = SyncValue::new(0.0);
        let shadow_spread = SyncValue::new(0.0);
        let shadow_color = SyncValue::new(Color::new_rgba(0.0, 0.0, 0.0, 1.0));
        let content = SyncValue::new(None);
        let blend_mode = BlendMode::Normal;
        let engine = RwLock::new(None);

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
            matrix,
            blend_mode,
            engine,
            _backend_texture: RwLock::new(None),
        }
    }
}

impl Drawable for ModelLayer {
    fn draw(&self, canvas: &mut Canvas) {
        let layer: Layer = Layer::from(self);
        draw_layer(canvas, &layer);
    }
    fn bounds(&self) -> Rectangle {
        let s = self.size.value();
        Rectangle {
            x: 0.0,
            y: 0.0,
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
        let anchor_translate = M44::translate(
            -anchor_point.x as f32 * size.x as f32,
            -anchor_point.y as f32 * size.y as f32,
            0.0,
        );
        let identity = M44::new_identity();
        let translate = M44::translate(p.x as f32, p.y as f32, 0.0);
        let _scale = M44::scale(s.x as f32, s.y as f32, 1.0);
        let rotate_x = M44::rotate(
            V3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            rotation.x as f32,
        );
        let rotate_y = M44::rotate(
            V3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            rotation.y as f32,
        );
        let rotate_z = M44::rotate(
            V3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            rotation.z as f32,
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
        (s.x as f32, s.y as f32)
    }
}

impl ChangeProducer for ModelLayer {
    fn set_engine(&self, engine: Arc<Engine>, id: NodeRef) {
        *self.engine.write().unwrap() = Some((id, engine));
    }
}

impl RenderNode for ModelLayer {}

// Convertion helpers

impl From<&ModelLayer> for Layer {
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

impl From<Layer> for ModelLayer {
    fn from(layer: Layer) -> Self {
        let size = layer.size;
        let background_color = layer.background_color;
        let border_color = layer.border_color;
        let border_corner_radius = layer.border_corner_radius;
        let border_width = layer.border_width;
        let shadow_offset = layer.shadow_offset;
        let shadow_radius = layer.shadow_radius;
        let shadow_spread = layer.shadow_spread;
        let shadow_color = layer.shadow_color;
        let matrix = M44::new_identity();
        let content = match layer.content {
            None => SyncValue::new(None),
            Some(image) => SyncValue::new(Some(Image::from(image))),
        };

        let blend_mode = layer.blend_mode;

        let (x, y) = (
            layer.matrix.translate_x() as f64,
            layer.matrix.translate_y() as f64,
        );
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
            matrix,
            blend_mode,
            engine: RwLock::new(None),
            _backend_texture: RwLock::new(None),
        }
    }
}

impl From<ModelLayer> for Arc<dyn RenderNode> {
    fn from(model: ModelLayer) -> Self {
        Arc::new(model)
    }
}
