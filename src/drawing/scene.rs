#![allow(warnings)]

use indextree::{Arena, NodeId};
use skia_safe::Canvas;
use skia_safe::Contains;

use crate::engine::{
    node::{DrawCacheManagement, SceneNode},
    scene::Scene,
    storage::TreeStorageId,
    NodeRef,
};

use super::layer::draw_layer;
use std::iter::IntoIterator;

pub trait DrawScene {
    fn draw_scene(&self, scene: &Scene, root_id: NodeRef, damage: Option<skia_safe::Rect>);
}

pub fn draw_scene(canvas: &Canvas, scene: &Scene, root_id: NodeRef) {
    let arena = scene.nodes.data();
    let arena = arena.read().unwrap();
    if let Some(_root) = scene.get_node(root_id) {
        render_node_tree(root_id, &arena, canvas, 1.0);
    }
}
pub fn node_tree_list(
    node_ref: NodeRef,
    arena: &Arena<SceneNode>,
    context_opacity: f32,
) -> Vec<(NodeRef, f32)> {
    let mut nodes = Vec::new();
    let node_id: TreeStorageId = node_ref.into();

    let node = arena.get(node_id).unwrap().get();
    let render_layer = node.render_layer.read().unwrap();
    let context_opacity = render_layer.opacity * context_opacity;
    if !node.layer.hidden() && context_opacity > 0.0 {
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
        let render_layer = node.render_layer.read().unwrap();
        let rbounds = render_layer.transformed_rbounds;
        let bounds = render_layer.transformed_bounds;

        let is_covered = damage.iter().any(|rect| rect.contains(bounds));
        // If the rectangle is not completely covered, add the node to visible_nodes
        if !is_covered {
            visible_nodes.push((node_ref.clone(), context_opacity.clone()));

            if context_opacity.to_bits() == 1_f32.to_bits()
                && render_layer.blend_mode != crate::prelude::BlendMode::BackgroundBlur
            {
                damage.push(rbounds);
            }
        }
    }
    visible_nodes
}
pub fn render_node_tree(
    node_ref: NodeRef,
    arena: &Arena<SceneNode>,
    canvas: &skia_safe::Canvas,
    context_opacity: f32,
) {
    #[cfg(feature = "profile-with-puffin")]
    profiling::puffin::profile_scope!("render_node_tree");
    let node_id: TreeStorageId = node_ref.into();
    let scene_node = arena.get(node_id).unwrap().get();
    if scene_node.layer.hidden() {
        return;
    }
    let restore_transform = render_node(node_ref, arena, canvas, context_opacity);

    let render_layer = scene_node.render_layer.read().unwrap();
    let context_opacity = render_layer.opacity * context_opacity;
    // let bounds = skia_safe::Rect::from_wh(render_layer.size.x, render_layer.size.y);
    // canvas.clip_rect(bounds, None, None);
    node_id.children(arena).for_each(|child_id| {
        let child_ref = NodeRef(child_id);
        render_node_tree(child_ref, arena, canvas, context_opacity);
    });

    // canvas.restore_to_count(restore_transform);
}

pub(crate) fn render_node(
    node_id: NodeRef,
    arena: &Arena<SceneNode>,
    canvas: &skia_safe::Canvas,
    context_opacity: f32,
) -> usize {
    let node_id: TreeStorageId = node_id.into();
    let node = arena.get(node_id).unwrap().get();
    let render_layer = node.render_layer.read().unwrap();
    let node_opacity = render_layer.opacity;
    let opacity = context_opacity * node_opacity;

    let blend_mode = render_layer.blend_mode;
    let restore_transform = 0; //canvas.save();
    if render_layer.size.width <= 0.0 || render_layer.size.height <= 0.0 {
        return restore_transform;
    }
    canvas.set_matrix(&render_layer.transform);

    let draw_cache = node.draw_cache.read().unwrap();

    let before_backdrop = canvas.save();

    let bounds_to_origin =
        skia_safe::Rect::from_xywh(0.0, 0.0, render_layer.size.width, render_layer.size.height);

    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(opacity);

    if blend_mode == crate::prelude::BlendMode::BackgroundBlur && opacity > 0.0 {
        let border_corner_radius = render_layer.border_corner_radius;
        let rrbounds = skia_safe::RRect::new_rect_radii(
            bounds_to_origin,
            &[
                skia_safe::Point::new(border_corner_radius.top_left, border_corner_radius.top_left),
                skia_safe::Point::new(
                    border_corner_radius.top_right,
                    border_corner_radius.top_right,
                ),
                skia_safe::Point::new(
                    border_corner_radius.bottom_left,
                    border_corner_radius.bottom_left,
                ),
                skia_safe::Point::new(
                    border_corner_radius.bottom_right,
                    border_corner_radius.bottom_right,
                ),
            ],
        );
        canvas.clip_rrect(rrbounds, skia_safe::ClipOp::Intersect, Some(true));

        let mut save_layer_rec = skia_safe::canvas::SaveLayerRec::default();
        let SAFE_MARGIN = 100.0;
        let crop_rect = Some(skia_safe::image_filters::CropRect::from(
            bounds_to_origin.with_outset((SAFE_MARGIN, SAFE_MARGIN)),
        ));

        let blur = skia_safe::image_filters::blur(
            (25.0, 25.0),
            skia_safe::TileMode::Clamp,
            None,
            crop_rect,
        )
        .unwrap();

        save_layer_rec = save_layer_rec
            .backdrop(&blur)
            .bounds(&bounds_to_origin)
            .paint(&paint);
        canvas.save_layer(&save_layer_rec);
    }
    canvas.restore_to_count(before_backdrop);

    if let Some(draw_cache) = &*draw_cache {
        draw_cache.draw(canvas, &paint);
    }

    // FIXME if the cache is not present we could draw directly yhe layer, but we need to
    // handle the opacity

    //  else {
    // node.set_need_repaint(true);
    // draw_layer(canvas, &render_layer);
    // }

    restore_transform
}

pub fn debug_scene(scene: &Scene, root_id: NodeRef) {
    let arena = scene.nodes.data();
    let arena = arena.read().unwrap();
    if let Some(_root) = scene.get_node(root_id) {
        debug_node_tree(root_id, &arena, 1.0, 0);
    }
}

pub fn debug_node_tree(
    node_ref: NodeRef,
    arena: &Arena<SceneNode>,
    context_opacity: f32,
    level: usize,
) {
    let node_id: TreeStorageId = node_ref.into();
    let scene_node = arena.get(node_id).unwrap().get();
    if scene_node.layer.hidden() {
        return;
    }
    debug_node(node_ref, arena, context_opacity, level);

    let render_layer = scene_node.render_layer.read().unwrap();
    let context_opacity = render_layer.opacity * context_opacity;
    node_id.children(arena).for_each(|child_id| {
        let child_ref = NodeRef(child_id);
        debug_node_tree(child_ref, arena, context_opacity, level + 1);
    });
}

pub fn debug_node(node_id: NodeRef, arena: &Arena<SceneNode>, context_opacity: f32, level: usize) {
    let node_id: TreeStorageId = node_id.into();
    let node = arena.get(node_id).unwrap().get();
    let render_layer = node.render_layer.read().unwrap();

    let bounds =
        skia_safe::Rect::from_xywh(0.0, 0.0, render_layer.size.width, render_layer.size.height);

    println!(
        "{}Layer key: {:?} position: {:?} size: {:?} opacity: {:?}",
        "* ".repeat(level),
        node.layer.key(),
        (
            render_layer.transformed_bounds.x(),
            render_layer.transformed_bounds.y()
        ),
        (
            render_layer.transformed_bounds.width(),
            render_layer.transformed_bounds.height()
        ),
        render_layer.opacity,
    );
}
