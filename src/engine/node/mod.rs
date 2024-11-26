use bitflags::bitflags;
use indextree::Arena;

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc, RwLock,
    },
};
use taffy::prelude::{Layout, NodeId as TaffyNodeId};

use crate::{
    drawing::render_node_tree,
    layers::layer::{render_layer::RenderLayer, Layer},
    types::*, // utils::save_image,
};

use super::{draw_to_picture::DrawDebugInfo, NodeRef};
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
    size: skia_safe::Size,
    offset: skia_safe::Point,
}
thread_local! {
    static ID_COUNTER: AtomicUsize = const { AtomicUsize::new(0) };
    static SURFACES: RefCell<HashMap<usize, skia_safe::Surface>> = RefCell::new(HashMap::new());
}

impl DrawCache {
    pub fn new(picture: Picture, size: skia_safe::Size, offset: skia_safe::Point) -> Self {
        Self {
            picture,
            size,
            offset,
        }
    }
    pub fn picture(&self) -> &Picture {
        &self.picture
    }
    pub fn size(&self) -> &skia_safe::Size {
        &self.size
    }
    pub fn draw(&self, canvas: &skia_safe::Canvas, paint: &skia_safe::Paint) {
        if self.size.width == 0.0 || self.size.height == 0.0 {
            return;
        }
        canvas.draw_picture(&self.picture, None, Some(paint));
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
    pub(crate) draw_cache: Arc<RwLock<Option<DrawCache>>>,
    pub(crate) flags: Arc<RwLock<RenderableFlags>>,
    pub(crate) layout_node_id: TaffyNodeId,
    pub(crate) deleted: Arc<AtomicBool>,
    pub(crate) pointer_hover: Arc<AtomicBool>,
    pub(crate) debug_info: Arc<RwLock<Option<DrawDebugInfo>>>,
    pub(crate) repaint_damage: Arc<RwLock<skia_safe::Rect>>,
    pub(crate) frame: Arc<AtomicUsize>,
    pub(crate) _follow_node: Arc<RwLock<Option<NodeRef>>>,
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
            repaint_damage: Arc::new(RwLock::new(skia_safe::Rect::default())),
            debug_info: Arc::new(RwLock::new(None)),
            frame: Arc::new(AtomicUsize::new(0)),
            _follow_node: Arc::new(RwLock::new(None)),
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
    pub fn bounds_with_children(&self) -> skia_safe::Rect {
        let render_layer = self.render_layer.read().unwrap();
        render_layer.bounds_with_children
    }
    pub fn transformed_bounds(&self) -> skia_safe::Rect {
        let render_layer = self.render_layer.read().unwrap();
        render_layer.global_transformed_bounds
    }
    pub fn transformed_bounds_with_effects(&self) -> skia_safe::Rect {
        let render_layer = self.render_layer.read().unwrap();
        render_layer
            .global_transformed_bounds_with_children
            .with_outset((
                render_layer.border_width / 2.0,
                render_layer.border_width / 2.0,
            ))
    }
    pub fn transform(&self) -> Matrix {
        self.render_layer.read().unwrap().transform.to_m33()
    }
    pub fn mark_for_deletion(&self) {
        self.deleted
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn is_deleted(&self) -> bool {
        self.deleted.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn set_debug_info(&self, debug_info: bool) {
        let mut dbg_info = self.debug_info.write().unwrap();
        if debug_info {
            let id: usize = self.layer.id().unwrap().0.into();
            *dbg_info = Some(DrawDebugInfo {
                info: format!("{}", id),
                frame: self.frame.load(std::sync::atomic::Ordering::Relaxed),
                render_layer: self.render_layer(),
            });
        } else {
            *dbg_info = None;
        }
        self.layer.set_opacity(self.layer.opacity(), None);
        self.set_need_repaint(true);
    }
    pub fn is_image_cached(&self) -> bool {
        self.layer
            .image_cache
            .load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn render_layer(&self) -> RenderLayer {
        self.render_layer.read().unwrap().clone()
    }
    pub fn cached_picture(&self) -> Option<DrawCache> {
        let draw_cache = self.draw_cache.read().unwrap();
        if let Some(dc) = &*draw_cache {
            return Some(dc.clone());
        }
        None
    }
    pub(crate) fn increase_frame(&self) {
        if self.is_image_cached() {
            // println!("{:?} increase  _frame", self.id());
            if self
                .frame
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                > 99999
            {
                self.frame.store(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
    pub(crate) fn change_hover(&self, value: bool) -> bool {
        let hover = self
            .pointer_hover
            .load(std::sync::atomic::Ordering::Relaxed);
        if hover != value {
            self.pointer_hover
                .store(value, std::sync::atomic::Ordering::SeqCst);
            return true;
        }
        false
    }
    pub(crate) fn follow_node(&self, nodeid: &Option<NodeRef>) {
        let mut _follow_node = self._follow_node.write().unwrap();
        *_follow_node = *nodeid;
    }

    pub fn replicate_node(&self, nodeid: &Option<NodeRef>) {
        if let Some(nodeid) = nodeid {
            let nodeid = *nodeid;
            let draw_function =
                move |c: &skia::Canvas, w: f32, h: f32, arena: &Arena<SceneNode>| {
                    render_node_tree(nodeid, arena, c, 1.0);
                    skia::Rect::from_xywh(0.0, 0.0, w, h)
                };

            self.layer.set_draw_content_internal(draw_function);
        }

        self.follow_node(nodeid);
    }
}

impl DrawCacheManagement for SceneNode {
    fn repaint_if_needed(&self, arena: &Arena<SceneNode>) -> skia_safe::Rect {
        let mut damage = skia_safe::Rect::default();
        let render_layer = self.render_layer.read().unwrap();

        if self.layer.hidden() || render_layer.premultiplied_opacity == 0.0 {
            let mut last_damage = self.repaint_damage.write().unwrap();
            let ld = *last_damage;
            *last_damage = damage;
            return ld;
        }
        let mut needs_repaint = self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_PAINT);
        let mut draw_cache = self.draw_cache.write().unwrap();

        // if the size has changed from the layout, we need to repaint
        // the flag want be set if the size has changed from the layout calculations
        if let Some(dc) = &*draw_cache {
            if render_layer.size != *dc.size() {
                needs_repaint = true;
            }
        }
        // FIXME
        if render_layer.blend_mode == BlendMode::BackgroundBlur {
            needs_repaint = true;
        }
        if needs_repaint {
            let (picture, layer_damage) = render_layer.draw_to_picture(arena);
            let (layer_damage_transformed, _) =
                render_layer.transform.to_m33().map_rect(layer_damage);

            damage.join(layer_damage_transformed);
            if self.layer.is_picture_cache() {
                if let Some(picture) = picture {
                    // update or create the draw cache
                    if let Some(dc) = &mut *draw_cache {
                        dc.picture = picture;
                        dc.size = render_layer.size;
                    } else {
                        let size = render_layer.size;

                        let new_cache = DrawCache::new(
                            picture,
                            size,
                            skia_safe::Point {
                                x: render_layer.border_width * 2.0,
                                y: render_layer.border_width * 2.0,
                            },
                        );
                        *draw_cache = Some(new_cache);
                    }
                    let mut repaint_damage = self.repaint_damage.write().unwrap();
                    let previous_damage = *repaint_damage;
                    *repaint_damage = damage;
                    damage.join(previous_damage);
                    self.set_need_repaint(false);
                }
            }
        }
        damage
    }

    fn layout_if_needed(
        &self,
        layout: &Layout,
        matrix: Option<&M44>,
        context_opacity: f32,
        arena: &Arena<SceneNode>,
    ) -> bool {
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
            render_layer.update_with_model_and_layout(
                &self.layer.model,
                layout,
                matrix,
                context_opacity,
                self.is_content_cached(),
                arena,
            );

            self.set_need_layout(false);
            // self.increase_frame();
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
        let render_layer = self.render_layer.read().unwrap();
        let draw_cache = self.draw_cache.read().unwrap();

        let mut needs_repaint = self
            .flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_PAINT)
            || render_layer.blend_mode == BlendMode::BackgroundBlur;
        if let Some(dc) = &*draw_cache {
            if render_layer.size != *dc.size() {
                needs_repaint = true;
            }
        }
        needs_repaint
    }
    fn needs_layout(&self) -> bool {
        self.flags
            .read()
            .unwrap()
            .contains(RenderableFlags::NEEDS_LAYOUT)
    }
    fn is_content_cached(&self) -> bool {
        self.layer
            .picture_cache
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub(crate) fn try_get_node(node: &indextree::Node<SceneNode>) -> Option<SceneNode> {
    if node.is_removed() {
        None
    } else {
        Some(node.get().to_owned())
    }
}
