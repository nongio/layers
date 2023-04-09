use std::f64::consts::PI;

use layers::engine::node::ContainsPoint;
use layers::engine::LayersEngine;
use layers::types::{Point, Point3d};

#[test]
pub fn layer_contains() {
    let engine = LayersEngine::new();
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    layer.set_size(Point { x: 100.0, y: 100.0 }, None);

    assert!(layer.bounds().contains(Point { x: 50.0, y: 50.0 }));
    assert!(!layer.bounds().contains(Point { x: 200.0, y: 50.0 }));

    layer.set_position(Point { x: 100.0, y: 100.0 }, None);
    // bounds rectangle is in layer coordinates, origin is 0,0
    assert!(!layer.bounds().contains(Point { x: 200.0, y: 200.0 }));
    assert!(layer.bounds().contains(Point { x: 50.0, y: 50.0 }));
}

#[test]
pub fn scene_node_contains() {
    let engine = LayersEngine::new();
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    const RECT_WIDTH: f32 = 100.0;
    const RECT_HEIGHT: f32 = 100.0;

    layer.set_anchor_point(Point { x: 0.5, y: 0.5 }, None);
    layer.set_size(
        Point {
            x: RECT_WIDTH as f32,
            y: RECT_HEIGHT as f32,
        },
        None,
    );

    layer.set_position(Point { x: 160.0, y: 160.0 }, None);

    engine.update(0.0);

    let point = Point { x: 200.0, y: 200.0 };

    let node = engine.scene_layer_at(point);

    assert!(node.is_some());

    layer.set_rotation(
        Point3d {
            x: 0.0,
            y: 0.0,
            z: PI as f32 / 4.0,
        },
        None,
    );
    engine.update(0.0);
    let node = engine.scene_layer_at(point);
    assert!(node.is_none());
}
