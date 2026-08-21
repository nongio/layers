use bitflags::bitflags;
use skia::Contains;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
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

/// Contains the layout of a layer and information required for drawing
#[derive(Clone)]
pub struct SceneNode {
    pub(crate) render_layer: RenderLayer,
    rendering_flags: RenderableFlags,
    pub(crate) hidden: bool,
    pub(crate) image_cached: bool,
    pub(crate) picture_cached: bool,
    pub(crate) is_deleted: bool,
    pub(crate) followers: HashSet<NodeRef>,
    /// The node this layer is following (for replicate_node)
    pub(crate) following: Option<NodeRef>,
    pub(crate) _debug_info: Option<DrawDebugInfo>,
    pub(crate) frame_number: usize,
    /// Externally reported content damage, in layer-local coordinates.
    /// Populated by `Layer::add_damage` / `Layer::set_damage` for content
    /// whose damage source is outside the draw closure (e.g. Wayland
    /// surface buffer damage). Consumed and cleared by `do_repaint`,
    /// where it is unioned with the closure's returned rect.
    pub(crate) pending_damage: Option<skia_safe::Rect>,
}

/// What changed when a node's `RenderLayer` was refreshed.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RenderLayerUpdate {
    /// The layer's own box changed size — the recorded picture is stale.
    pub size_changed: bool,
    /// The layer's origin in its parent's space changed — the picture is still
    /// valid, only the transform used to replay it differs.
    pub moved: bool,
}

impl RenderLayerUpdate {
    pub fn any(&self) -> bool {
        self.size_changed || self.moved
    }
}

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            render_layer: RenderLayer::default(),
            rendering_flags: RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT,
            hidden: false,
            image_cached: false,
            picture_cached: true,
            is_deleted: false,
            _debug_info: None,
            frame_number: 0,
            followers: HashSet::new(),
            following: None,
            pending_damage: None,
        }
    }
}

/// Contains the outputs of drawing the layer: cache, damage, and flags
#[derive(Clone, Default)]
pub struct SceneNodeRenderable {
    pub(crate) repaint_damage: skia_safe::Rect,
    pub(crate) draw_cache: Option<DrawCache>,
    pub(crate) content_cache: Option<Picture>,
}

impl SceneNodeRenderable {
    /// Diagnostic summary: op counts of the recorded pictures
    /// (`draw_cache`, `content_cache`), `-1` when absent. An op count of 0
    /// on a node that should paint means the recording was made while the
    /// node had nothing to give — replaying it draws nothing.
    pub fn debug_ops(&self) -> (i64, i64) {
        (
            self.draw_cache
                .as_ref()
                .map(|c| c.picture().approximate_op_count() as i64)
                .unwrap_or(-1),
            self.content_cache
                .as_ref()
                .map(|c| c.approximate_op_count() as i64)
                .unwrap_or(-1),
        )
    }

    /// Diagnostic: the `DrawCache`'s STORED size — the value its `draw()`
    /// gates on, independent of the layer's live size.
    pub fn debug_draw_cache_size(&self) -> Option<(f32, f32)> {
        self.draw_cache
            .as_ref()
            .map(|c| (c.size().width, c.size().height))
    }

    /// Diagnostic: the cull rects of the recorded pictures
    /// (draw_cache, content_cache). `canvas.draw_picture` quick-rejects
    /// against these; `playback()` does not.
    pub fn debug_cull_rects(&self) -> (Option<skia_safe::Rect>, Option<skia_safe::Rect>) {
        (
            self.draw_cache.as_ref().map(|c| c.picture().cull_rect()),
            self.content_cache.as_ref().map(|c| c.cull_rect()),
        )
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
    pub fn set_picture_cached(&mut self, value: bool) {
        self.picture_cached = value;
    }
    pub fn is_picture_cached(&self) -> bool {
        self.picture_cached
    }
    /// Hint that the custom draw content fills the entire bounds with opaque
    /// pixels. When set, the layer can act as an occluder for occlusion culling
    /// even when its background color is transparent.
    pub fn set_content_opaque(&mut self, value: bool) {
        self.render_layer.content_opaque = value;
    }
    pub fn is_content_opaque(&self) -> bool {
        self.render_layer.content_opaque
    }
    pub fn render_layer(&self) -> &RenderLayer {
        &self.render_layer
    }
    /// Mutable access to the render layer for testing and diagnostic tools.
    pub fn render_layer_mut(&mut self) -> &mut RenderLayer {
        &mut self.render_layer
    }
    pub(crate) fn increase_frame(&mut self) {
        // if self.is_image_cached() {
        // check to not overflow the frame number
        if self.frame_number < usize::MAX {
            self.frame_number += 1;
        } else {
            self.frame_number = 1;
        }
        // }
    }
    /// Update the renderlayer based on model and layout.
    ///
    /// The returned [`RenderLayerUpdate`] keeps `size_changed` separate from
    /// `moved` because the recorded `draw_cache` picture lives in the layer's
    /// own local space: a pure translation (or scale/rotation, which are canvas
    /// transforms applied at draw time) leaves the picture valid, while a size
    /// change does not.
    #[profiling::function]
    pub(crate) fn update_render_layer_if_needed(
        &mut self,
        layout: &Layout,
        model: Arc<ModelLayer>,
        matrix: Option<&M44>,
        context_opacity: f32,
        local_children_bounds: skia_safe::Rect,
        force_update: bool,
    ) -> RenderLayerUpdate {
        let is_hidden = self.hidden();
        let current_width = self.render_layer.size.width;
        let current_height = self.render_layer.size.height;
        let current_x = self.render_layer.local_transformed_bounds.x();
        let current_y = self.render_layer.local_transformed_bounds.y();
        if current_width != layout.size.width
            || current_height != layout.size.height
            || current_x != layout.location.x
            || current_y != layout.location.y
        {
            self.set_needs_layout(true);
        }
        let mut changed = RenderLayerUpdate::default();
        if force_update
            || self.rendering_flags.contains(RenderableFlags::NEEDS_LAYOUT)
            || self.rendering_flags.contains(RenderableFlags::NEEDS_PAINT)
        {
            self.render_layer
                .update_with_model_and_layout(&model, layout, matrix, context_opacity);
            // When this layer clips its children, a child can never put a pixel
            // outside the layer box no matter how big its own buffer is, so the
            // part of the children union that falls outside must not enlarge the
            // subtree rects. We intersect rather than ignoring the children
            // outright, because the visible part of an oversized child is still
            // real geometry: damage and subtree culling need the clipped
            // rectangle, and a child smaller than the parent should not claim the
            // parent's whole box. The clip is against the tight `bounds`, matching
            // what the painter clips to (`clip_to_shape`) and what occlusion uses
            // as the child clip rect — deliberately NOT the shadow-inflated rect,
            // since the parent's shadow is drawn behind the parent, not somewhere
            // a child may paint.
            let mut children_local = local_children_bounds;
            if self.render_layer.clip_children
                && !children_local.intersect(self.render_layer.bounds)
            {
                children_local = skia::Rect::new_empty();
            }

            // `update_with_model_and_layout` has already seeded all three
            // `*_with_children` rects with this layer's own bounds grown to cover
            // its drop shadow. Join into that seed instead of overwriting it: the
            // shadow legitimately paints outside `bounds` and must stay in the
            // damage rects, and clipping the children must not shrink it away.

            // bounds_with_children: union in this node's local space
            self.render_layer.bounds_with_children.join(children_local);

            // local_transformed_bounds_with_children: union in parent-of-this-node space
            let (children_in_parent_space, _) = self
                .render_layer
                .local_transform
                .to_m33()
                .map_rect(children_local);
            self.render_layer
                .local_transformed_bounds_with_children
                .join(children_in_parent_space);
            // global_transformed_bounds_with_children: map final local union through global transform
            let (global_bwc, _) = self
                .render_layer
                .transform_33
                .map_rect(self.render_layer.bounds_with_children);
            self.render_layer.global_transformed_bounds_with_children = global_bwc;
            changed.size_changed = current_width != self.render_layer.size.width
                || current_height != self.render_layer.size.height;
            changed.moved = current_x != self.render_layer.local_transformed_bounds.x()
                || current_y != self.render_layer.local_transformed_bounds.y();
        }
        self.render_layer.visible = !is_hidden && self.render_layer.has_visible_drawables();
        changed
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
        self.rendering_flags.contains(RenderableFlags::NEEDS_PAINT)
    }
    pub fn needs_layout(&self) -> bool {
        self.rendering_flags.contains(RenderableFlags::NEEDS_LAYOUT)
    }
    pub fn pointer_events(&self) -> bool {
        self.render_layer.pointer_events
    }
    pub fn contains_point(&self, point: &skia::Point) -> bool {
        self.render_layer.global_transformed_bounds.contains(point)
    }
    /// Returns the current frame number for this node, incremented each time the node is repainted.
    pub fn frame_number(&self) -> usize {
        self.frame_number
    }
}

impl SceneNodeRenderable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_cached_picture(&self) -> Option<&DrawCache> {
        self.draw_cache.as_ref()
    }
}
/// generate the SkPicture from drawing the Renderlayer
/// if the layer is not hidden
/// if the layer has opacity
/// returns the damaged Rect of from drawing the layer, in layers coordinates
#[profiling::function]
pub fn do_repaint(
    renderable: &SceneNodeRenderable,
    scene_node: &SceneNode,
    pending_damage: Option<skia_safe::Rect>,
) -> SceneNodeRenderable {
    let mut damage = skia_safe::Rect::default();
    let render_layer = &scene_node.render_layer;
    let mut new_renderable = renderable.clone();
    if scene_node.hidden() || render_layer.premultiplied_opacity == 0.0 {
        new_renderable.repaint_damage = damage;
        // Nothing to record for an invisible layer — but the caller clears
        // NEEDS_PAINT all the same, so a recorded picture would survive as the
        // only trace of the state the layer had when it was last visible.
        // Anything changed while it was invisible (its shape, its content)
        // would then be replayed from that stale picture the moment the layer
        // is shown again, under an otherwise up-to-date transform. Drop the
        // cache instead: `update_node` re-records on the next update, because a
        // node with no draw cache always repaints.
        new_renderable.draw_cache = None;
        return new_renderable;
    }

    // Re-run the draw closure to get content damage and record the content
    // picture. The recorded picture is stored in `content_cache` and later
    // replayed by `draw_layer` instead of re-invoking the closure — otherwise
    // the closure would be called twice per repaint (once here, once from
    // `draw_layer_to_picture` → `draw_layer` at drawing/layer.rs:139).
    let mut content_only = false;
    if render_layer.content_draw_func.is_some() {
        let content_draw_func = render_layer.content_draw_func.clone();
        let size = render_layer.size;
        if let Some(draw_func) = content_draw_func {
            let mut recorder = skia_safe::PictureRecorder::new();
            let canvas =
                recorder.begin_recording(skia_safe::Rect::from_wh(size.width, size.height), true);
            let caller = draw_func.0.as_ref();
            let content_damage = caller(canvas, size.width, size.height);
            damage.join(content_damage);
            new_renderable.content_cache = recorder.finish_recording_as_picture(None);
            // If the draw cache already exists (layer was previously painted)
            // and the draw closure returned a damage rect, only that content
            // region changed — no need to report the full layer bounds.
            content_only = renderable.draw_cache.is_some() && !content_damage.is_empty();
        }
    }

    // Union externally reported damage (case 3: content whose damage source
    // is outside the draw closure, e.g. Wayland surface buffer damage).
    // Same contract as the closure return rect: layer-local coordinates,
    // unioned into `damage`, and enables `content_only` so the repaint
    // doesn't degrade to full layer bounds when a cache already exists.
    if let Some(pending) = pending_damage {
        if !pending.is_empty() {
            damage.join(pending);
            if renderable.draw_cache.is_some() {
                content_only = true;
            }
        }
    }

    let (picture, layer_damage) = draw_layer_to_picture(render_layer, &new_renderable);
    // Only join full layer damage when the layer itself changed (first paint,
    // background/border/shadow change).  When only content was updated, the
    // draw closure's damage rect is sufficient.
    if !content_only {
        damage.join(layer_damage);
    }

    if let Some(picture) = picture {
        // update or create the draw cache
        if let Some(draw_cache) = &mut new_renderable.draw_cache {
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
            new_renderable.draw_cache = Some(new_cache);
        }
        let previous_damage = new_renderable.repaint_damage;
        new_renderable.repaint_damage = damage;
        damage.join(previous_damage);
    }
    // }
    new_renderable
}
