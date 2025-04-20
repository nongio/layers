use lay_rs::{prelude::*, types::Size};

#[test]
pub fn engine_update() {
    let engine = Engine::create(1000.0, 1000.0);

    let layer = engine.new_layer();

    layer.set_size(Size::points(100.0, 100.0), None);

    let child_layer = engine.new_layer();
    child_layer.set_size(Size::percent(0.5, 0.5), None);

    layer.add_sublayer(&child_layer);

    engine.add_layer(&layer);

    engine.update(0.016);
    engine.update(0.016);

    layer.set_size(Size::points(200.0, 200.0), None);

    engine.update(0.016);
    engine.update(0.016);

    assert!(true);
}
