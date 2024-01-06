use bitflags::bitflags;

use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};
use taffy::prelude::{Layout, Node};

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
    pub layout_node_id: Node,
}

impl SceneNode {
    pub fn id(&self) -> Option<NodeRef> {
        self.layer.id()
    }
    pub fn with_renderable_and_layout(layer: Layer, layout_node: Node) -> Self {
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
        }
    }
    pub fn insert_flags(&self, flags: RenderableFlags) {
        self.flags.write().unwrap().insert(flags);
    }
    pub fn remove_flags(&self, flags: RenderableFlags) {
        self.flags.write().unwrap().remove(flags);
    }
    pub fn bounds(&self) -> Rectangle {
        self.render_layer.read().unwrap().bounds
    }
    pub fn transform(&self) -> Matrix {
        self.render_layer.read().unwrap().transform
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
        let render_layer = self.render_layer.read().unwrap();

        // if the size has changed from the layout, we need to repaint
        // the flag want be set if the size has changed from the layout calculations
        if let Some(dc) = &*draw_cache {
            if render_layer.size != *dc.size() {
                needs_repaint = true;
                // println!("Repainting because size changed");
            }
        }
        if needs_repaint {
            let picture = render_layer.draw_to_picture();
            if let Some(picture) = picture {
                let new_cache = DrawCache::new(picture, render_layer.size);
                *draw_cache = Some(new_cache);
                self.set_need_repaint(false);
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
            *self.render_layer.write().unwrap() =
                RenderLayer::from_model_and_layout(&self.layer.model, layout);

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

pub(crate) fn try_get_node(node: indextree::Node<SceneNode>) -> Option<SceneNode> {
    if node.is_removed() {
        None
    } else {
        Some(node.get().to_owned())
    }
}
