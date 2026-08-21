//! Regression: a change made while a layer is invisible must not be lost.
//!
//! `do_repaint` early-returns without recording a picture when the node is
//! hidden or has `premultiplied_opacity == 0.0`, but `update_node_single`
//! clears NEEDS_PAINT afterwards regardless. A repaint requested while the
//! node was invisible was therefore dropped, leaving the pre-change picture as
//! the node's only recording — replayed as-is once the layer became visible
//! again, under an otherwise up-to-date transform. Symptom downstream: a
//! subtree rendered in isolation via `render_node_tree` (the KMS plane path)
//! painting stale content or nothing at all, while every live field on the
//! nodes matched a good frame.
//!
//! The fix drops `draw_cache` on the invisible path, so the node re-records on
//! the next update. These tests cover the three ways a layer can be invisible
//! when the change lands: a zero-opacity ancestor, an explicitly hidden layer,
//! and a layer that has never been painted at all.

use layers::{
    drawing::render_node_tree,
    prelude::*,
    skia, taffy,
    types::{Color, Size},
};

const W: i32 = 100;
const H: i32 = 100;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

/// Render `root`'s subtree into its own transparent buffer, exactly the way
/// Otto renders a KMS plane, and read the centre pixel.
fn render_subtree(engine: &std::sync::Arc<Engine>, root: NodeRef) -> [u8; 4] {
    let mut surface = skia::surfaces::raster_n32_premul((W, H)).unwrap();
    {
        let canvas = surface.canvas();
        canvas.clear(skia::Color::TRANSPARENT);
        let scene_ref = engine.scene();
        scene_ref.with_arena(|arena| {
            scene_ref.with_renderable_arena(|renderables| {
                render_node_tree(root, arena, renderables, canvas, 1.0, None, None, None);
            });
        });
    }
    let image = surface.image_snapshot();
    let info = skia::ImageInfo::new(
        (1, 1),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Unpremul,
        None,
    );
    let mut px = [0u8; 4];
    assert!(image.read_pixels(
        &info,
        &mut px,
        4,
        skia::IPoint::new(W / 2, H / 2),
        skia::image::CachingHint::Disallow,
    ));
    px
}

/// mid (container) -> child (solid colour). `mid` is the subtree root that a
/// compositor would hand to `render_node_tree`.
fn scene() -> (std::sync::Arc<Engine>, Layer, Layer) {
    let engine = Engine::create(W as f32, H as f32);

    let mid = engine.new_layer();
    engine.add_layer(&mid).unwrap();
    mid.set_layout_style(absolute());
    mid.set_position((0.0, 0.0), None);
    mid.set_size(Size::points(W as f32, H as f32), None);
    mid.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let child = engine.new_layer();
    engine.append_layer(&child, Some(mid.id)).unwrap();
    child.set_layout_style(absolute());
    child.set_position((0.0, 0.0), None);
    child.set_size(Size::points(W as f32, H as f32), None);
    child.set_background_color(Color::new_rgba255(255, 0, 0, 255), None);

    engine.update(0.016);
    (engine, mid, child)
}

/// A content change made while an ancestor's opacity is 0 must not be lost
/// when the ancestor becomes visible again.
#[test]
fn content_change_while_ancestor_opacity_zero_is_not_lost() {
    let (engine, mid, child) = scene();

    let before = render_subtree(&engine, mid.id);
    assert_eq!(before[0], 255, "baseline should be red, got {before:?}");

    // Ancestor fades fully out: child's premultiplied_opacity becomes 0.
    mid.set_opacity(0.0, None);
    engine.update(0.016);

    // Content changes while the child is effectively invisible. This sets
    // NEEDS_PAINT, but do_repaint bails on premultiplied_opacity == 0.0 and
    // update_node_single clears the flag anyway.
    child.set_background_color(Color::new_rgba255(0, 255, 0, 255), None);
    engine.update(0.016);

    // Ancestor fades back in.
    mid.set_opacity(1.0, None);
    engine.update(0.016);

    let after = render_subtree(&engine, mid.id);
    assert_eq!(
        after[1], 255,
        "child should be green after the ancestor faded back in, got {after:?}"
    );
    assert_eq!(
        after[0], 0,
        "child should no longer be red, got {after:?} (stale draw_cache replayed)"
    );
}

/// Same shape, but the ancestor is hidden via set_hidden rather than faded.
#[test]
fn content_change_while_hidden_is_not_lost() {
    let (engine, mid, child) = scene();

    let before = render_subtree(&engine, mid.id);
    assert_eq!(before[0], 255, "baseline should be red, got {before:?}");

    child.set_hidden(true);
    engine.update(0.016);

    child.set_background_color(Color::new_rgba255(0, 255, 0, 255), None);
    engine.update(0.016);

    child.set_hidden(false);
    engine.update(0.016);

    let after = render_subtree(&engine, mid.id);
    assert_eq!(
        after[1], 255,
        "child should be green after being un-hidden, got {after:?}"
    );
}

/// A layer first painted only while it was invisible: its draw_cache may never
/// have been created at all, so replaying it draws nothing (black plane).
#[test]
fn first_paint_while_invisible_still_draws_once_visible() {
    let engine = Engine::create(W as f32, H as f32);

    let mid = engine.new_layer();
    engine.add_layer(&mid).unwrap();
    mid.set_layout_style(absolute());
    mid.set_position((0.0, 0.0), None);
    mid.set_size(Size::points(W as f32, H as f32), None);
    mid.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    // Fade the container out *before* the child ever exists, so the child's
    // very first update happens with premultiplied_opacity == 0.
    mid.set_opacity(0.0, None);
    engine.update(0.016);

    let child = engine.new_layer();
    engine.append_layer(&child, Some(mid.id)).unwrap();
    child.set_layout_style(absolute());
    child.set_position((0.0, 0.0), None);
    child.set_size(Size::points(W as f32, H as f32), None);
    child.set_background_color(Color::new_rgba255(0, 0, 255, 255), None);
    engine.update(0.016);

    mid.set_opacity(1.0, None);
    engine.update(0.016);

    let after = render_subtree(&engine, mid.id);
    assert_eq!(
        after[2], 255,
        "child should be blue on its first visible frame, got {after:?} \
         (draw_cache never created while invisible)"
    );
}
