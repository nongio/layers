#![allow(warnings)]

use indextree::{Arena, NodeId};
use skia::{FontStyle, Surface};
use skia_safe::Canvas;
use skia_safe::Contains;

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
    utils,
};

use super::layer::{draw_debug, draw_layer};
use std::{collections::HashMap, iter::IntoIterator};

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
                render_node_tree(root_id, scene_arena, renderables_arena, canvas, 1.0);
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
) {
    let node_id: TreeStorageId = node_ref.into();

    paint_node(
        node_ref,
        scene_arena,
        renderables_arena,
        render_canvas,
        context_opacity,
        offscreen,
    );
    if let Some(dbg_info) = dbg_info {
        draw_debug(render_canvas, dbg_info, render_layer);
    }
    let mut context_opacity = render_layer.opacity * context_opacity;
    if (offscreen) {
        context_opacity = render_layer.opacity;
    }
    // TODO: clip bounds only if the layer is set to clip children
    let restore_point = render_canvas.save();
    if render_layer.clip_children {
        render_canvas.clip_rrect(
            render_layer.rbounds,
            Some(skia_safe::ClipOp::Intersect),
            Some(true),
        );
    }
    // let bounds = skia_safe::Rect::from_wh(render_layer.size.x, render_layer.size.y);
    // canvas.clip_rect(bounds, None, None);
    node_id.children(scene_arena).for_each(|child_id| {
        let child_ref = NodeRef(child_id);
        let restore_point = render_canvas.save();
        let child = scene_arena.get(child_id).unwrap().get();
        set_node_transform(child, render_canvas);
        render_node_tree(
            child_ref,
            scene_arena,
            renderables_arena,
            render_canvas,
            context_opacity,
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
) {
    let node_id: TreeStorageId = node_ref.into();
    #[cfg(feature = "profile-with-puffin")]
    profiling::puffin::profile_scope!("render_node_tree", format!("{}", node_id));
    let scene_node = scene_arena.get(node_id);
    if (scene_node.is_none()) {
        return;
    }
    let scene_node = scene_node.unwrap().get();
    if scene_node.hidden() {
        return;
    }
    let render_layer = &scene_node.render_layer;
    let restore_point = render_canvas.save();
    // render_canvas.concat(&render_layer.local_transform.to_m33());
    let dbg_info = scene_node._debug_info.as_ref();
    if scene_node.is_image_cached() {
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
                    if current_frame != recorded_frame {
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

                    render_canvas.draw_image(&image, (x, y), Some(&paint));

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
    );

    render_canvas.restore_to_count(restore_point);
}
pub(crate) const BACKGROUND_BLUR_SIGMA: f32 = 25.0;

// paint a single node in the provided canvas
#[profiling::function]
pub(crate) fn paint_node(
    node_ref: NodeRef,
    scene_arena: &Arena<SceneNode>,
    renderables_arena: &FlatStorageData<SceneNodeRenderable>,
    canvas: &skia_safe::Canvas,
    context_opacity: f32,
    offscreen: bool,
) -> usize {
    let node_id: TreeStorageId = node_ref.into();
    profiling::scope!("paint_node", format!("{}", node_id));
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

    let draw_cache = node_renderable.draw_cache.as_ref();

    let before_backdrop = canvas.save();

    let bounds_to_origin =
        skia_safe::Rect::from_xywh(0.0, 0.0, render_layer.size.width, render_layer.size.height);

    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(opacity);

    if blend_mode == crate::prelude::BlendMode::BackgroundBlur && opacity > 0.0 {
        profiling::scope!("background_blur");
        let border_corner_radius = render_layer.border_corner_radius;
        let rrbounds = render_layer.rbounds;
        canvas.clip_rrect(rrbounds, skia_safe::ClipOp::Intersect, Some(true));

        let crop_rect = Some(skia_safe::image_filters::CropRect::from(
            bounds_to_origin.with_outset((BACKGROUND_BLUR_SIGMA, BACKGROUND_BLUR_SIGMA)),
        ));

        let blur = skia_safe::image_filters::blur(
            (BACKGROUND_BLUR_SIGMA, BACKGROUND_BLUR_SIGMA),
            skia_safe::TileMode::Clamp,
            None,
            crop_rect,
        );

        // blur can fail
        if let Some(blur) = blur.as_ref() {
            profiling::scope!("apply backdrop");
            let mut save_layer_rec = skia_safe::canvas::SaveLayerRec::default();
            save_layer_rec = save_layer_rec.bounds(&bounds_to_origin).paint(&paint);
            save_layer_rec = save_layer_rec.backdrop(blur);
            canvas.save_layer(&save_layer_rec);
            canvas.restore_to_count(before_backdrop);
        }
    }
    if let Some(draw_cache) = draw_cache {
        let mut p = None;
        if opacity != 1.0
            || (blend_mode == crate::prelude::BlendMode::BackgroundBlur && opacity > 0.0)
        {
            p = Some(&paint);
        }
        // passing a None for paint is important to optimise
        // skia creates a new layer when painting a picture with a paint
        draw_cache.draw(canvas, p);
    } else {
        draw_layer(canvas, &render_layer, opacity, node_renderable);
    }

    restore_transform
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
