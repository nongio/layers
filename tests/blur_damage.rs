//! What a `BackgroundBlur` layer costs the scene when something changes.
//!
//! A blurred backdrop depends only on what is painted *beneath* the blur shape
//! within its reach. A change to the layer's own content, to its descendants,
//! or to anything painted above it or out of its reach leaves the backdrop as
//! it was, and must not repaint the whole shape.

use layers::prelude::*;
use layers::types::{Color, Size};
use skia_safe::Contains;

fn absolute(layer: &Layer) {
    layer.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    });
}

struct Scene {
    engine: std::sync::Arc<Engine>,
    /// Painted before the frosted layer, inside its shape.
    below: Layer,
    /// The frosted layer: 800×600 at (500, 500).
    frosted: Layer,
    /// A child of the frosted layer, 50×50 at its (100, 100).
    content: Layer,
    /// Painted after the frosted layer, overlapping it.
    above: Layer,
    /// Far from the frosted layer, beyond any blur reach.
    elsewhere: Layer,
}

const FROSTED: (f32, f32, f32, f32) = (500.0, 500.0, 800.0, 600.0);

fn frosted_rect() -> skia_safe::Rect {
    skia_safe::Rect::from_xywh(FROSTED.0, FROSTED.1, FROSTED.2, FROSTED.3)
}

fn solid(engine: &Engine, parent: &Layer, rect: (f32, f32, f32, f32), hex: &str) -> Layer {
    let layer = engine.new_layer();
    absolute(&layer);
    layer.set_position((rect.0, rect.1), None);
    layer.set_size(Size::points(rect.2, rect.3), None);
    layer.set_background_color(Color::new_hex(hex), None);
    engine.append_layer(&layer.id, Some(parent.id)).unwrap();
    layer
}

fn scene() -> Scene {
    let engine = Engine::create(2000.0, 2000.0);

    let root = engine.new_layer();
    absolute(&root);
    root.set_size(Size::points(2000.0, 2000.0), None);
    engine.add_layer(&root).unwrap();

    let below = solid(&engine, &root, (600.0, 600.0, 100.0, 100.0), "#ff0000ff");

    let frosted = engine.new_layer();
    absolute(&frosted);
    frosted.set_position((FROSTED.0, FROSTED.1), None);
    frosted.set_size(Size::points(FROSTED.2, FROSTED.3), None);
    frosted.set_blend_mode(BlendMode::BackgroundBlur);
    frosted.set_blur_include_content(true);
    engine.append_layer(&frosted.id, Some(root.id)).unwrap();

    let content = solid(&engine, &frosted, (100.0, 100.0, 50.0, 50.0), "#00ff00ff");
    let above = solid(&engine, &root, (900.0, 900.0, 50.0, 50.0), "#ffffffff");
    let elsewhere = solid(&engine, &root, (1800.0, 100.0, 50.0, 50.0), "#0000ffff");

    engine.update(0.016);
    engine.clear_damage();

    Scene {
        engine,
        below,
        frosted,
        content,
        above,
        elsewhere,
    }
}

#[test]
fn own_content_does_not_repaint_the_whole_shape() {
    let s = scene();
    s.content
        .set_background_color(Color::new_hex("#ffff00ff"), None);
    s.engine.update(0.016);

    let damage = s.engine.damage();
    assert!(
        damage.width() <= 100.0 && damage.height() <= 100.0,
        "repainted {damage:?} for a 50×50 change inside the frosted layer"
    );
    let subtree = s.engine.subtree_damage(s.frosted.id).unwrap();
    assert!(
        subtree.width() <= 100.0 && subtree.height() <= 100.0,
        "subtree damage {subtree:?} for a 50×50 change inside the frosted layer"
    );
}

#[test]
fn content_painted_above_does_not_repaint_the_whole_shape() {
    let s = scene();
    s.above
        .set_background_color(Color::new_hex("#000000ff"), None);
    s.engine.update(0.016);

    let damage = s.engine.damage();
    assert!(
        damage.width() <= 100.0 && damage.height() <= 100.0,
        "repainted {damage:?} for a 50×50 change painted above the frosted layer"
    );
}

#[test]
fn damage_out_of_reach_does_not_repaint_the_shape() {
    let s = scene();
    s.elsewhere
        .set_background_color(Color::new_hex("#ff00ffff"), None);
    s.engine.update(0.016);

    let damage = s.engine.damage();
    assert!(
        !damage.intersects(frosted_rect()),
        "a change far from the frosted layer repainted {damage:?}"
    );
}

#[test]
fn damage_beneath_repaints_the_whole_shape() {
    let s = scene();
    s.below
        .set_background_color(Color::new_hex("#00ffffff"), None);
    s.engine.update(0.016);

    let damage = s.engine.damage();
    assert!(
        damage.contains(frosted_rect()),
        "the blur over a changed backdrop must be redone across the shape, got {damage:?}"
    );
}

#[test]
fn moving_the_frosted_layer_repaints_its_old_and_new_shape() {
    let s = scene();
    s.frosted.set_position((FROSTED.0 + 50.0, FROSTED.1), None);
    s.engine.update(0.016);

    let damage = s.engine.damage();
    let mut both = frosted_rect();
    both.join(frosted_rect().with_offset((50.0, 0.0)));
    assert!(
        damage.contains(both),
        "a moved blur must be repainted where it was and where it is, got {damage:?}"
    );
}
