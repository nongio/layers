use bitflags::bitflags;
// use skia_safe::surface;

use std::{
    fmt::Debug,
    sync::{atomic::AtomicBool, Arc, RwLock},
};
use taffy::prelude::{Layout, NodeId as TaffyNodeId};

use crate::{
    layers::layer::{render_layer::RenderLayer, Layer},
    types::*,
};

use super::NodeRef;
use crate::engine::draw_to_picture::DrawToPicture;

pub(crate) mod contains_point;
pub(crate) mod draw_cache_management;

pub use contains_point::ContainsPoint;
pub use draw_cache_management::DrawCacheManagement;

/// SceneNode is the main data structure for the engine. It contains a model
/// that can be rendered, and a layout node that can be used to position and size the
/// model. As well it contains the data structures that are used to cache
/// the rendering of the model. Caching is done using skia Picture.

#[derive(Clone, Debug)]
pub struct DrawCache {
    picture: Picture,
    image: Arc<RwLock<Option<skia_safe::Image>>>,
    size: skia_safe::Size,
    offset: skia_safe::Point,
    cache_to_image: bool,
    // surface: Arc<RwLock<Option<skia_safe::Surface>>>,
}

#[allow(dead_code)]
fn save_image<'a>(
    context: impl Into<Option<&'a mut skia_safe::gpu::DirectContext>>,
    image: &skia_safe::Image,
    name: &str,
) {
    use std::fs::File;
    use std::io::Write;

    let data = image
        .encode(context.into(), skia_safe::EncodedImageFormat::PNG, None)
        .unwrap();
    let bytes = data.as_bytes();
    let filename = format!("{}.png", name);
    let mut file = File::create(filename).unwrap();
    file.write_all(bytes).unwrap();
}

impl DrawCache {
    pub fn new(
        picture: Picture,
        size: skia_safe::Size,
        offset: skia_safe::Point,
        cache_to_image: bool,
    ) -> Self {
        Self {
            picture,
            size,
            image: Arc::new(RwLock::new(None)),
            offset,
            cache_to_image,
        }
    }
    pub fn picture(&self) -> &Picture {
        &self.picture
    }
    pub fn size(&self) -> &skia_safe::Size {
        &self.size
    }
    pub fn draw_picture_to_canvas(&self, canvas: &skia_safe::Canvas, paint: &skia_safe::Paint) {
        canvas.draw_picture(&self.picture, None, Some(paint));
    }
    pub fn draw_to_image(&self, context: &mut skia_safe::gpu::DirectContext) {
        // Define the width and height of the canvas
        let width = self.size.width as i32 + self.offset.x as i32 * 2;
        let height = self.size.height as i32 + self.offset.y as i32 * 2;
        if width == 0 || height == 0 {
            return;
        }

        let image_info = skia_safe::ImageInfo::new(
            (width, height),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );

        let mut surface = skia_safe::gpu::surfaces::render_target(
            context,
            skia_safe::gpu::Budgeted::No,
            &image_info,
            None,
            skia_safe::gpu::SurfaceOrigin::TopLeft,
            None,
            None,
            false,
        )
        .unwrap();

        // // Get the canvas from the surface
        let canvas = surface.canvas();
        let translate = skia_safe::Matrix::translate((self.offset.x, self.offset.y));
        canvas.concat(&translate);
        self.draw_picture_to_canvas(canvas, &skia_safe::Paint::default());
        context.flush_and_submit_surface(&mut surface, None);

        let image = surface.image_snapshot();

        let mut image_option = self.image.write().unwrap();
        *image_option = Some(image);
    }
    pub fn draw(&self, canvas: &skia_safe::Canvas, paint: &skia_safe::Paint) {
        let image_option = self.image.read().unwrap();
        if self.size.width == 0.0 || self.size.height == 0.0 {
            return;
        }
        if let Some(image) = image_option.as_ref() {
            // let mut paint = paint.clone();
            // let resampler = skia_safe::CubicResampler::catmull_rom();
            // let matrix = skia_safe::Matrix::translate((self.offset.x, self.offset.y));
            // paint.set_shader(image.to_shader(
            //     (skia_safe::TileMode::Repeat, skia_safe::TileMode::Repeat),
            //     skia_safe::SamplingOptions::from(resampler),
            //     // skia_safe::SamplingOptions::default(),
            //     &matrix,
            // ));
            // let rect = skia_safe::Rect::from_xywh(0.0, 0.0, self.size.width, self.size.height);
            // canvas.draw_rect(rect, &paint);
            canvas.draw_image_rect_with_sampling_options(
                image,
                None,
                skia_safe::Rect::from_xywh(
                    -self.offset.x,
                    -self.offset.y,
                    self.size.width + self.offset.x * 2.0,
                    self.size.height + self.offset.y * 2.0,
                ),
                skia_safe::SamplingOptions::default(),
                // skia_safe::SamplingOptions::from(resampler),
                paint,
            );
            // canvas.draw_image(image, (-self.offset.x, -self.offset.y), Some(paint));
        } else {
            drop(image_option);
            canvas.draw_picture(&self.picture, None, Some(paint));
            if self.cache_to_image {
                if let Some(surface) = unsafe { canvas.surface() } {
                    let mut ctx = surface.recording_context().unwrap();
                    self.draw_to_image(&mut ctx.as_direct_context().unwrap());
                }
            }
        }
    }
}

bitflags! {
    pub struct RenderableFlags: u32 {
        const NOOP = 1 << 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_PAINT = 1 << 2;
        const ANIMATING = 1 << 3;
    }
}

#[derive(Clone)]
pub struct SceneNode {
    pub layer: Layer,
    pub(crate) render_layer: Arc<RwLock<RenderLayer>>,
    pub draw_cache: Arc<RwLock<Option<DrawCache>>>,
    pub flags: Arc<RwLock<RenderableFlags>>,
    pub layout_node_id: TaffyNodeId,
    pub deleted: Arc<AtomicBool>,
    pub(crate) pointer_hover: Arc<AtomicBool>,
}

impl SceneNode {
    pub fn id(&self) -> Option<NodeRef> {
        self.layer.id()
    }
    pub fn with_renderable_and_layout(layer: Layer, layout_node: TaffyNodeId) -> Self {
        let render_layer = RenderLayer::default();
        Self {
            layer,
            draw_cache: Arc::new(RwLock::new(None)),
            flags: Arc::new(RwLock::new(
                RenderableFlags::NEEDS_PAINT
                    | RenderableFlags::NEEDS_LAYOUT
                    | RenderableFlags::NEEDS_PAINT,
            )),
            layout_node_id: layout_node,
            render_layer: Arc::new(RwLock::new(render_layer)),
            deleted: Arc::new(AtomicBool::new(false)),
            pointer_hover: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn insert_flags(&self, flags: RenderableFlags) {
        self.flags.write().unwrap().insert(flags);
    }
    pub fn remove_flags(&self, flags: RenderableFlags) {
        self.flags.write().unwrap().remove(flags);
    }
    pub fn bounds(&self) -> skia_safe::Rect {
        let render_layer = self.render_layer.read().unwrap();
        render_layer.bounds.with_outset((
            render_layer.border_width / 2.0,
            render_layer.border_width / 2.0,
        ))
    }
    pub fn transformed_bounds(&self) -> skia_safe::Rect {
        let render_layer = self.render_layer.read().unwrap();
        render_layer.transformed_bounds.with_outset((
            render_layer.border_width / 2.0,
            render_layer.border_width / 2.0,
        ))
    }
    pub fn bounds_with_children(&self) -> skia_safe::Rect {
        let render_layer = self.render_layer.read().unwrap();
        render_layer.bounds_with_children.with_outset((
            render_layer.border_width / 2.0,
            render_layer.border_width / 2.0,
        ))
    }
    pub fn transform(&self) -> Matrix {
        self.render_layer.read().unwrap().transform.to_m33()
    }
    pub fn delete(&self) {
        self.deleted
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn is_deleted(&self) -> bool {
        self.deleted.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl DrawCacheManagement for SceneNode {
    fn repaint_if_needed(&self) -> skia_safe::Rect {
        let mut damage = skia_safe::Rect::default();

        if self.layer.hidden() {
            return damage;
        }
        let mut needs_repaint = self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_PAINT);
        let mut draw_cache = self.draw_cache.write().unwrap();
        let render_layer = self.render_layer.read().unwrap();

        // if the size has changed from the layout, we need to repaint
        // the flag want be set if the size has changed from the layout calculations
        if let Some(dc) = &*draw_cache {
            if render_layer.size != *dc.size() {
                needs_repaint = true;
            }
        }
        if render_layer.blend_mode == BlendMode::BackgroundBlur {
            needs_repaint = true;
        }
        if needs_repaint {
            let (picture, layer_damage) = render_layer.draw_to_picture();
            let (layer_damage, _) = render_layer.transform.to_m33().map_rect(layer_damage);
            // println!(
            //     "layer dmg {} {} {} {}",
            //     layer_damage.x(),
            //     layer_damage.y(),
            //     layer_damage.width(),
            //     layer_damage.height()
            // );
            damage.join(layer_damage);
            if let Some(picture) = picture {
                if let Some(dc) = &mut *draw_cache {
                    dc.picture = picture;
                    dc.size = render_layer.size;
                    dc.image.write().unwrap().take();
                } else {
                    let size = render_layer.size;

                    let new_cache = DrawCache::new(
                        picture,
                        size,
                        skia_safe::Point {
                            x: render_layer.border_width / 2.0,
                            y: render_layer.border_width / 2.0,
                        },
                        self.layer
                            .image_cache
                            .load(std::sync::atomic::Ordering::Relaxed),
                    );
                    *draw_cache = Some(new_cache);
                }
                self.set_need_repaint(false);
            }
        }
        damage
    }

    fn layout_if_needed(&self, layout: &Layout, matrix: Option<&M44>) -> bool {
        if self.layer.hidden() {
            return false;
        }
        if self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_LAYOUT)
        {
            let mut render_layer = self.render_layer.write().unwrap();
            render_layer.update_with_model_and_layout(&self.layer.model, layout, matrix);

            self.set_need_layout(false);
            return true;
        }
        false
    }

    fn set_need_repaint(&self, need_repaint: bool) {
        self.flags
            .write()
            .unwrap()
            .set(RenderableFlags::NEEDS_PAINT, need_repaint);
    }
    fn set_need_layout(&self, need_layout: bool) {
        self.flags
            .write()
            .unwrap()
            .set(RenderableFlags::NEEDS_LAYOUT, need_layout);
    }
    fn needs_repaint(&self) -> bool {
        self.flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_PAINT)
    }
    fn needs_layout(&self) -> bool {
        self.flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_LAYOUT)
    }
}

pub(crate) fn try_get_node(node: indextree::Node<SceneNode>) -> Option<SceneNode> {
    if node.is_removed() {
        None
    } else {
        Some(node.get().to_owned())
    }
}
