//! Damage a clipping ancestor cuts away.
//!
//! A scrolled band is content taller than the clip it moves inside. Moving it
//! changes only what the clip shows, so that is all the damage may cover —
//! not the band's whole height, most of which is never drawn.

use layers::prelude::*;
use layers::types::{Color, Size};
use skia_safe::Contains;

fn absolute(layer: &Layer) {
    layer.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    });
}

const CLIP: (f32, f32, f32, f32) = (100.0, 100.0, 200.0, 200.0);

fn clip_rect() -> skia_safe::Rect {
    skia_safe::Rect::from_xywh(CLIP.0, CLIP.1, CLIP.2, CLIP.3)
}

/// A root, a clip at [`CLIP`] and an 800-point band hanging out of it at both
/// ends. `clips` decides whether the clip clips its children.
fn scene(clips: bool) -> (std::sync::Arc<Engine>, Layer) {
    let engine = Engine::create(1000.0, 1000.0);

    let root = engine.new_layer();
    absolute(&root);
    root.set_size(Size::points(1000.0, 1000.0), None);
    engine.add_layer(&root).unwrap();

    let clip = engine.new_layer();
    absolute(&clip);
    clip.set_position((CLIP.0, CLIP.1), None);
    clip.set_size(Size::points(CLIP.2, CLIP.3), None);
    clip.set_clip_children(clips, None);
    engine.append_layer(&clip.id, Some(root.id)).unwrap();

    let band = engine.new_layer();
    absolute(&band);
    band.set_position((0.0, -300.0), None);
    band.set_size(Size::points(CLIP.2, 800.0), None);
    band.set_background_color(Color::new_hex("#ff0000ff"), None);
    engine.append_layer(&band.id, Some(clip.id)).unwrap();

    engine.update(0.016);
    engine.clear_damage();
    (engine, band)
}

#[test]
fn moving_content_inside_a_clip_damages_only_what_the_clip_shows() {
    let (engine, band) = scene(true);

    band.set_position((0.0, -310.0), None);
    engine.update(0.016);

    let damage = engine.damage();
    assert!(!damage.is_empty(), "the move shows inside the clip");
    assert!(
        clip_rect().contains(damage),
        "damage {damage:?} spills out of the clip {:?}",
        clip_rect()
    );
}

#[test]
fn without_a_clip_the_whole_move_is_damaged() {
    // The control: the same move with nothing clipping it reaches past the
    // clip's box, which is what the clipped case must not do.
    let (engine, band) = scene(false);

    band.set_position((0.0, -310.0), None);
    engine.update(0.016);

    let damage = engine.damage();
    assert!(
        !clip_rect().contains(damage),
        "an unclipped band damages its own extent, got {damage:?}"
    );
}

#[test]
fn a_move_entirely_outside_the_clip_damages_nothing() {
    let (engine, band) = scene(true);
    // Shrink the band to a sliver that sits wholly below the clip, then move
    // it further down: nothing of it was ever visible.
    band.set_size(Size::points(CLIP.2, 10.0), None);
    band.set_position((0.0, 400.0), None);
    engine.update(0.016);
    engine.clear_damage();

    band.set_position((0.0, 420.0), None);
    engine.update(0.016);

    assert!(
        engine.damage().is_empty(),
        "a move the clip hides entirely is not damage, got {:?}",
        engine.damage()
    );
}
