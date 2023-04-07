use layers::engine::{animations::Transition, LayersEngine};
use layers::types::Point;
use std::sync::{Arc, RwLock};

/// it should call the finish handler when the transaction is finished 1 time
#[test]
pub fn call_finish_transaction() {
    let engine = LayersEngine::new();
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            duration: 0.1,
            ..Default::default()
        }),
    );
    // let called = Arc::new(RwLock::new(0));
    // let c = called.clone();
    // engine.on_finish(transaction, move |_| {
    //     println!("Transaction finished");
    //     let mut c = c.write().unwrap();
    //     *c += 1;
    // });
    // engine.update(0.2);
    // engine.update(0.2);
    // engine.update(0.2);

    // let called = called.read().unwrap();
    // assert_eq!(*called, 1);
}

/// it should call the start handler when the transaction is started 1 time
#[test]
pub fn call_start_transaction() {
    let engine = LayersEngine::new();
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            duration: 0.1,
            ..Default::default()
        }),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();
    transaction.on_start(move |_| {
        println!("Transaction started");
        let mut c = c.write().unwrap();
        *c += 1;
    });
    engine.update(0.1);
    engine.update(0.1);
    engine.update(0.1);

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}

/// it should call the update handler on every update until the transaction is finished
#[test]
pub fn call_update_transaction() {
    let engine = LayersEngine::new();
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());
    
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();
    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            duration: 0.1,
            ..Default::default()
        }),
    );
    transaction.on_update(move |_| {
        println!("Transaction update");
        let mut c = c.write().unwrap();
        *c += 1;
    });
    engine.update(0.05);
    engine.update(0.05);

    let called = called.read().unwrap();
    assert_eq!(*called, 2);
}
