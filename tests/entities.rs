use hello::engine::scene::Scene;
use hello::layers::layer::ModelLayer;
use hello::types::Point;
use std::sync::Arc;

#[test]
pub fn change_layer_position() {
    let scene = Scene::create();

    let layer = ModelLayer::create();
    
    let layer_id = scene.add_renderable(layer.clone());

    assert_eq!(layer.position.value().x, 0.0);

    layer.position(Point { x: 200.0, y: 100.0 }, None);
    // scene.add_change(id, change);
    scene.update(0.01);

    assert_eq!(layer.position.value().x, 200.0);
}
