//! A `BackgroundBlur` layer leaves its opaque region out of the backdrop.
//!
//! Wherever the layer's own content is opaque the backdrop is covered as soon
//! as the content is drawn, so leaving it out must not change a single pixel —
//! in a full render, and in a partial repaint that replays a kept blur.

use layers::drawing::{clear_blur_cache, render_node_tree};
use layers::prelude::*;
use layers::skia;
use layers::skia::RoundOut;
use layers::types::{Color, Size};

const W: i32 = 400;
const H: i32 = 300;

fn abs_layer(engine: &Engine, x: f32, y: f32, w: f32, h: f32) -> Layer {
    let layer = engine.new_layer();
    layer.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    });
    layer.set_size(Size::points(w, h), None);
    layer.set_position((x, y), None);
    layer
}

/// The opaque child, in the frosted layer's own coordinates.
const OPAQUE: (f32, f32, f32, f32) = (40.0, 30.0, 200.0, 120.0);

struct Scene {
    engine: std::sync::Arc<Engine>,
    frosted: Layer,
    /// Painted after the frosted layer, overlapping it outside the opaque part.
    above: Layer,
}

fn scene(with_region: bool) -> Scene {
    let engine = Engine::create(W as f32, H as f32);
    let root = abs_layer(&engine, 0.0, 0.0, W as f32, H as f32);
    engine.add_layer(&root).unwrap();

    // A sharp red/blue edge through the frosted shape, so a missing or wrong
    // blur shows in the pixels.
    let left = abs_layer(&engine, 0.0, 0.0, 200.0, H as f32);
    left.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
    engine.append_layer(&left, root.id).unwrap();
    let right = abs_layer(&engine, 200.0, 0.0, 200.0, H as f32);
    right.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
    engine.append_layer(&right, root.id).unwrap();

    let frosted = abs_layer(&engine, 50.0, 50.0, 300.0, 200.0);
    frosted.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 0.2), None);
    frosted.set_blend_mode(BlendMode::BackgroundBlur);
    engine.append_layer(&frosted, root.id).unwrap();

    // Fully opaque, and exactly where the region says.
    let paper = abs_layer(&engine, OPAQUE.0, OPAQUE.1, OPAQUE.2, OPAQUE.3);
    paper.set_background_color(Color::new_rgba(0.9, 0.9, 0.9, 1.0), None);
    engine.append_layer(&paper, frosted.id).unwrap();
    if with_region {
        frosted.set_opaque_region(vec![skia::Rect::from_xywh(
            OPAQUE.0, OPAQUE.1, OPAQUE.2, OPAQUE.3,
        )]);
    }

    let above = abs_layer(&engine, 280.0, 200.0, 50.0, 30.0);
    above.set_background_color(Color::new_rgba(1.0, 1.0, 0.0, 0.5), None);
    engine.append_layer(&above, root.id).unwrap();

    engine.update(0.016);
    engine.update(0.016);
    Scene {
        engine,
        frosted,
        above,
    }
}

fn render(engine: &Engine, surface: &mut skia::Surface, damage: Option<skia::IRect>) {
    let region = damage.map(|rect| {
        let mut region = skia::Region::new();
        region.set_rect(rect);
        region
    });
    let canvas = surface.canvas();
    let restore = canvas.save();
    if let Some(region) = region.as_ref() {
        canvas.clip_region(region, Some(skia::ClipOp::Intersect));
    }
    let scene = engine.scene();
    scene.with_arena(|arena| {
        scene.with_renderable_arena(|renderables| {
            render_node_tree(
                engine.scene_root().unwrap(),
                arena,
                renderables,
                canvas,
                1.0,
                None,
                region.as_ref(),
                None,
            );
        });
    });
    canvas.restore_to_count(restore);
}

fn surface() -> skia::Surface {
    let mut surface = skia::surfaces::raster_n32_premul((W, H)).unwrap();
    surface.canvas().clear(skia::Color::TRANSPARENT);
    surface
}

fn pixels(surface: &mut skia::Surface) -> Vec<u8> {
    let image = surface.image_snapshot();
    let info = skia::ImageInfo::new(
        (W, H),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Premul,
        None,
    );
    let mut px = vec![0u8; (W * H * 4) as usize];
    assert!(image.read_pixels(
        &info,
        &mut px,
        (W * 4) as usize,
        (0, 0),
        skia::image::CachingHint::Allow
    ));
    px
}

fn full_render(s: &Scene) -> Vec<u8> {
    clear_blur_cache();
    let mut fresh = surface();
    render(&s.engine, &mut fresh, None);
    pixels(&mut fresh)
}

fn worst_difference(a: &[u8], b: &[u8]) -> (u8, i32, i32) {
    let mut worst = (0u8, 0, 0);
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        let d = x.abs_diff(*y);
        if d > worst.0 {
            let pixel = (i / 4) as i32;
            worst = (d, pixel % W, pixel / W);
        }
    }
    worst
}

#[test]
fn leaving_the_opaque_region_out_changes_no_pixel() {
    let without = full_render(&scene(false));
    let with = full_render(&scene(true));
    let (diff, x, y) = worst_difference(&with, &without);
    assert!(
        diff <= 2,
        "an opaque region over opaque content changed the render by {diff} at ({x}, {y})"
    );
}

#[test]
fn a_partial_repaint_with_an_opaque_region_matches_a_full_render() {
    clear_blur_cache();
    let s = scene(true);
    let mut screen = surface();
    render(&s.engine, &mut screen, None);
    s.engine.clear_damage();

    // Painted above the blur: the kept blur is replayed under the repaint.
    s.above
        .set_background_color(Color::new_rgba(0.0, 1.0, 1.0, 0.5), None);
    s.engine.update(0.016);
    let damage: skia::IRect = s.engine.damage().round_out();
    render(&s.engine, &mut screen, Some(damage));
    let repainted = pixels(&mut screen);

    let expected = full_render(&s);
    let (diff, x, y) = worst_difference(&repainted, &expected);
    assert!(
        diff <= 2,
        "a repaint of {damage:?} differs from a full render by {diff} at ({x}, {y})"
    );
}

#[test]
fn changing_the_opaque_region_does_not_replay_a_stale_blur() {
    // The kept blur was produced with the region left out; a region that
    // shrinks must blur again rather than replay pixels that hold no backdrop.
    clear_blur_cache();
    let s = scene(true);
    let mut screen = surface();
    render(&s.engine, &mut screen, None);
    s.engine.clear_damage();

    s.frosted.set_opaque_region(Vec::new());
    s.engine.update(0.016);
    render(&s.engine, &mut screen, None);
    let after = pixels(&mut screen);

    let expected = full_render(&s);
    let (diff, x, y) = worst_difference(&after, &expected);
    assert!(
        diff <= 2,
        "after the region changed the render differs by {diff} at ({x}, {y})"
    );
}
