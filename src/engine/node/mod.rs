use bitflags::bitflags;

use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};
use taffy::prelude::{Layout, Node};

use crate::{layers::Layers, types::*};

use super::{draw_to_picture::DrawToPicture, rendering::Drawable};
pub(crate) mod contains_point;
pub(crate) mod draw_cache_management;

pub use contains_point::ContainsPoint;
pub use draw_cache_management::DrawCacheManagement;

/// SceneNode is the main data structure for the engine. It contains a model
/// that can be rendered, and a layout node that can be used to layout the
/// model. As well it contains the data structures that are used to cache
/// the rendering of the model. Caching is done using skia display lists.

#[derive(Clone, Debug)]
pub struct DrawCache {
    picture: Picture,
    size: Size,
}

impl DrawCache {
    pub fn new(picture: Picture, size: Size) -> Self {
        Self { picture, size }
    }
    pub fn picture(&self) -> &Picture {
        &self.picture
    }
    pub fn size(&self) -> &Size {
        &self.size
    }
}

/// A trait for objects that can be rendered (and cached) by the engine.
pub trait RenderNode: Drawable + DrawToPicture + Send + Sync {}

bitflags! {
    pub struct RenderableFlags: u32 {
        const NOOP = 1 << 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_PAINT = 1 << 2;
        const NEEDS_RASTER = 1 << 3;
        const ANIMATING = 1 << 4;
    }
}

#[derive(Clone)]
pub struct SceneNode {
    pub model: Arc<Layers>,
    pub transformation: Arc<RwLock<skia_safe::Matrix>>,
    pub scale: Arc<RwLock<(f32, f32)>>,
    pub draw_cache: Arc<RwLock<Option<DrawCache>>>,
    pub flags: Arc<RwLock<RenderableFlags>>,
    pub layout_node: Node,
}

impl SceneNode {
    pub fn with_renderable_and_layout(model: Arc<Layers>, layout_node: Node) -> Self {
        Self {
            model,
            transformation: Arc::new(RwLock::new(skia_safe::Matrix::new_identity())),
            scale: Arc::new(RwLock::new((1.0, 1.0))),
            draw_cache: Arc::new(RwLock::new(None)),
            flags: Arc::new(RwLock::new(
                RenderableFlags::NEEDS_PAINT
                    | RenderableFlags::NEEDS_LAYOUT
                    | RenderableFlags::NEEDS_PAINT
                    | RenderableFlags::NEEDS_RASTER,
            )),
            layout_node,
        }
    }
    pub fn insert_flags(&self, flags: RenderableFlags) {
        self.flags.write().unwrap().insert(flags);
    }
    pub fn remove_flags(&self, flags: RenderableFlags) {
        self.flags.write().unwrap().remove(flags);
    }
}
#[derive(Clone)]
pub struct SceneNodeHandle(pub Arc<SceneNode>);
impl SceneNodeHandle {
    pub fn new(node: SceneNode) -> Self {
        Self(Arc::new(node))
    }
}

impl DrawCacheManagement for SceneNode {
    fn repaint_if_needed(&self) {
        let mut needs_repaint = self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_PAINT);
        let mut draw_cache = self.draw_cache.write().unwrap();
        if let Some(dc) = &*draw_cache {
            let bounds = self.model.bounds();
            let size = Size {
                x: bounds.width,
                y: bounds.height,
            };
            if size != *dc.size() {
                needs_repaint = true;
                // println!("Repainting because size changed");
            }
        }
        if needs_repaint {
            let picture = self.model.draw_to_picture();
            if let Some(picture) = picture {
                let bounds = self.model.bounds();
                let size = Size {
                    x: bounds.width,
                    y: bounds.height,
                };
                let new_cache = DrawCache::new(picture, size);
                *draw_cache = Some(new_cache);
                self.set_need_repaint(false);
                self.set_need_raster(true);
            }
        }
    }

    fn layout_if_needed(&self, layout: &Layout) {
        // TODO check if the layout position has changed
        if self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_LAYOUT)
        {
            let identity = M44::new_identity();
            let bounds = self.model.bounds();
            let translate = M44::translate(
                layout.location.x + bounds.x,
                layout.location.y + bounds.y,
                0.0,
            );
            let scale = self.model.scale();
            let scale = M44::scale(scale.0, scale.1, 0.0);
            let transform = M44::concat(&translate, &identity);
            let transform = M44::concat(&transform, &scale);
            // let transform = M44::concat(&transform, &rotate_x);
            // let transform = M44::concat(&transform, &rotate_y);
            // let transform = M44::concat(&transform, &rotate_z);
            // let transform = M44::concat(&transform, &anchor_translate);
            *self.transformation.write().unwrap() = transform.to_m33();
            *self.scale.write().unwrap() = self.model.scale();
            self.set_need_layout(false);
            // self.set_need_repaint(true);
            // self.set_need_raster(true);
        }
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
    fn set_need_raster(&self, need_raster: bool) {
        self.flags
            .write()
            .unwrap()
            .set(RenderableFlags::NEEDS_RASTER, need_raster);
    }
    fn need_raster(&self) -> bool {
        self.flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_RASTER)
    }
}