use bitflags::bitflags;
use skia_safe::Picture;
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use super::{
    rendering::{DrawToPicture, Drawable},
    ChangeProducer,
};

#[derive(Clone, Debug)]
pub struct DrawCache {
    pub picture: Option<Picture>,
}

/// A trait for objects that can be rendered (and cached) by the engine.
pub trait RenderNode: Drawable + DrawToPicture + ChangeProducer + Send + Sync {}

bitflags! {
    pub struct RenderableFlags: u32 {
        const NEEDS_LAYOUT = 1 << 0;
        const NEEDS_PAINT = 1 << 1;
    }
}

#[derive(Clone)]
pub struct SceneNode {
    pub model: Arc<dyn RenderNode>,
    pub transformation: Arc<RwLock<skia_safe::Matrix>>,
    pub draw_cache: Arc<RwLock<DrawCache>>,
    pub flags: Arc<RwLock<RenderableFlags>>,
}

impl SceneNode {
    pub fn with_renderable(model: Arc<dyn RenderNode>) -> Self {
        Self {
            model,
            transformation: Arc::new(RwLock::new(skia_safe::Matrix::new_identity())),
            draw_cache: Arc::new(RwLock::new(DrawCache { picture: None })),
            flags: Arc::new(RwLock::new(
                RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT,
            )),
        }
    }
    pub fn insert_flags(&self, flags: RenderableFlags) {
        self.flags.write().unwrap().insert(flags);
    }
}
#[derive(Clone)]
pub struct SceneNodeHandle(pub Arc<SceneNode>);
impl SceneNodeHandle {
    pub fn new(node: SceneNode) -> Self {
        Self(Arc::new(node))
    }
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
        if self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_PAINT)
        {
            self.draw_cache.write().unwrap().picture = self.model.draw_to_picture();
            self.set_need_repaint(false);
        }
    }

    fn layout_if_needed(&self) {
        if self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_LAYOUT)
        {
            *self.transformation.write().unwrap() = self.model.transform();
            self.set_need_layout(false);
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
}

impl SceneNode {}

pub fn render_node(node: &SceneNode, canvas: &mut skia_safe::Canvas) {
    let draw_cache = node.draw_cache.read().unwrap();
    let matrix = node.transformation.read().unwrap();
    if let Some(picture) = &draw_cache.picture {
        canvas.draw_picture(&picture, Some(&matrix), None);
    }
}
