use layers::prelude::*;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A no-op waker for manually driving futures in synchronous tests.
fn noop_waker() -> Waker {
    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

/// it should call the finish handler when the transaction is finished 1 time
#[test]
pub fn call_finish_transaction() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(&layer).unwrap();

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
    engine.add_layer(&layer).unwrap();

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
    engine.add_layer(&layer).unwrap();

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
    engine.add_layer(&layer).unwrap();

    // let s = Spring::with_duration_and_bounce(1.0, 0.2);
    // println!("spring {:#?}", s);
    // Spring {
    //     mass: 1.0,
    //     stiffness: 39.47842,
    //     damping: 10.053097,
    //     ...
    // }

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.0,
            timing: TimingFunction::Spring(Spring::new(1.0, 39.47842, 10.053097)),
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
            assert!(true);
        },
        true,
    );
    engine.update(0.5); // 0.5
    engine.update(0.5); // 1.0
    engine.update(0.5); // 1.5
    engine.update(0.5); // 2.0
    engine.update(0.5); // 2.5
    engine.update(0.5); // 3.0
    engine.update(0.5); // 3.5

    let called = called.read().unwrap();
    assert_eq!(*called, 1);
}

/// it should call the finish handler when the spring transaction is finished 1 time
#[test]
pub fn call_finish_transaction_spring_predictable() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(&layer).unwrap();

    let transaction = layer.set_position(
        Point { x: 200.0, y: 100.0 },
        Some(Transition {
            delay: 0.0,
            timing: TimingFunction::spring(1.0, 0.1),
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
            assert!(true);
        },
        true,
    );
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
    engine.add_layer(&layer).unwrap();

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
    engine.add_layer(&layer).unwrap();

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

/// TransitionFuture must resolve on the first poll when the transaction was
/// already completed (and cleaned up) before into_future() was called.
///
/// Previously the callback was registered lazily in poll(), so a transaction
/// that finished before poll() ran would leave the future pending forever.
#[test]
pub fn transition_future_resolves_when_transaction_already_done() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(&layer).unwrap();

    let transaction =
        layer.set_position(Point { x: 200.0, y: 100.0 }, Some(Transition::linear(0.1)));

    // Advance past the transition duration so it is completed and cleaned up.
    engine.update(0.2);

    // Create the future *after* the transaction is gone.
    let mut future = std::future::IntoFuture::into_future(transaction);
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    // Should resolve on the very first poll, not hang as Pending.
    let result = Pin::new(&mut future).poll(&mut cx);
    assert!(
        matches!(result, Poll::Ready(())),
        "TransitionFuture should return Poll::Ready on first poll when the transaction is already done"
    );
}

/// TransitionFuture must also resolve when the transaction finishes between
/// into_future() and the first poll() (the callback fires with no waker yet,
/// but the finished flag is set and poll() sees it).
#[test]
pub fn transition_future_resolves_when_transaction_completes_before_poll() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(&layer).unwrap();

    let transaction =
        layer.set_position(Point { x: 200.0, y: 100.0 }, Some(Transition::linear(0.1)));

    // Create the future (registers the callback eagerly) …
    let mut future = std::future::IntoFuture::into_future(transaction);

    // … then complete the transaction before the first poll.
    engine.update(0.2);

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    // The eagerly-registered callback has fired and set finished=true, so the
    // first poll should return Ready.
    let result = Pin::new(&mut future).poll(&mut cx);
    assert!(
        matches!(result, Poll::Ready(())),
        "TransitionFuture should return Poll::Ready on first poll when the transaction finished before poll() ran"
    );
}
