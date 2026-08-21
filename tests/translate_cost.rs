//! Regression: a pure translation of a cached layer must not re-run the
//! content draw closure nor bump the node's frame number (which drives the
//! image cache and the per-subtree plane buffer cache).
use layers::prelude::*;
use layers::taffy;
use layers::types::{Point as LPoint, Size as LSize};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn setup() -> (Arc<Engine>, Layer, Arc<AtomicUsize>) {
    let engine = Engine::create(1000.0, 1000.0);
    let root = engine.new_layer();
    root.set_size(LSize::points(1000.0, 1000.0), None);
    engine.scene_set_root(root.clone());

    let layer = engine.new_layer();
    layer.set_size(LSize::points(200.0, 200.0), None);
    layer.set_background_color(Color::new_hex("#ff0000ff"), None);
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    layer.set_draw_content(move |canvas: &skia_safe::Canvas, w: f32, h: f32| {
        c.fetch_add(1, Ordering::SeqCst);
        let mut p = skia_safe::Paint::default();
        p.set_color(skia_safe::Color::BLUE);
        canvas.draw_rect(skia_safe::Rect::from_wh(w, h), &p);
        skia_safe::Rect::from_wh(w, h)
    });
    let _ = root.add_sublayer(&layer);
    (engine, layer, calls)
}

fn frame_of(engine: &Engine, layer: &Layer) -> usize {
    engine
        .scene()
        .with_arena(|a| a.get(layer.id().0).unwrap().get().frame_number())
}

#[test]
fn translation_does_not_rerecord_content() {
    let (engine, layer, calls) = setup();
    engine.update(0.016);
    let after_first = calls.load(Ordering::SeqCst);
    assert!(after_first >= 1, "first paint must run the closure");
    let frame0 = frame_of(&engine, &layer);

    for i in 1..=30 {
        layer.set_position(LPoint::new(i as f32 * 3.0, 0.0), None);
        engine.update(0.016);
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        after_first,
        "pure translation re-ran the content closure"
    );
    assert_eq!(
        frame_of(&engine, &layer),
        frame0,
        "pure translation bumped frame_number (invalidates image/plane caches)"
    );
}

#[test]
fn translation_still_damages_old_and_new_bounds() {
    let (engine, layer, _calls) = setup();
    engine.update(0.016);
    engine.clear_damage();
    layer.set_position(LPoint::new(100.0, 0.0), None);
    engine.update(0.016);
    let d = engine.damage();
    assert_eq!(
        d,
        skia_safe::Rect::from_xywh(0.0, 0.0, 300.0, 200.0),
        "got {:?}",
        d
    );
}

#[test]
fn resize_does_rerecord_content() {
    let (engine, layer, calls) = setup();
    engine.update(0.016);
    let base = calls.load(Ordering::SeqCst);
    layer.set_size(LSize::points(300.0, 200.0), None);
    engine.update(0.016);
    assert!(calls.load(Ordering::SeqCst) > base, "resize must re-record");
}

#[test]
fn parent_translation_does_not_rerecord_child() {
    let engine = Engine::create(1000.0, 1000.0);
    let root = engine.new_layer();
    root.set_size(LSize::points(1000.0, 1000.0), None);
    engine.scene_set_root(root.clone());
    let parent = engine.new_layer();
    parent.set_size(LSize::points(400.0, 400.0), None);
    let child = engine.new_layer();
    child.set_size(LSize::points(100.0, 100.0), None);
    child.set_background_color(Color::new_hex("#00ff00ff"), None);
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    child.set_draw_content(move |_c: &skia_safe::Canvas, w: f32, h: f32| {
        c.fetch_add(1, Ordering::SeqCst);
        skia_safe::Rect::from_wh(w, h)
    });
    let _ = root.add_sublayer(&parent);
    let _ = parent.add_sublayer(&child);
    engine.update(0.016);
    let base = calls.load(Ordering::SeqCst);
    for i in 1..=10 {
        parent.set_position(LPoint::new(i as f32 * 5.0, 0.0), None);
        engine.update(0.016);
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        base,
        "parent move re-recorded the child"
    );
}

#[test]
fn content_damage_still_repaints() {
    let (engine, layer, calls) = setup();
    engine.update(0.016);
    let base = calls.load(Ordering::SeqCst);
    layer.add_damage(skia_safe::Rect::from_wh(50.0, 50.0));
    engine.update(0.016);
    assert!(
        calls.load(Ordering::SeqCst) > base,
        "explicit repaint must re-record"
    );
}

// ---------------------------------------------------------------------------
// Rendering correctness for the caches a pure translation now REUSES instead
// of rebuilding: the picture cache, the image cache (offscreen surface) and
// the per-subtree plane buffer.
// ---------------------------------------------------------------------------

const W: i32 = 200;
const H: i32 = 200;

fn render(engine: &Arc<Engine>, root: &Layer) -> Vec<u8> {
    let mut surface = skia_safe::surfaces::raster_n32_premul((W, H)).unwrap();
    {
        let canvas = surface.canvas();
        canvas.clear(skia_safe::Color::WHITE);
        layers::drawing::draw_scene(canvas, engine.scene(), root.id());
    }
    let img = surface.image_snapshot();
    let mut px = vec![0u8; (W * H * 4) as usize];
    let info = skia_safe::ImageInfo::new(
        (W, H),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );
    assert!(img.read_pixels(
        &info,
        &mut px,
        (W * 4) as usize,
        (0, 0),
        skia_safe::image::CachingHint::Allow
    ));
    px
}

fn pixel(px: &[u8], x: i32, y: i32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [px[i], px[i + 1], px[i + 2], px[i + 3]]
}

fn scene_with_box(image_cached: bool) -> (Arc<Engine>, Layer, Layer) {
    let engine = Engine::create(W as f32, H as f32);
    let root = engine.new_layer();
    root.set_layout_style(taffy::Style {
        position: taffy::style::Position::Absolute,
        ..Default::default()
    });
    root.set_size(LSize::points(W as f32, H as f32), None);
    engine.scene_set_root(root.clone());

    let boxl = engine.new_layer();
    boxl.set_layout_style(taffy::Style {
        position: taffy::style::Position::Absolute,
        ..Default::default()
    });
    boxl.set_size(LSize::points(40.0, 40.0), None);
    boxl.set_position(LPoint::new(10.0, 10.0), None);
    boxl.set_background_color(Color::new_hex("#ff0000ff"), None);
    boxl.set_image_cached(image_cached);
    let _ = root.add_sublayer(&boxl);
    (engine, root, boxl)
}

/// The cached content must land at the NEW position, and the OLD position must
/// be clean. Run for both cache kinds.
fn translate_renders_at_new_position(image_cached: bool) {
    let (engine, root, boxl) = scene_with_box(image_cached);
    engine.update(0.016);
    let _ = render(&engine, &root);

    boxl.set_position(LPoint::new(120.0, 120.0), None);
    engine.update(0.016);
    let px = render(&engine, &root);

    let at_new = pixel(&px, 140, 140);
    let at_old = pixel(&px, 30, 30);
    assert_eq!(
        &at_new[0..3],
        &[255, 0, 0],
        "image_cached={image_cached}: content missing at the new position ({at_new:?})"
    );
    assert_eq!(
        &at_old[0..3],
        &[255, 255, 255],
        "image_cached={image_cached}: content still painted at the old position ({at_old:?})"
    );
}

#[test]
fn picture_cached_translate_renders_correctly() {
    translate_renders_at_new_position(false);
}

#[test]
fn image_cached_translate_renders_correctly() {
    translate_renders_at_new_position(true);
}

/// KMS plane path: a subtree that only moved must report the NEW origin, not a
/// cached stale one. `frame_number` no longer changes on a pure move, so the
/// buffer cache has to key on the origin.
#[test]
fn subtree_buffer_follows_a_pure_move() {
    let (engine, _root, boxl) = scene_with_box(false);
    engine.update(0.016);
    let b0 = layers::prelude::render_subtree_to_buffer(engine.scene(), boxl.id(), None, None)
        .expect("first buffer");
    assert_eq!((b0.origin.x, b0.origin.y), (10.0, 10.0));

    boxl.set_position(LPoint::new(120.0, 120.0), None);
    engine.update(0.016);
    let b1 = layers::prelude::render_subtree_to_buffer(engine.scene(), boxl.id(), None, None)
        .expect("second buffer");
    assert_eq!(
        (b1.origin.x, b1.origin.y),
        (120.0, 120.0),
        "plane buffer kept the stale origin after a pure move"
    );
    assert!(
        !b1.from_cache,
        "a moved subtree must not report a cache hit"
    );
}
