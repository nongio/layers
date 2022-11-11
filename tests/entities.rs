use hello::engine::node::RenderNode;
use hello::engine::Engine;
use hello::layers::layer::ModelLayer;
use hello::types::Point;
use std::sync::Arc;

#[test]
pub fn change_layer_position() {
    let engine = Engine::create();

    let scene = engine.scene.clone();

    let layer = ModelLayer::create();

    let _id = scene.add(layer.clone() as Arc<dyn RenderNode>);

    assert_eq!(layer.position.value().x, 0.0);

    layer.set_position(Point { x: 200.0, y: 100.0 }, None);

    engine.update(0.01);

    assert_eq!(layer.position.value().x, 200.0);
}
