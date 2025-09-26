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
    pub(crate) _debug_info: Option<DrawDebugInfo>,
    pub(crate) frame_number: usize,
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
        }
    }
}

/// Contains the outputs of drawing the layer: cache, damage, and flags
#[derive(Clone)]
pub struct SceneNodeRenderable {
    pub(crate) repaint_damage: skia_safe::Rect,
    pub(crate) draw_cache: Option<DrawCache>,
    pub(crate) content_cache: Option<Picture>,
}

impl Default for SceneNodeRenderable {
    fn default() -> Self {
        Self {
            repaint_damage: skia_safe::Rect::default(),
            draw_cache: None,
            content_cache: None,
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
    pub fn set_picture_cached(&mut self, value: bool) {
        self.picture_cached = value;
    }
    pub fn is_picture_cached(&self) -> bool {
        self.picture_cached
    }
    pub fn render_layer(&self) -> &RenderLayer {
        &self.render_layer
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
    /// update the renderlayer based on model and layout
    #[profiling::function]
    pub(crate) fn update_render_layer_if_needed(
        &mut self,
        layout: &Layout,
        model: Arc<ModelLayer>,
        matrix: Option<&M44>,
        context_opacity: f32,
        local_children_bounds: skia_safe::Rect,
    ) -> bool {
        if self.hidden() {
            return false;
        }
        let current_width = self.render_layer.size.width;
        let current_height = self.render_layer.size.height;
        let current_x = self.render_layer.local_transformed_bounds.x();
        let current_y = self.render_layer.local_transformed_bounds.y();
        if current_width != layout.size.width as f32
            || current_height != layout.size.height as f32
            || current_x != layout.location.x as f32
            || current_y != layout.location.y as f32
        {
            self.set_needs_layout(true);
        }
        let mut changed = false;
        if self.rendering_flags.contains(RenderableFlags::NEEDS_LAYOUT) {
            self.render_layer
                .update_with_model_and_layout(&model, layout, matrix, context_opacity);
            // bounds_with_children: union in this node's local space
            self.render_layer.bounds_with_children = self.render_layer.bounds;
            self.render_layer
                .bounds_with_children
                .join(local_children_bounds);

            // local_transformed_bounds_with_children: union in parent space
            // local_transformed_bounds_with_children: union in parent-of-this-node space
            let (children_in_parent_space, _) = self
                .render_layer
                .local_transform
                .to_m33()
                .map_rect(local_children_bounds);
            self.render_layer.local_transformed_bounds_with_children =
                self.render_layer.local_transformed_bounds;
            self.render_layer
                .local_transformed_bounds_with_children
                .join(children_in_parent_space);

            let (_children_in_global_space, _) = self
                .render_layer
                .transform_33
                .map_rect(local_children_bounds);
            // global_transformed_bounds_with_children: map final local union through global transform
            let (global_bwc, _) = self
                .render_layer
                .transform_33
                .map_rect(self.render_layer.bounds_with_children);
            self.render_layer.global_transformed_bounds_with_children = global_bwc;
            changed = current_width != self.render_layer.size.width
                || current_height != self.render_layer.size.height
                || current_x != self.render_layer.local_transformed_bounds.x()
                || current_y != self.render_layer.local_transformed_bounds.y();
        }
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
        let needs_repaint = self.rendering_flags.contains(RenderableFlags::NEEDS_PAINT);

        needs_repaint
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
pub fn do_repaint(renderable: &SceneNodeRenderable, scene_node: &SceneNode) -> SceneNodeRenderable {
    let mut damage = skia_safe::Rect::default();
    let render_layer = &scene_node.render_layer;
    let mut new_renderable = renderable.clone();
    if scene_node.hidden() || render_layer.premultiplied_opacity == 0.0 {
        new_renderable.repaint_damage = damage;
        return new_renderable;
    }

    // if scene_node.is_picture_cached() {
    // disable content cache as there is a bug with rendering images
    if render_layer.content_draw_func.is_some() {
        let content_draw_func = render_layer.content_draw_func.clone();
        let size = render_layer.size;
        if let Some(draw_func) = content_draw_func {
            //         // only redraw if the content changed or the size changed
            //         // if renderable.content_cache.is_none()
            //         // || ((scene_node.size != size)
            //         //     || (self.content_draw_func.as_ref() != content_draw_func))
            //         // {
            let mut recorder = skia_safe::PictureRecorder::new();
            let canvas =
                recorder.begin_recording(skia_safe::Rect::from_wh(size.width, size.height), false);
            // let draw_func = content_draw_func;
            let caller = draw_func.0.as_ref();
            let content_damage = caller(canvas, size.width, size.height);
            damage.join(content_damage);
            //         new_renderable.content_cache = recorder.finish_recording_as_picture(None);
        }
    }
    // }
    let (picture, layer_damage) = draw_layer_to_picture(render_layer, &new_renderable);
    // Don't transform here - let the caller handle coordinate transformation
    damage.join(layer_damage);

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
