use bitflags::bitflags;
use indextree::{Arena, NodeId};
use skia_safe::{Picture, Point as SkiaPoint};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::types::{Point, Rectangle};

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
        canvas.draw_picture(picture, Some(&matrix), None);
    }
}

pub fn render_node_children(
    node_id: NodeId,
    arena: &Arena<SceneNode>,
    canvas: &mut skia_safe::Canvas,
) {
    let node = arena.get(node_id).unwrap().get();
    let sc = canvas.save();
    let matrix = *node.transformation.read().unwrap();
    canvas.concat(&matrix);
    node_id.children(arena).for_each(|child_id| {
        if let Some(child) = arena.get(child_id) {
            render_node(child.get(), canvas);
        }
    });
    canvas.restore_to_count(sc);
}

pub trait ContainsPoint {
    fn contains(&self, point: Point) -> bool;
}

impl ContainsPoint for SceneNode {
    fn contains(&self, point: Point) -> bool {
        let matrix = self.transformation.read().unwrap();
        let inverse = matrix.invert().unwrap();
        let point = inverse.map_point(SkiaPoint::new(point.x as f32, point.y as f32));
        let point = Point {
            x: point.x as f64,
            y: point.y as f64,
        };
        self.model.bounds().contains(point)
    }
}

impl ContainsPoint for Rectangle {
    fn contains(&self, point: Point) -> bool {
        self.x <= point.x
            && self.y <= point.y
            && self.x + self.width >= point.x
            && self.y + self.height >= point.y
    }
}
