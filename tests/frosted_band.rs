//! A scroll band moving inside a frosted window.
//!
//! The window blurs what is behind it. Its content — a clip, and inside the
//! clip a band of rows taller than the clip — is painted over that blur and is
//! the window's own subtree, so moving the band changes nothing beneath the
//! blur: the backdrop must be kept, and the damage must be only what the clip
//! shows.

use layers::prelude::*;
use layers::types::{Color, Size};
use skia_safe::Contains;

fn absolute(layer: &Layer) {
    layer.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    });
}

/// The frosted window: 800×600 at (500, 500).
const WINDOW: (f32, f32, f32, f32) = (500.0, 500.0, 800.0, 600.0);
/// The clip, in the window's coordinates: 200×300 at (100, 100).
const CLIP: (f32, f32, f32, f32) = (100.0, 100.0, 200.0, 300.0);

fn window_rect() -> skia_safe::Rect {
    skia_safe::Rect::from_xywh(WINDOW.0, WINDOW.1, WINDOW.2, WINDOW.3)
}

/// The clip in scene coordinates.
fn clip_rect() -> skia_safe::Rect {
    skia_safe::Rect::from_xywh(WINDOW.0 + CLIP.0, WINDOW.1 + CLIP.1, CLIP.2, CLIP.3)
}

struct Scene {
    engine: std::sync::Arc<Engine>,
    band: Layer,
}

fn scene() -> Scene {
    let engine = Engine::create(2000.0, 2000.0);

    let root = engine.new_layer();
    absolute(&root);
    root.set_size(Size::points(2000.0, 2000.0), None);
    engine.add_layer(&root).unwrap();

    // Something beneath the window for the blur to take in.
    let wallpaper = engine.new_layer();
    absolute(&wallpaper);
    wallpaper.set_size(Size::points(2000.0, 2000.0), None);
    wallpaper.set_background_color(Color::new_hex("#3366ccff"), None);
    engine.append_layer(&wallpaper.id, Some(root.id)).unwrap();

    let window = engine.new_layer();
    absolute(&window);
    window.set_position((WINDOW.0, WINDOW.1), None);
    window.set_size(Size::points(WINDOW.2, WINDOW.3), None);
    window.set_blend_mode(BlendMode::BackgroundBlur);
    window.set_background_color(Color::new_hex("#ffffff80"), None);
    engine.append_layer(&window.id, Some(root.id)).unwrap();

    let clip = engine.new_layer();
    absolute(&clip);
    clip.set_position((CLIP.0, CLIP.1), None);
    clip.set_size(Size::points(CLIP.2, CLIP.3), None);
    clip.set_clip_children(true, None);
    engine.append_layer(&clip.id, Some(window.id)).unwrap();

    // Taller than the clip, hanging out of it — and out of the window — at
    // both ends, the way a scroll band does.
    let band = engine.new_layer();
    absolute(&band);
    band.set_position((0.0, -400.0), None);
    band.set_size(Size::points(CLIP.2, 1100.0), None);
    band.set_background_color(Color::new_hex("#00000020"), None);
    engine.append_layer(&band.id, Some(clip.id)).unwrap();

    engine.update(0.016);
    engine.clear_damage();
    Scene { engine, band }
}

#[test]
fn a_band_moving_inside_a_frosted_window_damages_only_its_clip() {
    let s = scene();
    s.band.set_position((0.0, -410.0), None);
    s.engine.update(0.016);

    let damage = s.engine.damage();
    assert!(!damage.is_empty(), "the move shows inside the clip");
    assert!(
        clip_rect().contains(damage),
        "damage {damage:?} reaches outside the clip {:?} — the blur was redone \
         for a change painted over it",
        clip_rect()
    );
    assert!(
        !damage.contains(window_rect()),
        "the whole frosted window was repainted for a band move"
    );
}

#[test]
fn a_band_moving_inside_a_frosted_window_keeps_the_subtree_damage_to_its_clip() {
    // What the KMS plane path asks: the window plane's own damage.
    let s = scene();
    s.band.set_position((0.0, -410.0), None);
    s.engine.update(0.016);

    let root = s.engine.scene_root().expect("a root");
    let subtree = s.engine.subtree_damage(root).unwrap_or_default();
    assert!(
        clip_rect().contains(subtree),
        "subtree damage {subtree:?} reaches outside the clip {:?}",
        clip_rect()
    );
}
