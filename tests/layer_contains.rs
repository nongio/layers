use lay_rs::types::Point;
use lay_rs::{engine::LayersEngine, types::Size};

#[test]
pub fn layer_contains() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(100.0, 100.0), None);

    engine.scene_add_layer(layer.clone());

    engine.update(0.016);

    assert!(layer.cointains_point((50.0, 50.0)));
    assert!(!layer.cointains_point((200.0, 50.0)));

    layer.set_position(Point { x: 100.0, y: 100.0 }, None);
    engine.update(0.016);
    // bounds rectangle is in layer coordinates, origin is 0,0
    assert!(!layer.cointains_point((200.0, 200.0)));
    assert!(layer.cointains_point((150.0, 150.0)));
}

#[test]
pub fn scene_node_contains() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());
    layer.set_size(Size::points(100.0, 100.0), None);
    layer.set_anchor_point(Point { x: 0.5, y: 0.5 }, None);
    layer.set_position(Point { x: 100.0, y: 100.0 }, None);

    engine.update(0.016);

    let point = Point { x: 149.0, y: 149.0 };

    let node = engine.scene_layer_at(point);

    assert!(layer.cointains_point(point));
    assert!(node.is_some());

    // FIXME we do not support rotation hit testing properly yet

    // layer.set_rotation(
    //     Point3d {
    //         x: 0.0,
    //         y: 0.0,
    //         z: PI as f32 / 4.0,
    //     },
    //     None,
    // );

    // engine.update(0.016);
    // let node = engine.scene_layer_at(point);
    // assert!(!layer.cointains_point(point));
    // assert!(node.is_none());
}

#[test]
pub fn layer_contains_scale() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(100.0, 100.0), None);

    engine.scene_add_layer(layer.clone());

    layer.set_scale(Point { x: 2.0, y: 2.0 }, None);
    engine.update(0.016);
    // bounds rectangle is in layer coordinates, origin is 0,0
    assert!(layer.cointains_point((199.0, 199.0)));
    assert!(!layer.cointains_point((210.0, 210.0)));
}
