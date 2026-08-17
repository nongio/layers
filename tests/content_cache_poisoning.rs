//! `content_cache` poisoning: a layer whose draw closure produces nothing on
//! one repaint keeps replaying that empty recording on every later frame.
//!
//! Background (Otto's black background plane): `draw_layer`
//! (drawing/layer.rs:133) prefers `renderable.content_cache` and only invokes
//! the live `content_draw_func` when that cache is `None`. `do_repaint`
//! (engine/node/mod.rs:359) assigns
//!
//!     new_renderable.content_cache = recorder.finish_recording_as_picture(None);
//!
//! unconditionally — so a repaint that happens while the content source has
//! nothing to give (a Wayland client that hasn't attached a buffer yet)
//! overwrites a good recording with an empty one. Because the empty recording
//! is `Some`, `draw_layer` never falls back to the closure again, and the
//! layer stays blank until something re-triggers `do_repaint`.
//!
//! Note this is independent of `set_picture_cached(false)`: that only disables
//! the whole-layer `draw_cache` (drawing/scene.rs:923), not `content_cache`.

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

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

fn render_subtree(engine: &Arc<Engine>, root: NodeRef) -> [u8; 4] {
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

/// A layer mirroring a client surface: the closure paints green only while the
/// "client" has a buffer attached, exactly like Otto's `layer_shell_bg_mirror`
/// drawing a wlr-layer-shell surface that may not have committed yet.
#[test]
fn empty_content_recording_does_not_poison_the_layer() {
    let engine = Engine::create(W as f32, H as f32);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(W as f32, H as f32), None);
    root.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let mirror = engine.new_layer();
    engine.append_layer(&mirror, Some(root.id)).unwrap();
    mirror.set_layout_style(absolute());
    mirror.set_position((0.0, 0.0), None);
    mirror.set_size(Size::points(W as f32, H as f32), None);
    mirror.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);
    // Same configuration Otto uses for the wallpaper mirror.
    mirror.set_picture_cached(false);

    let buffer_ready = Arc::new(AtomicBool::new(true));
    let closure_calls = Arc::new(AtomicUsize::new(0));
    {
        let buffer_ready = buffer_ready.clone();
        let closure_calls = closure_calls.clone();
        mirror.set_draw_content(move |canvas: &skia::Canvas, w: f32, h: f32| {
            closure_calls.fetch_add(1, Ordering::SeqCst);
            if buffer_ready.load(Ordering::SeqCst) {
                let mut paint = skia::Paint::default();
                paint.set_color4f(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                canvas.draw_rect(skia::Rect::from_wh(w, h), &paint);
            }
            skia::Rect::from_wh(w, h)
        });
    }

    engine.update(0.016);
    let good = render_subtree(&engine, root.id);
    assert_eq!(good[1], 255, "baseline should be green, got {good:?}");

    // The client momentarily has nothing to give, and something triggers a
    // repaint in that window (in Otto: the leader surface reporting damage,
    // which sets NEEDS_PAINT on every follower).
    buffer_ready.store(false, Ordering::SeqCst);
    mirror.set_damage(skia::Rect::from_wh(W as f32, H as f32));
    engine.update(0.016);

    let blank = render_subtree(&engine, root.id);
    assert_eq!(blank[3], 0, "sanity: expected a blank frame, got {blank:?}");

    // The client is back. Nothing marks the layer dirty — from lay-rs's point
    // of view nothing about the layer changed. The closure is the only thing
    // that knows there is content again.
    buffer_ready.store(true, Ordering::SeqCst);
    let calls_before = closure_calls.load(Ordering::SeqCst);
    engine.update(0.016);
    let after = render_subtree(&engine, root.id);
    let calls_after = closure_calls.load(Ordering::SeqCst);

    println!("closure calls before={calls_before} after={calls_after}, pixel={after:?}");
    assert_eq!(
        after[1], 255,
        "layer should paint again once the source has content, got {after:?} \
         (empty content_cache replayed instead of re-invoking the closure)"
    );
}
