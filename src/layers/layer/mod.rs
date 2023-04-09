pub(crate) mod drawable;
pub(crate) mod model;
pub(crate) mod render_layer;
pub(crate) mod render_node;

pub(crate) use self::model::ModelLayer;
pub use self::render_layer::RenderLayer;

use skia_safe::{Bitmap, Pixmap};
use std::sync::Arc;
use std::sync::RwLock;
use taffy::prelude::Node;
use taffy::style::Style;

use crate::engine::animations::{Easing, Transition};
use crate::engine::command::change_model;
use crate::engine::command::ModelChange;
use crate::engine::node::RenderableFlags;
use crate::engine::rendering::Drawable;
use crate::engine::Engine;
use crate::engine::NodeRef;
use crate::types::*;

#[allow(dead_code)]
pub struct LayerTree {
    pub root: RenderLayer,
    pub children: Vec<RenderLayer>,
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

    pub fn into_render_layer(self) -> RenderLayer {
        let model = &*self.model;
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

    pub fn build(&self, layer: &LayerTree) -> &Self {
        self.set_size(layer.root.size, None);
        self.set_background_color(layer.root.background_color.clone(), None)
            .set_border_corner_radius(layer.root.border_corner_radius, None);
        self
    }
}
