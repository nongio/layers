use std::sync::{Arc, RwLock};

use lay_rs::prelude::*;
use lay_rs::types::Size;

/// it should call the pointer move handler
#[test]
pub fn pointer_move() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_position((0.0, 0.0), None);
    engine.add_layer(layer.clone());

    engine.update(0.016);
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    layer.add_on_pointer_move(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("pointer move!!");
    });
    let root_id = engine.scene_root().unwrap();
    engine.pointer_move((0.0, 0.0), root_id.0);

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}

/// it should not call the pointer move handler
#[test]
pub fn pointer_doesnt_move() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_position((200.0, 200.0), None);
    engine.add_layer(layer.clone());
    engine.update(0.016);

    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    layer.add_on_pointer_move(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("pointer move!!");
    });
    let root_id = engine.scene_root().unwrap();
    engine.pointer_move((0.0, 0.0), root_id.0);

    let called = called.read().unwrap();
    assert_eq!(*called, 0);
}

/// it should not call the pointer move handler
#[test]
pub fn pointer_move_nested() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_position((200.0, 200.0), None);
    engine.add_layer(layer.clone());

    let layer2 = engine.new_layer();
    layer2.set_size(Size::points(200.0, 200.0), None);
    layer2.set_position((200.0, 200.0), None);
    engine.append_layer(layer2.clone(), layer.id());

    engine.update(0.016);

    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    layer2.add_on_pointer_move(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("pointer move!!");
    });
    let root_id = engine.scene_root().unwrap();

    engine.pointer_move((400.0, 400.0), root_id.0);

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}

/// it should not call the pointer move handler
#[test]
pub fn pointer_move_nested_parent() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_position((200.0, 200.0), None);
    engine.add_layer(layer.clone());

    let layer2 = engine.new_layer();
    layer2.set_size(Size::points(200.0, 200.0), None);
    layer2.set_position((200.0, 200.0), None);
    engine.append_layer(layer2.clone(), layer.id());

    engine.update(0.016);

    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    layer.add_on_pointer_move(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("pointer move!!");
    });
    let root_id = engine.scene_root().unwrap();

    engine.pointer_move((210.0, 210.0), root_id.0);

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}
/// it should not call the pointer move handler
#[test]
pub fn pointer_doesnt_move_nested() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_position((200.0, 200.0), None);
    engine.add_layer(layer.clone());

    let layer2 = engine.new_layer();
    layer2.set_size(Size::points(200.0, 200.0), None);
    layer2.set_position((200.0, 200.0), None);
    engine.append_layer(layer2.clone(), layer.id());

    engine.update(0.016);

    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    layer2.add_on_pointer_move(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("pointer move!!");
    });
    let root_id = engine.scene_root().unwrap();

    engine.pointer_move((100.0, 100.0), root_id.0);

    let called = called.read().unwrap();
    assert_eq!(*called, 0);
}

/// it should not call the pointer move handler
#[test]
pub fn pointer_remove() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_position((0.0, 0.0), None);

    engine.add_layer(layer.clone());

    engine.update(0.016);
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    let handler_id = layer.add_on_pointer_move(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("**** pointer move!!");
    });

    layer.remove_on_pointer_move(handler_id);
    let root_id = engine.scene_root().unwrap();

    engine.pointer_move((0.0, 0.0), root_id.0);

    let called = called.read().unwrap();
    assert_eq!(*called, 0);
}

/// it should not call the pointer move handler
#[test]
pub fn pointer_in_out_nested_parent() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_position((200.0, 200.0), None);
    engine.add_layer(layer.clone());

    let layer2 = engine.new_layer();
    layer2.set_size(Size::points(200.0, 200.0), None);
    layer2.set_position((200.0, 200.0), None);
    engine.append_layer(layer2.clone(), layer.id());

    engine.update(0.016);

    let called = Arc::new(RwLock::new(0));

    let root_id = engine.scene_root().unwrap();

    let c = called.clone();
    layer.add_on_pointer_in(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("pointer in!!");
    });

    let c = called.clone();
    layer.add_on_pointer_out(move |_, _, _| {
        let mut c = c.write().unwrap();
        *c += 1;
        println!("pointer out!!");
    });

    let _c = called.clone();

    engine.pointer_move((210.0, 210.0), root_id.0);
    {
        let called = called.read().unwrap();
        assert_eq!(*called, 1);
    }

    engine.pointer_move((400.0, 400.0), root_id.0);
    {
        let called = called.read().unwrap();
        assert_eq!(*called, 2);
    }
}
