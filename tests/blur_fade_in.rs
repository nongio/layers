//! A `BackgroundBlur` layer fading in under a spring must end up frosted.
//!
//! The dock's context menu fades its wrapper in with a critically damped
//! spring; the spring settled a hair under 1.0 and stayed there, which kept
//! the layer inside the fade group forever, and without an external backdrop
//! the group held nothing to blur. Sharp stripes showed through the panel.

use layers::{
    prelude::*,
    skia, taffy,
    types::{BlendMode, Color, Size},
};

const W: i32 = 128;
const H: i32 = 128;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

fn info() -> skia::ImageInfo {
    skia::ImageInfo::new(
        (W, H),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Premul,
        None,
    )
}

/// Peak-to-peak red across one stripe period in the middle of the panel.
fn ripple(engine: &std::sync::Arc<Engine>, root: NodeRef, surface: &mut skia::Surface) -> u8 {
    let canvas = surface.canvas();
    canvas.clear(skia::Color::TRANSPARENT);
    let scene = engine.scene();
    scene.with_arena(|arena| {
        scene.with_renderable_arena(|renderables| {
            render_node_tree(root, arena, renderables, canvas, 1.0, None, None, None);
        });
    });
    let mut px = vec![0u8; (W * H * 4) as usize];
    assert!(surface.read_pixels(&info(), &mut px, (W * 4) as usize, (0, 0)));
    let vals: Vec<u8> = (56..72)
        .map(|x| px[((H / 2 * W + x) * 4) as usize])
        .collect();
    vals.iter().max().unwrap() - vals.iter().min().unwrap()
}

#[test]
fn blur_layer_fading_in_ends_frosted() {
    let engine = Engine::create(W as f32, H as f32);
    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_size(Size::points(W as f32, H as f32), None);
    root.set_background_color(Color::new_rgba(0.0, 0.0, 0.0, 1.0), None);
    for i in 0..(W / 8) {
        let stripe = engine.new_layer();
        engine.append_layer(&stripe, Some(root.id)).unwrap();
        stripe.set_layout_style(absolute());
        stripe.set_position(((i * 8) as f32, 0.0), None);
        stripe.set_size(Size::points(4.0, H as f32), None);
        stripe.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 1.0), None);
    }
    let wrap = engine.new_layer();
    engine.append_layer(&wrap, Some(root.id)).unwrap();
    wrap.set_layout_style(absolute());
    wrap.set_size(Size::points(W as f32, H as f32), None);
    wrap.set_background_color(Color::new_rgba(0.0, 0.0, 0.0, 0.0), None);
    let panel = engine.new_layer();
    engine.append_layer(&panel, Some(wrap.id)).unwrap();
    panel.set_layout_style(absolute());
    panel.set_position((16.0, 16.0), None);
    panel.set_size(Size::points(96.0, 96.0), None);
    panel.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 0.3), None);
    panel.set_blend_mode(BlendMode::BackgroundBlur);

    wrap.set_opacity(0.0, None);
    wrap.set_opacity(
        1.0,
        Some(Transition {
            delay: 0.0,
            timing: TimingFunction::Spring(Spring::with_duration_and_bounce(0.05, 0.0)),
        }),
    );

    let mut surface = skia::surfaces::raster(&info(), None, None).unwrap();
    engine.update(0.016);
    // Mid-fade the frost is already there, crossfaded with the sharp stripes.
    let mid = ripple(&engine, root.id, &mut surface);
    assert!(mid < 200, "mid-fade: no frost at all (ripple {mid})");
    for _ in 0..30 {
        engine.update(0.016);
    }
    assert_eq!(wrap.opacity(), 1.0, "a settled spring lands on its target");
    let settled = ripple(&engine, root.id, &mut surface);
    assert!(
        settled < 40,
        "settled: stripes still sharp (ripple {settled})"
    );
}
