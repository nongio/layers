use lay_rs::prelude::*;
use std::sync::{Arc, RwLock};

/// it should call the finish handler when the transaction is finished 1 time
#[test]
pub fn call_finish_transaction() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(layer.clone());

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition::ease_out(0.5)),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();
    engine.on_finish(
        transaction,
        move |_: &Layer, _| {
            let mut c = c.write().unwrap();
            *c += 1;
            layer.set_position(
                Point { x: 200.0, y: 100.0 },
                Some(Transition::ease_out(0.3)),
            );
            // check we are not in a dead lock
            // assert!(true);
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
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(layer.clone());

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.2,
            ..Default::default()
        }),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();
    transaction.on_start(
        move |_: &Layer, _| {
            let mut c = c.write().unwrap();
            *c += 1;
        },
        true,
    );
    engine.update(0.1);
    // with a delay in the animation the start handler should not be called
    {
        let c = called.read().unwrap();
        assert_eq!(*c, 0);
    }
    // now it should be called
    engine.update(0.1);
    {
        let c = called.read().unwrap();
        assert_eq!(*c, 1);
    }
    // it should be called only once
    engine.update(0.1);

    {
        let c = called.read().unwrap();
        assert_eq!(*c, 1);
    }
}

/// it should call the update handler on every update until the transaction is finished
#[test]
pub fn call_update_transaction() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(layer.clone());

    let called = Arc::new(RwLock::new(0.0));
    let c = called.clone();
    layer
        .set_position(Point { x: 200.0, y: 100.0 }, Some(Transition::linear(0.1)))
        .on_update(
            move |_: &Layer, progress| {
                let mut c = c.write().unwrap();
                *c = progress;
            },
            false,
        );
    engine.update(0.05);
    {
        let called = called.read().unwrap();
        assert_eq!(*called, 0.5);
    }

    engine.update(0.05);
    {
        let called = called.read().unwrap();
        assert_eq!(*called, 1.0);
    }
}

/// it should call the finish handler when the spring transaction is finished 1 time
#[test]
pub fn call_finish_transaction_spring() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(layer.clone());

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.0,
            timing: TimingFunction::Spring(Spring::new(1.0, 100.0, 2.0)),
        }),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();
    engine.on_finish(
        transaction,
        move |_: &Layer, _| {
            let mut c = c.write().unwrap();
            *c += 1;
            layer.set_position(
                Point { x: 200.0, y: 100.0 },
                Some(Transition::ease_out(0.3)),
            );
            // check we are not in a dead lock
            // assert!(true);
        },
        true,
    );
    engine.update(2.0);
    engine.update(2.0);
    engine.update(2.0);

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}

/// it should call the finish handler when the spring transaction is finished 1 time
#[test]
pub fn call_finish_transaction_spring_predictable() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(layer.clone());

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.0,
            timing: TimingFunction::spring(1.0, 0.2),
        }),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();
    engine.on_finish(
        transaction,
        move |_: &Layer, _| {
            let mut c = c.write().unwrap();
            *c += 1;
            layer.set_position(
                Point { x: 200.0, y: 100.0 },
                Some(Transition::ease_out(0.3)),
            );
            // check we are not in a dead lock
            // assert!(true);
        },
        true,
    );
    engine.update(0.5);
    engine.update(0.5);
    engine.update(0.5);

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}

/// it should call the start handler when the transaction is started 1 time
#[test]
pub fn call_start_value() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(layer.clone());

    layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.2,
            ..Default::default()
        }),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    engine.on_start_value(
        layer.position_value_id(),
        move |_: &Layer, _| {
            let mut c = c.write().unwrap();
            *c += 1;
        },
        false,
    );
    engine.update(0.1);
    // with a delay in the animation the start handler should not be called
    {
        let c = called.read().unwrap();
        assert_eq!(*c, 0);
    }
    // now it should be called
    engine.update(0.1);
    {
        let c = called.read().unwrap();
        assert_eq!(*c, 1);
    }
    // it should be called only once
    engine.update(0.1);

    {
        let c = called.read().unwrap();
        assert_eq!(*c, 1);
    }
    layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.0,
            ..Default::default()
        }),
    );

    // it should be called again
    engine.update(0.1);
    {
        let c = called.read().unwrap();
        assert_eq!(*c, 2);
    }
}

/// it should call the finish handler when the transaction is started 1 time
#[test]
pub fn call_update_value() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(layer.clone());

    layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.0,
            ..Default::default()
        }),
    );
    let called = Arc::new(RwLock::new(0));
    let c = called.clone();

    engine.on_update_value(
        layer.position_value_id(),
        move |_: &Layer, _| {
            let mut c = c.write().unwrap();
            *c += 1;
        },
        false,
    );
    engine.update(0.1);

    {
        let c = called.read().unwrap();
        assert_eq!(*c, 1);
    }

    engine.update(0.1);
    {
        let c = called.read().unwrap();
        assert_eq!(*c, 2);
    }

    engine.update(0.1);

    {
        let c = called.read().unwrap();
        assert_eq!(*c, 3);
    }
    layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.0,
            ..Default::default()
        }),
    );

    // it should be called again
    engine.update(0.1);
    {
        let c = called.read().unwrap();
        assert_eq!(*c, 4);
    }
}
