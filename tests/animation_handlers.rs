use layers::prelude::*;
use std::sync::{Arc, RwLock};

/// Test that animation callbacks are triggered correctly
#[test]
pub fn test_animation_start_callback() {
    let engine = Engine::create(1000.0, 1000.0);

    // Create an animation
    let animation = engine.add_animation_from_transition(&Transition::linear(0.5), true);

    let called = Arc::new(RwLock::new(false));
    let c = called.clone();

    engine.on_animation_start(
        animation,
        move |_progress| {
            let mut c = c.write().unwrap();
            *c = true;
        },
        true,
    );

    // Update to trigger the animation start
    engine.update(0.016);

    let c = called.read().unwrap();
    assert!(*c, "Animation start callback should have been called");
}

/// Test that animation update callbacks are triggered
#[test]
pub fn test_animation_update_callback() {
    let engine = Engine::create(1000.0, 1000.0);

    let animation = engine.add_animation_from_transition(&Transition::linear(0.5), true);

    let update_count = Arc::new(RwLock::new(0));
    let c = update_count.clone();

    engine.on_animation_update(
        animation,
        move |_progress| {
            let mut c = c.write().unwrap();
            *c += 1;
        },
        false, // Don't remove after first call
    );

    // Update multiple times
    engine.update(0.1);
    engine.update(0.1);
    engine.update(0.1);

    let count = update_count.read().unwrap();
    assert!(
        *count >= 3,
        "Animation update callback should have been called at least 3 times, got {}",
        *count
    );
}

/// Test that animation finish callbacks are triggered
#[test]
pub fn test_animation_finish_callback() {
    let engine = Engine::create(1000.0, 1000.0);

    let animation = engine.add_animation_from_transition(&Transition::linear(0.2), true);

    let finished = Arc::new(RwLock::new(false));
    let c = finished.clone();

    engine.on_animation_finish(
        animation,
        move |progress: f32| {
            let mut c = c.write().unwrap();
            *c = true;
            // Progress should be 1.0 when animation finishes
            assert!(
                (progress - 1.0).abs() < 0.01,
                "Expected progress to be 1.0, got {}",
                progress
            );
        },
        true,
    );

    // Run the animation to completion
    for _ in 0..20 {
        engine.update(0.05);
    }

    let c = finished.read().unwrap();
    assert!(*c, "Animation finish callback should have been called");
}

/// Test multiple callbacks on the same animation
#[test]
pub fn test_multiple_animation_callbacks() {
    let engine = Engine::create(1000.0, 1000.0);

    let animation = engine.add_animation_from_transition(&Transition::linear(0.2), true);

    let started = Arc::new(RwLock::new(false));
    let updated = Arc::new(RwLock::new(false));
    let finished = Arc::new(RwLock::new(false));

    let s = started.clone();
    let u = updated.clone();
    let f = finished.clone();

    engine.on_animation_start(
        animation,
        move |_| {
            *s.write().unwrap() = true;
        },
        true,
    );

    engine.on_animation_update(
        animation,
        move |_| {
            *u.write().unwrap() = true;
        },
        false,
    );

    engine.on_animation_finish(
        animation,
        move |_| {
            *f.write().unwrap() = true;
        },
        true,
    );

    // Start
    engine.update(0.016);
    assert!(*started.read().unwrap(), "Start should be called");

    // Update
    engine.update(0.05);
    assert!(*updated.read().unwrap(), "Update should be called");

    // Finish
    for _ in 0..10 {
        engine.update(0.05);
    }
    assert!(*finished.read().unwrap(), "Finish should be called");
}

/// Test that once=true removes callback after first call
#[test]
pub fn test_animation_callback_once() {
    let engine = Engine::create(1000.0, 1000.0);

    let animation = engine.add_animation_from_transition(&Transition::linear(0.5), true);

    let count = Arc::new(RwLock::new(0));
    let c = count.clone();

    engine.on_animation_start(
        animation,
        move |_| {
            *c.write().unwrap() += 1;
        },
        true, // Remove after first call
    );

    // Update multiple times - callback should only be called once
    engine.update(0.016);
    engine.update(0.016);
    engine.update(0.016);

    let c = count.read().unwrap();
    assert_eq!(*c, 1, "Callback with once=true should only be called once");
}

/// Test animation callbacks track progress correctly
#[test]
pub fn test_animation_progress_tracking() {
    let engine = Engine::create(1000.0, 1000.0);

    let animation = engine.add_animation_from_transition(&Transition::linear(1.0), true);

    let progresses = Arc::new(RwLock::new(Vec::new()));
    let p = progresses.clone();

    engine.on_animation_update(
        animation,
        move |progress| {
            p.write().unwrap().push(progress);
        },
        false,
    );

    // Update through the animation
    for _ in 0..10 {
        engine.update(0.1);
    }

    let p = progresses.read().unwrap();
    assert!(!p.is_empty(), "Should have recorded some progress values");

    // Progress should generally increase
    for i in 1..p.len() {
        assert!(p[i] >= p[i - 1] - 0.01, "Progress should be non-decreasing");
    }
}
