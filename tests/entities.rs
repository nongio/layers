use lay_rs::{prelude::*, types::Point};

#[test]
pub fn change_layer_position() {
    let engine = Engine::create(1000.0, 1000.0);

    let layer = engine.new_layer();

    let _id = engine.add_layer(&layer);

    assert_eq!(layer.position().x, 0.0);

    layer.set_position(Point { x: 200.0, y: 100.0 }, None);

    engine.update(0.01);

    assert_eq!(layer.position().x, 200.0);
}
