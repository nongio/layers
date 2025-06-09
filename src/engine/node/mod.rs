use bitflags::bitflags;
use skia::Contains;

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    sync::{atomic::AtomicUsize, Arc},
};
use taffy::prelude::Layout;

use crate::{
    engine::draw_to_picture::draw_layer_to_picture,
    layers::layer::{render_layer::RenderLayer, ModelLayer},
    types::*, // utils::save_image,
};

use super::{draw_to_picture::DrawDebugInfo, NodeRef};

pub(crate) mod contains_point;

pub use contains_point::ContainsPoint;

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
    #[profiling::function]
    pub fn draw(&self, canvas: &skia_safe::Canvas, paint: Option<&skia_safe::Paint>) {
        if self.size.width == 0.0 || self.size.height == 0.0 {
            return;
        }
        canvas.draw_picture(&self.picture, None, paint);
    }
}

// The RenderableFlags struct is a bitflags struct that is used to manage the rendering states of a SceneNode.
// Changing a Layer property will set the corresponding flag in the SceneNode.
// Noop has no effect on the layer.
// NeedsLayout will sync with the layout node properties might trigger a layout tree compute
// NeedsPaint will trigger a repaint of the layer

bitflags! {
    pub struct RenderableFlags: u32 {
        const NOOP = 1 << 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_PAINT = 1 << 2;
    }
}

/// The SceneNode struct represents a node in the scene graph.
/// It contains a Layer and manages rendering states, caching and interactions.
/// It provides methods for managing rendering and pointer events.
#[derive(Clone)]
pub struct SceneNode {
    pub(crate) render_layer: RenderLayer,
    rendering_flags: RenderableFlags,
    pub(crate) repaint_damage: skia_safe::Rect,
    pub(crate) hidden: bool,
    pub(crate) depth: usize,
    pub(crate) image_cached: bool,
    pub(crate) picture_cached: bool,
    pub(crate) is_deleted: bool,
    pub(crate) frame_number: usize,
    pub(crate) draw_cache: Option<DrawCache>,
    pub(crate) _debug_info: Option<DrawDebugInfo>,
    pub(crate) _follow_node: Option<NodeRef>,
}

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            render_layer: RenderLayer::default(),
            repaint_damage: skia_safe::Rect::default(),
            rendering_flags: RenderableFlags::NEEDS_PAINT
                | RenderableFlags::NEEDS_LAYOUT
                | RenderableFlags::NEEDS_PAINT,
            hidden: false,
            depth: 0,
            image_cached: false,
            picture_cached: true,
            is_deleted: false,
            frame_number: 0,
            draw_cache: None,
            _debug_info: None,
            _follow_node: None,
        }
    }
}

impl SceneNode {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert_flags(&mut self, flags: RenderableFlags) {
        self.rendering_flags.insert(flags);
    }
    pub fn remove_flags(&mut self, flags: RenderableFlags) {
        self.rendering_flags.remove(flags);
    }
    pub fn bounds(&self) -> skia_safe::Rect {
        self.render_layer.bounds.with_outset((
            self.render_layer.border_width / 2.0,
            self.render_layer.border_width / 2.0,
        ))
    }
    pub fn bounds_with_children(&self) -> skia_safe::Rect {
        self.render_layer.bounds_with_children
    }
    pub fn transformed_bounds(&self) -> skia_safe::Rect {
        self.render_layer.global_transformed_bounds
    }
    pub fn transformed_bounds_with_effects(&self) -> skia_safe::Rect {
        self.render_layer
            .global_transformed_bounds_with_children
            .with_outset((
                self.render_layer.border_width / 2.0,
                self.render_layer.border_width / 2.0,
            ))
    }
    pub fn transform(&self) -> Matrix {
        self.render_layer.transform_33
    }
    pub fn mark_for_deletion(&mut self) {
        self.is_deleted = true;
    }
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }
    pub fn set_debug_info(&mut self, debug_info: bool) {
        {
            if debug_info {
                // let id: usize = self.layer.id().unwrap().0.into();
                self._debug_info = Some(DrawDebugInfo {
                    info: "".to_string(),
                    frame: self.frame_number,
                    render_layer: self.render_layer().clone(),
                });
            } else {
                self._debug_info = None;
            }
        }
        // self.layer.set_opacity(self.layer.opacity(), None);
        self.set_needs_repaint(true);
    }
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }
    pub fn hidden(&self) -> bool {
        self.hidden
    }
    pub fn set_image_cached(&mut self, value: bool) {
        self.image_cached = value;
    }
    pub fn is_image_cached(&self) -> bool {
        self.image_cached
    }
    pub fn render_layer(&self) -> &RenderLayer {
        &self.render_layer
    }
    pub fn get_cached_picture(&self) -> Option<&DrawCache> {
        self.draw_cache.as_ref()
    }
    pub(crate) fn increase_frame(&mut self) {
        if self.is_image_cached() {
            // check to not overflow the frame number
            if self.frame_number < usize::MAX {
                self.frame_number += 1;
            } else {
                self.frame_number = 1;
            }
        }
    }
    pub(crate) fn follow_node(&mut self, nodeid: &Option<NodeRef>) {
        // let mut _follow_node = self._follow_node.write().unwrap();
        self._follow_node = *nodeid;
    }
    // pub fn replicate_node(&self, nodeid: &Option<NodeRef>) {
    //     if let Some(nodeid) = nodeid {
    //         let nodeid = *nodeid;
    //         let draw_function =
    //             move |c: &skia::Canvas, w: f32, h: f32, arena: &Arena<SceneNode>| {
    //                 profiling::scope!("replicate_node");
    //                 render_node_tree(nodeid, arena, c, 1.0);
    //                 skia::Rect::from_xywh(0.0, 0.0, w, h)
    //             };

    //         self.layer.set_draw_content_internal(draw_function);
    //         // when mirroring another layer we don't want to cache the content
    //         self.layer.set_picture_cached(false);
    //     }

    //     self.follow_node(nodeid);
    // }
    // pub fn layout_node_id(&self) -> TaffyNodeId {
    //     self.layer.layout_id
    // }

    /// generate the SkPicture from drawing the Renderlayer
    /// if the layer is not hidden
    /// if the layer has opacity
    /// if the layer is marked for needs repaint
    /// returns the damaged Rect of from drawing the layer, in layers coordinates
    #[profiling::function]
    pub fn repaint_if_needed(&mut self) -> skia_safe::Rect {
        let mut damage = skia_safe::Rect::default();
        let render_layer = &self.render_layer;
        if self.hidden() || render_layer.premultiplied_opacity == 0.0 {
            let rd = self.repaint_damage;
            self.repaint_damage = damage;
            return rd;
        }

        if self.needs_repaint() {
            let (picture, _layer_damage) = draw_layer_to_picture(render_layer);
            let (layer_damage_transformed, _) = render_layer.transform_33.map_rect(_layer_damage);

            damage.join(layer_damage_transformed);
            if self.is_picture_cached() {
                if let Some(picture) = picture {
                    // update or create the draw cache
                    if let Some(draw_cache) = &mut self.draw_cache {
                        draw_cache.picture = picture;
                        draw_cache.size = render_layer.size;
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
                        self.draw_cache = Some(new_cache);
                    }
                    let previous_damage = self.repaint_damage;
                    self.repaint_damage = damage;
                    damage.join(previous_damage);

                    self.set_needs_repaint(false);
                }
            }
        }
        damage
    }
    /// update the renderlayer based on model and layout
    #[profiling::function]
    pub fn update_render_layer_if_needed(
        &mut self,
        layout: &Layout,
        model: Arc<ModelLayer>,
        matrix: Option<&M44>,
        context_opacity: f32,
    ) -> bool {
        if self.hidden() {
            return false;
        }
        if self.render_layer.size.width != layout.size.width
            || self.render_layer.size.height != layout.size.height
            || self.render_layer.local_transformed_bounds.x() != layout.location.x
            || self.render_layer.local_transformed_bounds.y() != layout.location.y
        {
            self.set_needs_repaint(true);
        }
        if self.rendering_flags.contains(RenderableFlags::NEEDS_PAINT) {
            {
                self.render_layer.update_with_model_and_layout(
                    &model,
                    layout,
                    matrix,
                    context_opacity,
                );
            }
            self.increase_frame();
            return true;
        }
        false
    }
    pub fn set_needs_repaint(&mut self, need_repaint: bool) {
        self.rendering_flags
            .set(RenderableFlags::NEEDS_PAINT, need_repaint);
    }
    pub fn set_needs_layout(&mut self, need_layout: bool) {
        self.rendering_flags
            .set(RenderableFlags::NEEDS_LAYOUT, need_layout);
    }
    pub fn needs_repaint(&self) -> bool {
        let mut needs_repaint = self.rendering_flags.contains(RenderableFlags::NEEDS_PAINT)
            || self.render_layer.blend_mode == BlendMode::BackgroundBlur;
        if let Some(dc) = self.draw_cache.as_ref() {
            if self.render_layer.size != *dc.size() {
                needs_repaint = true;
            }
        }
        needs_repaint
    }
    pub fn needs_layout(&self) -> bool {
        self.rendering_flags.contains(RenderableFlags::NEEDS_LAYOUT)
    }
    pub fn is_picture_cached(&self) -> bool {
        self.picture_cached
    }
    pub fn pointer_events(&self) -> bool {
        self.render_layer.pointer_events
    }
    pub fn contains_point(&self, point: &skia::Point) -> bool {
        self.render_layer.global_transformed_bounds.contains(point)
    }
}
