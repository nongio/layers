use bitflags::bitflags;
use skia_safe::Picture;
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use super::{
    rendering::{DrawToPicture, Drawable},
    scene::SceneRef,
    storage::TreeStorageId,
    ChangeInvoker,
};

#[derive(Clone, Debug)]
pub struct DrawCache {
    pub picture: Option<Picture>,
}

/// A trait for objects that can be rendered (and cached) by the engine.
pub trait Renderable: Drawable + DrawToPicture + ChangeInvoker + Send + Sync {}

bitflags! {
    pub struct NodeFlags: u32 {
        const NEEDS_LAYOUT = 1 << 0;
        const NEEDS_PAINT = 1 << 1;
    }
}

#[derive(Clone)]
pub struct SceneNode {
    pub id: Option<TreeStorageId>,
    pub scene: Option<SceneRef>,
    pub model: Arc<dyn Renderable>,
    pub matrix: Arc<RwLock<skia_safe::Matrix>>,
    pub draw_cache: Arc<RwLock<DrawCache>>,
    pub flags: Arc<RwLock<NodeFlags>>,
}

/// A trait for Nodes to expose their cache management
pub trait DrawCacheManagement {
    fn repaint_if_needed(&self);
    fn set_need_repaint(&self, value: bool);
    fn layout_if_needed(&self);
    fn set_need_layout(&self, value: bool);
}

impl DrawCacheManagement for SceneNode {
    fn repaint_if_needed(&self) {
        if self.flags.read().unwrap().contains(NodeFlags::NEEDS_PAINT) {
            self.draw_cache.write().unwrap().picture = self.model.draw_to_picture();
        }
    }

    fn layout_if_needed(&self) {
        if self.flags.read().unwrap().contains(NodeFlags::NEEDS_LAYOUT) {
            *self.matrix.write().unwrap() = self.model.transform();
        }
    }

    fn set_need_repaint(&self, need_repaint: bool) {
        self.flags
            .write()
            .unwrap()
            .set(NodeFlags::NEEDS_PAINT, need_repaint);
    }
    fn set_need_layout(&self, need_layout: bool) {
        self.flags
            .write()
            .unwrap()
            .set(NodeFlags::NEEDS_LAYOUT, need_layout);
    }
}

impl SceneNode {
    pub fn with_renderable(model: Arc<dyn Renderable>) -> Self {
        Self {
            id: None,
            scene: None,
            draw_cache: Arc::new(RwLock::new(DrawCache { picture: None })),
            model: model.clone(),
            matrix: Arc::new(RwLock::new(skia_safe::Matrix::new_identity())),
            flags: Arc::new(RwLock::new(
                NodeFlags::NEEDS_PAINT | NodeFlags::NEEDS_LAYOUT,
            )),
        }
    }
}

pub fn render_node(node: &SceneNode, canvas: &mut skia_safe::Canvas) {
    let draw_cache = node.draw_cache.read().unwrap();
    let matrix = node.matrix.read().unwrap();
    if let Some(picture) = &draw_cache.picture {
        canvas.draw_picture(&picture, Some(&matrix), None);
    }
}
