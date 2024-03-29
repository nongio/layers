#![allow(warnings)]

use indextree::Arena;
use skia_safe::Canvas;

use crate::engine::{
    node::{DrawCacheManagement, SceneNode},
    scene::Scene,
    storage::TreeStorageId,
    NodeRef,
};

use super::layer::draw_layer;

pub trait DrawScene {
    fn draw_scene(&self, scene: &Scene, root_id: NodeRef, damage: Option<skia_safe::Rect>);
}

pub fn draw_scene(canvas: &mut Canvas, scene: &Scene, root_id: NodeRef) {
    let arena = scene.nodes.data();
    let arena = arena.read().unwrap();
    if let Some(_root) = scene.get_node(root_id) {
        render_node_tree(root_id, &arena, canvas, 1.0);
    }
}

pub fn render_node_tree(
    node_ref: NodeRef,
    arena: &Arena<SceneNode>,
    canvas: &mut skia_safe::Canvas,
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

    canvas.restore_to_count(restore_transform);
}

pub(crate) fn render_node(
    node_id: NodeRef,
    arena: &Arena<SceneNode>,
    canvas: &mut skia_safe::Canvas,
    context_opacity: f32,
) -> usize {
    let node_id: TreeStorageId = node_id.into();
    let node = arena.get(node_id).unwrap().get();
    let render_layer = node.render_layer.read().unwrap();
    let node_opacity = render_layer.opacity;
    let opacity = context_opacity * node_opacity;

    let blend_mode = render_layer.blend_mode;
    let restore_transform = canvas.save();
    canvas.concat(&render_layer.transform);

    let draw_cache = node.draw_cache.read().unwrap();

    let before_backdrop = canvas.save();

    let bounds_to_origin =
        skia_safe::Rect::from_xywh(0.0, 0.0, render_layer.size.x, render_layer.size.y);

    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(opacity);

    if blend_mode == crate::prelude::BlendMode::BackgroundBlur {
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
        let crop_rect = Some(skia_safe::image_filters::CropRect::from(bounds_to_origin));

        let blur = skia_safe::image_filters::blur(
            (50.0, 50.0),
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
