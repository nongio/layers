#![allow(warnings)]

use indextree::{Arena, NodeId};
use skia::{gpu::ganesh::gl::direct_contexts, FontStyle, Surface};
use skia_safe::Canvas;
use skia_safe::Contains;
use skia_safe::RoundOut;

use crate::{
    engine::{
        draw_to_picture::DrawDebugInfo,
        node::{SceneNode, SceneNodeRenderable},
        scene::Scene,
        storage::{FlatStorage, FlatStorageData, TreeStorageId},
        NodeRef,
    },
    layers::layer::render_layer::{self, RenderLayer},
    types::Color,
    utils::{self, save_image},
};

use super::layer::{draw_debug, draw_layer};
use std::{
    collections::{HashMap, HashSet},
    iter::IntoIterator,
};

/// Manual filter-chain downsample factor (scale-down → blur → scale-up).
///
/// Now that the backdrop downscaling is done by the save_layer itself via
/// [`set_backdrop_scale`] (Skia's experimental small-save-layer feature), the filter
/// chain blurs at full resolution — set to 1.0 so we don't downscale twice. The
/// `matrix_transform` stages collapse to identity and the blur uses the full sigma.
pub(crate) const BACKGROUND_BLUR_DOWNSAMPLE: f32 = 1.0;

/// Backdrop snapshot scale for the `BackgroundBlur` save_layer.
///
/// Passed to Skia's experimental `fExperimentalBackdropScale`: the backdrop is captured
/// and filtered into an offscreen buffer this much smaller in each dimension (so
/// `BACKGROUND_BLUR_SAVE_SCALE²` of the pixels). Since the result is heavily blurred the
/// reduced resolution is imperceptible, but the layer is far cheaper to allocate and blur.
pub(crate) const BACKGROUND_BLUR_SAVE_SCALE: f32 = 0.1;

/// Sets the experimental backdrop downscale factor on a [`skia_safe::canvas::SaveLayerRec`].
///
/// skia-safe exposes no setter for `SkCanvas::SaveLayerRec::fExperimentalBackdropScale`
/// (the field is private and `NativeAccess` is not re-exported), so we mirror the
/// `#[repr(C)]` layout and write the trailing field directly. The compile-time size
/// assertion guards against the upstream layout drifting. Matches skia-safe 0.93.
fn set_backdrop_scale(rec: &mut skia_safe::canvas::SaveLayerRec, scale: f32) {
    // Mirror of skia_safe::canvas::SaveLayerRec (skia-safe 0.93, #[repr(C)]).
    // filters is an SkSpan = { ptr, len }.
    #[repr(C)]
    struct SaveLayerRecLayout<'a> {
        bounds: Option<&'a skia_safe::Rect>,
        paint: Option<&'a skia_safe::Paint>,
        filters_ptr: *const std::ffi::c_void,
        filters_len: usize,
        backdrop: *const std::ffi::c_void,
        backdrop_tile_mode: u32,
        color_space: *const std::ffi::c_void,
        flags: skia_safe::canvas::SaveLayerFlags,
        experimental_backdrop_scale: f32,
    }
    // Fail the build if the upstream layout ever drifts from our mirror.
    //
    // CAVEAT: this only catches *size* changes. If a future skia-safe version reorders
    // fields, or swaps two same-sized fields, the total size is unchanged and this
    // assertion still passes — yet we'd write `scale` into the wrong field. This is why
    // skia-safe is pinned to `=0.93` in Cargo.toml: a version bump is the only thing that
    // can change the upstream layout, so re-check the real `SaveLayerRec` definition (and
    // this mirror) whenever that pin moves.
    const _: () = assert!(
        std::mem::size_of::<SaveLayerRecLayout<'static>>()
            == std::mem::size_of::<skia_safe::canvas::SaveLayerRec<'static>>()
    );
    // SAFETY: identical #[repr(C)] layout (size asserted above); we only write the trailing f32.
    let mirror = unsafe { &mut *(rec as *mut _ as *mut SaveLayerRecLayout) };
    mirror.experimental_backdrop_scale = scale;
}

// Thread-local caches so the filter objects are built once per rendering thread
// and reused every frame. Skia ImageFilters are immutable ref-counted descriptors
// (no GPU state), so cloning is just an atomic ref-count bump.
thread_local! {
    static BACKDROP_FILTER_CACHE: std::cell::RefCell<Option<skia_safe::ImageFilter>> =
        const { std::cell::RefCell::new(None) };
    static BACKDROP_FILTER_VIBRANCY_CACHE: std::cell::RefCell<Option<skia_safe::ImageFilter>> =
        const { std::cell::RefCell::new(None) };
}

/// Returns the backdrop filter for `BackgroundBlur`, creating it on first call per thread.
///
/// The filter chain is: scale_down → blur (smaller kernel) → scale_up [→ vibrancy].
/// Parameters are derived from `BACKGROUND_BLUR_SIGMA` and `BACKGROUND_BLUR_DOWNSAMPLE`.
fn backdrop_filter(apply_vibrancy: bool) -> Option<skia_safe::ImageFilter> {
    let cache = if apply_vibrancy {
        &BACKDROP_FILTER_VIBRANCY_CACHE
    } else {
        &BACKDROP_FILTER_CACHE
    };
    cache.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            *opt = build_backdrop_filter(BACKGROUND_BLUR_SIGMA, apply_vibrancy);
        }
        opt.clone()
    })
}

/// Builds the backdrop `ImageFilter` from scratch.
///
/// Prefer `backdrop_filter()` over calling this directly so the result is cached.
fn build_backdrop_filter(blur_sigma: f32, apply_vibrancy: bool) -> Option<skia_safe::ImageFilter> {
    let s = BACKGROUND_BLUR_DOWNSAMPLE;

    // Scale the backdrop down, blur with a proportionally smaller sigma, then scale back
    // up. The save_layer bounds already constrain the output region so no explicit
    // crop_rect is needed. When `s == 1.0` the scale stages are identity and the blur
    // runs at full resolution — downsampling is delegated to the save_layer's backdrop
    // scale (see [`set_backdrop_scale`]) instead, so skip building the no-op transforms.
    let blurred = if s == 1.0 {
        skia_safe::image_filters::blur(
            (blur_sigma, blur_sigma),
            skia_safe::TileMode::Mirror,
            None,
            None,
        )?
    } else {
        let sampling = skia_safe::SamplingOptions::new(
            skia_safe::FilterMode::Linear,
            skia_safe::MipmapMode::None,
        );
        let scale_down = skia_safe::image_filters::matrix_transform(
            &skia_safe::Matrix::scale((s, s)),
            sampling,
            None,
        )?;
        let blur = skia_safe::image_filters::blur(
            (blur_sigma * s, blur_sigma * s),
            skia_safe::TileMode::Mirror,
            scale_down,
            None,
        )?;
        skia_safe::image_filters::matrix_transform(
            &skia_safe::Matrix::scale((1.0 / s, 1.0 / s)),
            sampling,
            blur,
        )?
    };

    if apply_vibrancy {
        // Add a mild tone map to feel more "material".
        // Slightly increase contrast and saturation.
        let sat = 1.10_f32;
        let con = 1.06_f32;

        let matrix = skia_safe::ColorMatrix::new(
            con * (0.213 + 0.787 * sat),
            con * (0.715 - 0.715 * sat),
            con * (0.072 - 0.072 * sat),
            0.0,
            0.0,
            con * (0.213 - 0.213 * sat),
            con * (0.715 + 0.285 * sat),
            con * (0.072 - 0.072 * sat),
            0.0,
            0.0,
            con * (0.213 - 0.213 * sat),
            con * (0.715 - 0.715 * sat),
            con * (0.072 + 0.928 * sat),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
        );

        let tone_filter = skia_safe::color_filters::matrix(&matrix, None);

        // Apply vibrancy on top of the upscaled blur.
        skia_safe::image_filters::color_filter(tone_filter, blurred, None)
    } else {
        Some(blurred)
    }
}

/// Cross-buffer backdrop source used by [`render_subtrees_to_buffers`].
///
/// When a subtree is rendered into its own isolated buffer the canvas behind a
/// `BackgroundBlur` layer is empty, so the layer would blur nothing. This carries
/// the composited scene rendered so far (`image`, in global/scene coordinates)
/// plus the global `origin` of the buffer currently being rendered, so the blur
/// path can seed the real composition behind the layer before blurring it.
#[derive(Clone, Copy)]
pub struct ExternalBackdrop<'a> {
    /// The composition of every plane below the current one, in scene/global space.
    pub image: &'a skia_safe::Image,
    /// Global top-left of the buffer being rendered (`global − origin == buffer`).
    pub origin: skia_safe::Point,
}

pub trait DrawScene {
    fn draw_scene(
        &self,
        scene: std::sync::Arc<Scene>,
        root_id: NodeRef,
        damage: Option<skia_safe::Rect>,
    );
}

/// Draw the scene to the given skia::Canvas
pub fn draw_scene(canvas: &skia::Canvas, scene: std::sync::Arc<Scene>, root_id: NodeRef) {
    scene.with_arena(|scene_arena| {
        scene.with_renderable_arena(|renderables_arena| {
            if let Some(root) = scene_arena.get(root_id.into()) {
                let node = root.get();
                let restore_point = canvas.save();
                set_node_transform(node, canvas);
                render_node_tree(
                    root_id,
                    scene_arena,
                    renderables_arena,
                    canvas,
                    1.0,
                    None,
                    None,
                    None,
                );
                canvas.restore_to_count(restore_point);
            }
        });
    });
}

pub fn node_tree_list(
    node_ref: NodeRef,
    arena: &Arena<SceneNode>,
    context_opacity: f32,
) -> Vec<(NodeRef, f32)> {
    let mut nodes = Vec::new();
    let node_id: TreeStorageId = node_ref.into();

    let node = arena.get(node_id).unwrap().get();
    let context_opacity = node.render_layer.opacity * context_opacity;
    if !node.hidden() && context_opacity > 0.0 {
        nodes.push((node_ref, context_opacity));
        let children = node_id.children(arena).collect::<Vec<NodeId>>();
        for child_id in children.iter() {
            let child_ref = NodeRef(child_id.clone());

            nodes.extend(node_tree_list(child_ref, arena, context_opacity));
        }
    }
    nodes
}

pub fn node_tree_list_visible<'a>(
    nodes: impl std::iter::DoubleEndedIterator<Item = &'a (NodeRef, f32)>,
    arena: &Arena<SceneNode>,
) -> Vec<(NodeRef, f32)> {
    let mut visible_nodes = Vec::new();
    let mut damage = Vec::<skia_safe::RRect>::new();

    for (node_ref, context_opacity) in nodes.into_iter().rev() {
        let node_id: TreeStorageId = node_ref.clone().into();
        let node = arena.get(node_id).unwrap().get();
        let rbounds = node.render_layer.global_transformed_rbounds;
        let bounds = node.render_layer.global_transformed_bounds;

        let is_covered = damage.iter().any(|rect| rect.contains(bounds));
        // If the rectangle is not completely covered, add the node to visible_nodes
        if !is_covered {
            visible_nodes.push((node_ref.clone(), context_opacity.clone()));

            if context_opacity.to_bits() == 1_f32.to_bits()
                && node.render_layer.blend_mode != crate::prelude::BlendMode::BackgroundBlur
            {
                damage.push(rbounds);
            }
        }
    }
    visible_nodes
}

use std::sync::{Mutex, Once};

static INIT_SURFACES: Once = Once::new();
static mut NODE_SURFACES: Option<Mutex<HashMap<NodeRef, (usize, Surface, skia::Image)>>> = None;

fn surface_offset_for_render_layer(render_layer: &RenderLayer) -> skia_safe::Point {
    skia_safe::Point::new(
        render_layer.border_width / 2.0,
        render_layer.border_width / 2.0,
    )
}
fn surface_size_for_render_layer(render_layer: &RenderLayer) -> skia_safe::Point {
    let bounds = render_layer
        .bounds_with_children
        .with_outset((render_layer.border_width, render_layer.border_width));
    skia_safe::Point::new(bounds.width(), bounds.height())
}
pub fn set_surface_for_node(
    node_ref: &NodeRef,
    surface: Surface,
    image: skia::Image,
    frame: usize,
) {
    unsafe {
        if let Some(ref surfaces) = NODE_SURFACES {
            let mut surfaces = surfaces.lock().unwrap();
            surfaces.insert(node_ref.clone(), (frame, surface, image));
        }
    }
}
pub fn surface_for_node(
    node_ref: &NodeRef,
    node: &SceneNode,
    render_layer: &RenderLayer,
    context: &mut skia_safe::gpu::DirectContext,
) -> Option<(usize, Surface, skia::Image)> {
    INIT_SURFACES.call_once(|| unsafe {
        NODE_SURFACES = Some(Mutex::new(HashMap::new()));
    });

    unsafe {
        if let Some(ref surfaces) = NODE_SURFACES {
            let mut surfaces = surfaces.lock().unwrap();
            if let Some((frame, surface, image)) = surfaces.get(node_ref) {
                if surface.direct_context().unwrap().id() == context.id() {
                    let size = surface_size_for_render_layer(render_layer);
                    if surface.width() >= size.x as i32 && surface.height() >= size.y as i32 {
                        return Some((*frame, surface.clone(), image.clone()));
                    } else {
                        // Surface size is not enough, remove the surface
                        drop(surface);
                        surfaces.remove(&node_ref);
                    }
                } else {
                    // Surface is not created with the current context, remove the surface
                    drop(surface);
                    surfaces.remove(&node_ref);
                }
            }
            // Create a new surface for the node
            let mut surface = create_surface_for_node(node, render_layer, context)?;
            let image = surface.image_snapshot();

            surfaces.insert(node_ref.clone(), (0, surface.clone(), image.clone()));
            return Some((0, surface, image));
        }
    }
    None
}
pub fn create_surface_for_node(
    node: &SceneNode,
    render_layer: &RenderLayer,
    context: &mut skia_safe::gpu::DirectContext,
) -> Option<Surface> {
    let bounds = surface_size_for_render_layer(render_layer);
    const safe_multiplier: f32 = 1.2;
    let width = (bounds.x * safe_multiplier) as i32;
    let height = (bounds.y * safe_multiplier) as i32;
    if width == 0 || height == 0 {
        // tracing::warn!(
        //     "Invalid size for surface {:?} [{:?}]",
        //     node.id().unwrap(),
        //     bounds
        // );
        return None;
    }

    let image_info = skia_safe::ImageInfo::new(
        (width, height),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );

    let surface = skia_safe::gpu::surfaces::render_target(
        context,
        skia_safe::gpu::Budgeted::No,
        &image_info,
        None,
        skia_safe::gpu::SurfaceOrigin::TopLeft,
        None,
        false,
        false,
    )
    .unwrap();

    Some(surface)
}

#[profiling::function]
/// paint a node and his subtree in the provided canvas
pub fn paint_node_tree(
    node_ref: NodeRef,
    scene_arena: &Arena<SceneNode>,
    renderables_arena: &FlatStorageData<SceneNodeRenderable>,
    render_canvas: &skia_safe::Canvas,
    render_layer: &RenderLayer,
    context_opacity: f32,
    offscreen: bool,
    dbg_info: Option<&DrawDebugInfo>,
    skip_self: bool,
    occluded: Option<&HashSet<NodeRef>>,
    damage_region: Option<&skia_safe::Region>,
    external_backdrop: Option<ExternalBackdrop>,
) {
    let node_id: TreeStorageId = node_ref.into();

    if !skip_self {
        paint_node(
            node_ref,
            scene_arena,
            renderables_arena,
            render_canvas,
            context_opacity,
            offscreen,
            damage_region,
            external_backdrop,
        );
        if let Some(dbg_info) = dbg_info {
            draw_debug(render_canvas, dbg_info, render_layer);
        }
    }
    let mut context_opacity = render_layer.opacity * context_opacity;
    if (offscreen) {
        context_opacity = render_layer.opacity;
    }
    // TODO: clip bounds only if the layer is set to clip children
    let restore_point = render_canvas.save();
    if render_layer.clip_children {
        render_layer.clip_to_shape(render_canvas, skia_safe::ClipOp::Intersect, true);
    }
    // let bounds = skia_safe::Rect::from_wh(render_layer.size.x, render_layer.size.y);
    // canvas.clip_rect(bounds, None, None);
    node_id.children(scene_arena).for_each(|child_id| {
        let child_ref = NodeRef(child_id);
        let child = scene_arena.get(child_id).unwrap().get();

        // Damage-based subtree culling: if the child (including all its
        // descendants) falls entirely outside the damage region, skip it.
        if let Some(region) = damage_region {
            let child_bounds = child.render_layer.global_transformed_bounds_with_children;
            let irect: skia_safe::IRect = child_bounds.round_out();
            if !region.intersects_rect(irect) {
                return;
            }
        }

        let restore_point = render_canvas.save();
        set_node_transform(child, render_canvas);
        render_node_tree(
            child_ref,
            scene_arena,
            renderables_arena,
            render_canvas,
            context_opacity,
            occluded,
            damage_region,
            external_backdrop,
        );
        render_canvas.restore_to_count(restore_point);
    });
    render_canvas.restore_to_count(restore_point);
}

pub fn set_node_transform(node: &SceneNode, canvas: &Canvas) {
    let transform = node.render_layer.local_transform.to_m33();
    canvas.concat(&transform);
}

/// Render a node and his subtree in the provided canvas
/// taking care of the image caching if needed
#[profiling::function]
pub fn render_node_tree(
    node_ref: NodeRef,
    scene_arena: &Arena<SceneNode>,
    renderables_arena: &FlatStorageData<SceneNodeRenderable>,
    render_canvas: &skia_safe::Canvas,
    context_opacity: f32,
    occluded: Option<&HashSet<NodeRef>>,
    damage_region: Option<&skia_safe::Region>,
    external_backdrop: Option<ExternalBackdrop>,
) {
    let node_id: TreeStorageId = node_ref.into();
    #[cfg(feature = "profile-with-puffin")]
    profiling::puffin::profile_scope!("render_node_tree", format!("{}", node_id));
    let scene_node = scene_arena.get(node_id);
    if (scene_node.is_none()) {
        return;
    }
    let node = scene_node.unwrap();
    if (node.is_removed()) {
        return;
    }
    let scene_node = node.get();
    if scene_node.hidden() {
        return;
    }
    // Skip fully occluded nodes' own painting, but still traverse children.
    // A child might be the occluder of its parent, so we cannot skip the subtree.
    let is_self_occluded = occluded.map_or(false, |set| set.contains(&node_ref));

    let render_layer = &scene_node.render_layer;
    let restore_point = render_canvas.save();
    // render_canvas.concat(&render_layer.local_transform.to_m33());
    let dbg_info = scene_node._debug_info.as_ref();
    if scene_node.is_image_cached() && !is_self_occluded {
        #[cfg(feature = "profile-with-puffin")]
        profiling::puffin::profile_scope!("image_cached");
        // in this case the layer is ping-ponged
        if let Some(rendering_surface) = unsafe { render_canvas.surface() } {
            if let Some(mut recording_ctx) = rendering_surface.recording_context() {
                if let Some((recorded_frame, mut recording_surface, mut image)) = surface_for_node(
                    &node_ref,
                    scene_node,
                    &render_layer,
                    &mut recording_ctx.as_direct_context().unwrap(),
                ) {
                    // if there is a surface for image caching draw in it
                    let surface_size = surface_size_for_render_layer(&render_layer);
                    let surface_offset = surface_offset_for_render_layer(&render_layer);

                    let surface_position_x = render_layer.bounds_with_children.x();
                    let surface_position_y = render_layer.bounds_with_children.y();

                    let bounds = render_layer.bounds;

                    let offset = skia_safe::M44::translate(
                        surface_offset.x - surface_position_x,
                        surface_offset.y - surface_position_y,
                        0.0,
                    );

                    // draw into the offscreen surface
                    let current_frame = scene_node.frame_number;

                    // If this node is following another node (replicate_node),
                    // we need to check BOTH:
                    // 1. Has the follower itself changed? (its own frame)
                    // 2. Has the followed node's content changed? (source frame)
                    let needs_repaint = if let Some(following_ref) = scene_node.following {
                        // Guard against a freed leader node — indextree's Node::get() panics
                        // on freed slots, so we must check is_removed() first.
                        let followed_frame = scene_arena
                            .get(following_ref.0)
                            .filter(|n| !n.is_removed())
                            .map(|n| n.get().frame_number)
                            .unwrap_or(0);
                        // Repaint if either the follower or the followed node changed
                        current_frame != recorded_frame || followed_frame != recorded_frame
                    } else {
                        current_frame != recorded_frame
                    };

                    if needs_repaint || dbg_info.is_some() {
                        let surface_bounds = skia::Rect::from_wh(
                            recording_surface.width() as f32,
                            recording_surface.height() as f32,
                        );
                        let recording_canvas = recording_surface.canvas();
                        // recording_canvas.clip_rect(scene_node.bounds_with_children(), Some(skia::ClipOp::Intersect), false);

                        recording_canvas.clear(skia_safe::Color::TRANSPARENT);
                        recording_canvas.save();
                        recording_canvas.set_matrix(&offset);

                        paint_node_tree(
                            node_ref,
                            scene_arena,
                            renderables_arena,
                            &recording_canvas,
                            &render_layer,
                            context_opacity,
                            true,
                            dbg_info,
                            false, // not self-occluded (checked above)
                            occluded,
                            None, // damage is in screen-space, not offscreen-surface-space
                            // The offscreen surface has its own matrix; the global-space
                            // accumulator can't be aligned here, so cross-buffer blur is
                            // not applied to image-cached subtrees' descendants.
                            None,
                        );
                        // debug drawing
                        // let mut paint = skia_safe::Paint::default();
                        // paint.set_color4f(Color::new_hex("#27AC12").c4f(), None);
                        // paint.set_stroke(true);
                        // paint.set_stroke_width(4.0);
                        // recording_canvas.draw_rect(render_layer.bounds_with_children, &paint);
                        // let font_mgr = skia_safe::FontMgr::new();
                        // let typeface = font_mgr
                        //     .match_family_style("Inter", FontStyle::normal())
                        //     .unwrap();
                        // let font = skia::Font::from_typeface_with_params(typeface, 20.0, 1.0, 0.0);
                        // paint.set_stroke(false);
                        // recording_canvas.draw_str(
                        //     format!("{} | {}", render_layer.opacity, current_frame),
                        //     (
                        //         render_layer.bounds_with_children.x() + 5.0,
                        //         render_layer.bounds_with_children.y() + 25.0,
                        //     ),
                        //     &font,
                        //     &paint,
                        // );

                        recording_canvas.restore();

                        image = recording_surface.image_snapshot();
                        // Store the current frame as the recorded frame
                        // (we've now rendered with both our state and the followed state)
                        set_surface_for_node(
                            &node_ref,
                            recording_surface.clone(),
                            image.clone(),
                            current_frame,
                        );
                    } // end draw into the offscreen surface

                    let mut paint = skia_safe::Paint::default();
                    paint.set_color4f(skia_safe::Color4f::new(1.0, 0.0, 0.0, 1.0), None);

                    let width = recording_surface.width() as f32;
                    let height = recording_surface.height() as f32;
                    let x = render_layer.bounds.x();
                    let y = render_layer.bounds.y();

                    paint.set_alpha_f(context_opacity * render_layer.opacity);

                    if let Some(filter) = render_layer.image_filter.as_ref() {
                        if let Some(filter_bounds) = render_layer.image_filter_bounds.as_ref() {
                            render_canvas.clip_rect(&filter_bounds, None, None);
                        }
                        paint.set_image_filter(filter.clone());
                    }
                    if let Some(filter) = render_layer.color_filter.as_ref() {
                        if let Some(filter_bounds) = render_layer.image_filter_bounds.as_ref() {
                            render_canvas.clip_rect(&filter_bounds, None, None);
                        }
                        paint.set_color_filter(filter.clone());
                    }
                    // the render_canvas has already the transform applied
                    let x = render_layer.bounds_with_children.x() - surface_offset.x;
                    let y = render_layer.bounds_with_children.y() - surface_offset.y;

                    let opacity = context_opacity * render_layer.opacity;

                    // Apply backdrop effects for descendant regions that need it
                    // Converts Vec<RRect> to Path for clipping with rounded rectangles
                    if let Some(backdrop_rrects) = &render_layer.backdrop_blur_region {
                        profiling::scope!("background_blur_image_cached_descendants");

                        let before_backdrop = render_canvas.save();

                        // Build a path from all the rounded rects
                        let mut backdrop_builder = skia_safe::PathBuilder::new();
                        for rrect in backdrop_rrects {
                            backdrop_builder.add_rrect(*rrect, None, 0);
                        }
                        let backdrop_path = backdrop_builder.snapshot();

                        // Clip to the backdrop path (supports rounded rects)
                        render_canvas.clip_path(&backdrop_path, skia_safe::ClipOp::Intersect, true);

                        let path_bounds = backdrop_path.bounds();

                        // Use cached filter — same vibrancy as direct-rendered layers.
                        // The save_layer bounds constrains the output, no crop_rect needed.
                        if let Some(filter) = backdrop_filter(true) {
                            profiling::scope!("apply backdrop descendants");
                            let mut backdrop_paint = skia_safe::Paint::default();
                            backdrop_paint.set_alpha_f(opacity);
                            let mut save_layer_rec = skia_safe::canvas::SaveLayerRec::default();
                            save_layer_rec =
                                save_layer_rec.bounds(&path_bounds).paint(&backdrop_paint);
                            save_layer_rec = save_layer_rec.backdrop(&filter);
                            set_backdrop_scale(&mut save_layer_rec, BACKGROUND_BLUR_SAVE_SCALE);
                            render_canvas.save_layer(&save_layer_rec);
                        }

                        render_canvas.restore_to_count(before_backdrop);
                    }

                    render_canvas.draw_image_with_sampling_options(
                        &image,
                        (x, y),
                        skia_safe::SamplingOptions::from(skia_safe::CubicResampler::catmull_rom()),
                        Some(&paint),
                    );

                    render_canvas.restore_to_count(restore_point);
                    return;
                }
            }
        }
    }

    // here is when the layer is directly rendered to the screen

    paint_node_tree(
        node_ref,
        scene_arena,
        renderables_arena,
        render_canvas,
        &render_layer,
        context_opacity,
        false,
        dbg_info,
        is_self_occluded,
        occluded,
        damage_region,
        external_backdrop,
    );

    render_canvas.restore_to_count(restore_point);
}
pub(crate) const BACKGROUND_BLUR_SIGMA: f32 = 40.0;

// paint a single node in the provided canvas
#[profiling::function]
pub(crate) fn paint_node(
    node_ref: NodeRef,
    scene_arena: &Arena<SceneNode>,
    renderables_arena: &FlatStorageData<SceneNodeRenderable>,
    canvas: &skia_safe::Canvas,
    context_opacity: f32,
    offscreen: bool,
    damage_region: Option<&skia_safe::Region>,
    external_backdrop: Option<ExternalBackdrop>,
) -> usize {
    let node_id: TreeStorageId = node_ref.into();
    let node = scene_arena.get(node_id).unwrap().get();
    let node_u: usize = node_id.into();
    let node_renderable = renderables_arena.get(&node_u).unwrap();
    let render_layer = &node.render_layer;
    let node_opacity = render_layer.opacity;
    let mut opacity = 1.0;
    if !offscreen {
        opacity = context_opacity * node_opacity;
    }

    let blend_mode = render_layer.blend_mode;
    let restore_transform = canvas.save();
    if render_layer.size.width <= 0.0 || render_layer.size.height <= 0.0 {
        return restore_transform;
    }

    // Skip painting if the node's global bounds don't intersect the damage region
    if let Some(region) = damage_region {
        let global_bounds = render_layer.global_transformed_bounds;
        let irect: skia_safe::IRect = global_bounds.round_out();
        if !region.intersects_rect(irect) {
            return restore_transform;
        }
    }

    let draw_cache = node_renderable.draw_cache.as_ref();

    let before_backdrop = canvas.save();

    let bounds_to_origin =
        skia_safe::Rect::from_xywh(0.0, 0.0, render_layer.size.width, render_layer.size.height);

    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(opacity);

    if blend_mode == crate::prelude::BlendMode::BackgroundBlur && opacity > 0.0 {
        profiling::scope!("background_blur");
        render_layer.clip_to_shape(canvas, skia_safe::ClipOp::Intersect, true);

        // Cross-buffer vibrancy (render_subtrees): when this layer lives in an
        // isolated subtree buffer the pixels behind it are empty, so seed them
        // with the composited scene rendered so far. DstOver places the
        // accumulator *behind* any content this subtree already painted, so the
        // backdrop the blur reads is the true composite: lower planes plus
        // earlier same-subtree content. Confined to the blur clip set above.
        if let Some(backdrop) = external_backdrop {
            profiling::scope!("seed external backdrop");
            let src = render_layer.global_transformed_bounds;
            let mut backdrop_paint = skia_safe::Paint::default();
            backdrop_paint.set_blend_mode(skia_safe::BlendMode::DstOver);
            canvas.draw_image_rect(
                backdrop.image,
                Some((&src, skia_safe::canvas::SrcRectConstraint::Fast)),
                bounds_to_origin,
                &backdrop_paint,
            );
        }

        if let Some(blur) = backdrop_filter(true) {
            profiling::scope!("apply backdrop");
            let mut save_layer_rec = skia_safe::canvas::SaveLayerRec::default();
            save_layer_rec = save_layer_rec.bounds(&bounds_to_origin).paint(&paint);
            save_layer_rec = save_layer_rec.backdrop(&blur);
            set_backdrop_scale(&mut save_layer_rec, BACKGROUND_BLUR_SAVE_SCALE);
            canvas.save_layer(&save_layer_rec);
            canvas.restore_to_count(before_backdrop);
        }
    }
    if node.is_picture_cached() && draw_cache.is_some() {
        let draw_cache = draw_cache.unwrap();
        // Only pass paint when opacity is not 1.0: passing Some(paint) forces Skia
        // to create an intermediate offscreen layer even when alpha is a no-op.
        let p = (opacity != 1.0).then_some(&paint);
        // passing a None for paint is important to optimise
        // skia creates a new layer when painting a picture with a paint
        draw_cache.draw(canvas, p);
    } else {
        draw_layer(canvas, &render_layer, opacity, node_renderable);
    }

    restore_transform
}
/// One rendered subtree, ready to be handed to an external compositor.
///
/// `image` holds ONLY this subtree's own painted content plus the blurred
/// backdrop baked inside any `BackgroundBlur` shapes — the planes below it are
/// not present outside those shapes, so the compositor still owns stacking.
#[derive(Clone)]
pub struct SubtreeBuffer {
    /// The subtree root this buffer was produced from.
    pub root: NodeRef,
    /// The rendered image (snapshot of the per-subtree surface).
    pub image: skia_safe::Image,
    /// Global top-left where the compositor should place `image`.
    pub origin: skia_safe::Point,
    /// Pixel size of `image`.
    pub size: skia_safe::ISize,
    /// Z-order index (0 == bottom-most); equals the index in the input slice.
    pub z_index: usize,
}

/// Edge margin applied to each subtree buffer so blur kernels and clipped
/// children near the bounds aren't cut off. Mirrors `create_surface_for_node`.
const SUBTREE_SAFE_MARGIN: f32 = 1.2;

/// Allocate a drawing surface, GPU-backed when a context is given, else raster.
fn alloc_subtree_surface(
    width: i32,
    height: i32,
    context: Option<&mut skia_safe::gpu::DirectContext>,
) -> Option<Surface> {
    match context {
        Some(ctx) => {
            let image_info = skia_safe::ImageInfo::new(
                (width, height),
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Premul,
                None,
            );
            skia_safe::gpu::surfaces::render_target(
                ctx,
                skia_safe::gpu::Budgeted::No,
                &image_info,
                None,
                skia_safe::gpu::SurfaceOrigin::TopLeft,
                None,
                false,
                false,
            )
        }
        None => skia_safe::surfaces::raster_n32_premul((width, height)),
    }
}

/// Render each subtree `root` into its own independent buffer, in z-order
/// (`roots[0]` is bottom-most). Returns one [`SubtreeBuffer`] per root.
///
/// `BackgroundBlur` layers inside any subtree sample the composite of all lower
/// subtrees plus the content painted earlier within their own subtree
/// (cross-buffer vibrancy): a running "backdrop the composition" accumulator is
/// built bottom-to-top and seeded into each blur layer's shape before blurring.
///
/// Pass a GPU `context` for GPU-backed surfaces, or `None` for raster surfaces.
///
/// Notes:
/// - The input order MUST be a valid z-order; a blur in subtree `j` only sees
///   subtrees rendered before it.
/// - If a subtree root is itself `image_cache(true)`, cross-buffer blur is not
///   applied to its descendants (the offscreen cache has its own coordinate
///   space); render such subtrees without `image_cache` to get vibrancy.
pub fn render_subtrees_to_buffers(
    scene: std::sync::Arc<Scene>,
    roots: &[NodeRef],
    mut context: Option<&mut skia_safe::gpu::DirectContext>,
) -> Vec<SubtreeBuffer> {
    let mut result = Vec::with_capacity(roots.len());

    // Accumulator: the composition rendered so far, in scene/global coordinates.
    let scene_size = *scene.size.read().unwrap_or_else(|e| e.into_inner());
    let acc_w = (scene_size.x.ceil() as i32).max(1);
    let acc_h = (scene_size.y.ceil() as i32).max(1);
    let Some(mut accumulator) = alloc_subtree_surface(acc_w, acc_h, context.as_deref_mut()) else {
        return result;
    };
    accumulator.canvas().clear(skia_safe::Color::TRANSPARENT);

    // Direct arena access (not `with_arena`) so we can return skia images, which
    // don't satisfy the `Send + Sync` bound that the closure-based helpers impose.
    let nodes_arc = scene.nodes.data();
    let renderables_arc = scene.renderables.data();
    let scene_arena = nodes_arc.read().unwrap_or_else(|e| e.into_inner());
    let renderables_arena = renderables_arc.read().unwrap_or_else(|e| e.into_inner());

    for (z_index, root_ref) in roots.iter().enumerate() {
        let node_id: TreeStorageId = (*root_ref).into();
        let Some(node) = scene_arena.get(node_id) else {
            continue;
        };
        if node.is_removed() {
            continue;
        }
        let scene_node = node.get();
        if scene_node.hidden() {
            continue;
        }
        let render_layer = &scene_node.render_layer;

        // Buffer geometry in global space.
        let gbounds = render_layer.global_transformed_bounds_with_children;
        let origin = skia_safe::Point::new(gbounds.x(), gbounds.y());
        let width = (gbounds.width() * SUBTREE_SAFE_MARGIN).ceil() as i32;
        let height = (gbounds.height() * SUBTREE_SAFE_MARGIN).ceil() as i32;
        if width <= 0 || height <= 0 {
            continue;
        }

        let Some(mut buffer) = alloc_subtree_surface(width, height, context.as_deref_mut()) else {
            continue;
        };

        // Render the subtree into its transparent buffer, sampling the
        // accumulator for any BackgroundBlur layers (cross-buffer vibrancy).
        {
            let acc_image = accumulator.image_snapshot();
            let backdrop = ExternalBackdrop {
                image: &acc_image,
                origin,
            };
            let canvas = buffer.canvas();
            canvas.clear(skia_safe::Color::TRANSPARENT);
            let restore_point = canvas.save();
            // buffer-local := global − origin; apply the root's *global* transform
            // so the subtree (and its blur regions) land in global coordinates
            // shifted into the buffer, aligning with the global-space accumulator.
            canvas.translate((-origin.x, -origin.y));
            canvas.concat(&render_layer.transform.to_m33());
            render_node_tree(
                *root_ref,
                &scene_arena,
                &renderables_arena,
                canvas,
                1.0,
                None,
                None,
                Some(backdrop),
            );
            canvas.restore_to_count(restore_point);
        }

        let image = buffer.image_snapshot();

        // Fold this plane into the accumulator (global space) so the next plane
        // up backdrops against the updated composition.
        accumulator
            .canvas()
            .draw_image(&image, (origin.x, origin.y), None);

        result.push(SubtreeBuffer {
            root: *root_ref,
            image,
            origin,
            size: skia_safe::ISize::new(width, height),
            z_index,
        });
    }

    result
}

/// Print the node tree to the console
pub fn print_scene(scene: std::sync::Arc<Scene>, root_id: NodeRef) {
    scene.with_arena(|arena| {
        if let Some(_root) = arena.get(root_id.into()) {
            debug_node_tree(root_id, &arena, 1.0, 0);
        }
    });
}

fn debug_node_tree(
    node_ref: NodeRef,
    arena: &Arena<SceneNode>,
    context_opacity: f32,
    level: usize,
) {
    let node_id: TreeStorageId = node_ref.into();
    let scene_node = arena.get(node_id).unwrap().get();
    if scene_node.hidden() {
        return;
    }
    debug_node(node_ref, arena, context_opacity, level);

    let render_layer = &scene_node.render_layer;
    let context_opacity = render_layer.opacity * context_opacity;
    node_id.children(arena).for_each(|child_id| {
        if !child_id.is_removed(arena) {
            let child_ref = NodeRef(child_id);
            debug_node_tree(child_ref, arena, context_opacity, level + 1);
        }
    });
}

pub fn debug_node(node_id: NodeRef, arena: &Arena<SceneNode>, context_opacity: f32, level: usize) {
    let node_id: TreeStorageId = node_id.into();
    let node = arena.get(node_id).unwrap().get();
    let render_layer = &node.render_layer;

    let bounds =
        skia_safe::Rect::from_xywh(0.0, 0.0, render_layer.size.width, render_layer.size.height);

    println!(
        "{}Layer({}) key: {:?} position: {:?} size: {:?} opacity: {:?}",
        "* ".repeat(level),
        node_id,
        render_layer.key,
        (
            render_layer.global_transformed_bounds.x(),
            render_layer.global_transformed_bounds.y()
        ),
        (
            render_layer.global_transformed_bounds.width(),
            render_layer.global_transformed_bounds.height()
        ),
        render_layer.opacity,
    );
}
