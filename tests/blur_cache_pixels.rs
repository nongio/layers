//! A `BackgroundBlur` layer keeps its blurred backdrop between frames, so a
//! partial repaint — its own content changed, or something painted above it —
//! draws the kept blur instead of blurring again. Whatever path it takes, a
//! partial repaint must leave exactly the pixels a full render of the new state
//! would.

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

struct Scene {
    engine: std::sync::Arc<Engine>,
    /// Beneath the frosted layer: the right half of the backdrop.
    right: Layer,
    /// A child of the frosted layer.
    content: Layer,
    /// Painted after the frosted layer, overlapping it.
    above: Layer,
}

fn scene() -> Scene {
    let engine = Engine::create(W as f32, H as f32);
    let root = abs_layer(&engine, 0.0, 0.0, W as f32, H as f32);
    engine.add_layer(&root).unwrap();

    // A sharp red/blue edge straight through the frosted shape: the blur has
    // something to smear, so a wrong or partial blur shows.
    let left = abs_layer(&engine, 0.0, 0.0, 200.0, H as f32);
    left.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
    engine.append_layer(&left, root.id).unwrap();
    let right = abs_layer(&engine, 200.0, 0.0, 200.0, H as f32);
    right.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
    engine.append_layer(&right, root.id).unwrap();

    let frosted = abs_layer(&engine, 50.0, 50.0, 300.0, 200.0);
    frosted.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 0.2), None);
    frosted.set_blend_mode(BlendMode::BackgroundBlur);
    frosted.set_blur_include_content(true);
    engine.append_layer(&frosted, root.id).unwrap();

    // Both translucent: the blur beneath shows through what they repaint, so a
    // repaint that got the blur wrong is visible in the pixels.
    let content = abs_layer(&engine, 130.0, 80.0, 40.0, 40.0);
    content.set_background_color(Color::new_rgba(0.0, 1.0, 0.0, 0.5), None);
    engine.append_layer(&content, frosted.id).unwrap();

    let above = abs_layer(&engine, 250.0, 180.0, 60.0, 30.0);
    above.set_background_color(Color::new_rgba(1.0, 1.0, 0.0, 0.5), None);
    engine.append_layer(&above, root.id).unwrap();

    engine.update(0.016);
    engine.update(0.016);

    Scene {
        engine,
        right,
        content,
        above,
    }
}

/// Render the scene into `surface`, repainting only `damage` when given —
/// clipped and culled to it, the way a compositor's damage tracker does.
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

fn surface() -> skia::Surface {
    let mut surface = skia::surfaces::raster_n32_premul((W, H)).unwrap();
    surface.canvas().clear(skia::Color::TRANSPARENT);
    surface
}

/// Full render of the scene's current state into a fresh surface, with no
/// kept blur to draw from.
fn reference(engine: &Engine) -> Vec<u8> {
    clear_blur_cache();
    let mut fresh = surface();
    render(engine, &mut fresh, None);
    pixels(&mut fresh)
}

/// The largest per-channel difference between two renders, and where.
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

/// Render once in full, apply `change`, repaint only the resulting damage,
/// and compare with a full render of the changed scene.
fn partial_repaint_matches_full_render(change: impl FnOnce(&Scene)) -> skia::IRect {
    clear_blur_cache();
    let s = scene();
    let mut screen = surface();
    render(&s.engine, &mut screen, None);
    s.engine.clear_damage();

    change(&s);
    s.engine.update(0.016);
    let damage: skia::IRect = s.engine.damage().round_out();
    render(&s.engine, &mut screen, Some(damage));
    let repainted = pixels(&mut screen);

    let expected = reference(&s.engine);
    let (diff, x, y) = worst_difference(&repainted, &expected);
    println!("repaint of {damage:?}: worst difference {diff} at ({x}, {y})");
    assert!(
        diff <= 2,
        "a repaint of {damage:?} differs from a full render by {diff} at ({x}, {y})"
    );
    damage
}

#[test]
fn own_content_repaint_draws_the_kept_blur() {
    let damage = partial_repaint_matches_full_render(|s| {
        s.content
            .set_background_color(Color::new_rgba(1.0, 0.0, 1.0, 0.5), None);
    });
    assert!(
        damage.width() <= 80 && damage.height() <= 80,
        "expected a repaint of just the content, got {damage:?}"
    );
}

#[test]
fn repaint_above_draws_the_kept_blur() {
    let damage = partial_repaint_matches_full_render(|s| {
        s.above
            .set_background_color(Color::new_rgba(0.0, 1.0, 1.0, 0.5), None);
    });
    assert!(
        damage.width() <= 100 && damage.height() <= 100,
        "expected a repaint of just the layer above, got {damage:?}"
    );
}

#[test]
fn a_changed_backdrop_blurs_again() {
    partial_repaint_matches_full_render(|s| {
        s.right
            .set_background_color(Color::new_rgba(0.0, 1.0, 0.0, 1.0), None);
    });
}
