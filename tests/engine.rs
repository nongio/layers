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

#[test]
pub fn test_independent_engines() {
    // Create two separate engines
    let engine1 = Engine::create(800.0, 600.0);
    let engine2 = Engine::create(1024.0, 768.0);

    // Ensure they have different IDs
    assert_ne!(engine1.id, engine2.id);

    // Create a layer for each engine
    let layer1 = engine1.new_layer();
    let layer2 = engine2.new_layer();

    // Give them different sizes
    layer1.set_size(Size::points(200.0, 150.0), None);
    layer2.set_size(Size::points(300.0, 250.0), None);

    // Add them to their respective engines
    engine1.add_layer(&layer1);
    engine2.add_layer(&layer2);

    // Update both engines
    engine1.update(0.016);
    engine2.update(0.032);

    // Modify each layer independently
    layer1.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    layer2.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 255, 255),
        },
        None,
    );

    // Update both engines again
    engine1.update(0.016);
    engine2.update(0.016);

    // Verify the engines have different root layers (comparing node IDs)
    let root1 = engine1.scene_root().unwrap();
    let root2 = engine2.scene_root().unwrap();

    // Verify each engine's layer has the correct size
    let (width1, height1) = engine1.node_render_size(&root1);
    let (width2, height2) = engine2.node_render_size(&root2);

    assert_eq!(width1, 200.0);
    assert_eq!(height1, 150.0);
    assert_eq!(width2, 300.0);
    assert_eq!(height2, 250.0);
}

#[test]
fn removing_layer_subtree_triggers_layout() {
    let engine = Engine::create(800.0, 600.0);

    let root = engine.new_layer();
    root.set_size(Size::points(400.0, 400.0), None);

    let child = engine.new_layer();
    child.set_size(Size::points(200.0, 200.0), None);

    let grandchild = engine.new_layer();
    grandchild.set_size(Size::points(100.0, 100.0), None);

    child.add_sublayer(&grandchild);
    root.add_sublayer(&child);

    engine.add_layer(&root);

    // Allow the engine to process the additions so layout nodes exist.
    engine.update(0.016);

    // Removing the subtree in the same frame causes the parent layout node to be
    // dropped before the child, which makes layout.mark_dirty panic.
    grandchild.remove();
    child.remove();

    // The panic is triggered during cleanup inside this update.
    engine.update(0.016);
}
