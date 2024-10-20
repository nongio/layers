use layers::types::Point;
use layers::{
    engine::{animation::Transition, LayersEngine},
    prelude::Layer,
};
use std::sync::{Arc, RwLock};

/// it should call the finish handler when the transaction is finished 1 time
#[test]
pub fn call_finish_transaction() {
    let engine = LayersEngine::new(1000.0, 1000.0);
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
    engine.on_finish(
        transaction,
        move |_: &Layer, _| {
            println!("Transaction finished");
            let mut c = c.write().unwrap();
            *c += 1;
            layer.set_position(
                Point { x: 200.0, y: 100.0 },
                Some(Transition {
                    duration: 0.1,
                    ..Default::default()
                }),
            );
            // check we are not in a dead lock
            assert!(true);
        },
        true,
    );
    engine.update(0.2);
    engine.update(0.2);
    engine.update(0.2);

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}

/// it should call the start handler when the transaction is started 1 time
#[test]
pub fn call_start_transaction() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            duration: 1.0,
            ..Default::default()
        }),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();
    transaction.on_start(
        move |_: &Layer, _| {
            println!("Transaction started");
            let mut c = c.write().unwrap();
            *c += 1;
        },
        true,
    );
    engine.update(0.1);
    engine.update(0.1);
    engine.update(0.1);

    let called = called.read().unwrap();

    assert_eq!(*called, 1);
}

/// it should call the update handler on every update until the transaction is finished
#[test]
pub fn call_update_transaction() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    let called = Arc::new(RwLock::new(0.0));
    let c = called.clone();
    layer
        .set_position(
            Point { x: 200.0, y: 100.0 },
            Some(Transition {
                duration: 0.1,
                ..Default::default()
            }),
        )
        .on_update(
            move |_: &Layer, progress| {
                println!("Transaction update {}", progress);
                let mut c = c.write().unwrap();
                *c = progress;
            },
            true,
        );
    engine.update(0.05);
    engine.update(0.05);

    let called = called.read().unwrap();
    assert_eq!(*called, 1.0);
}
