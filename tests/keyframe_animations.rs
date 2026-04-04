use layers::prelude::*;

/// Helper to create a simple two-segment keyframe timing:
/// first half holds at 0, second half eases linearly to 1.
fn hold_then_linear() -> Vec<KeyframeSegment> {
    vec![
        KeyframeSegment {
            duration: 0.5,
            easing: Easing::linear(),
            start_progress: 0.0,
            end_progress: 0.0, // hold
        },
        KeyframeSegment {
            duration: 0.5,
            easing: Easing::linear(),
            start_progress: 0.0,
            end_progress: 1.0, // animate
        },
    ]
}

#[test]
fn keyframe_timing_update_at_hold_segment() {
    let mut tf = TimingFunction::keyframes(hold_then_linear());

    // During the hold segment (0–0.5s), progress should stay at 0.0
    let (progress, _) = tf.update_at(0.0);
    assert!(
        (progress - 0.0).abs() < 0.001,
        "progress at t=0 should be 0.0, got {progress}"
    );

    let (progress, _) = tf.update_at(0.25);
    assert!(
        (progress - 0.0).abs() < 0.001,
        "progress at t=0.25 should be 0.0 (hold), got {progress}"
    );

    let (progress, _) = tf.update_at(0.49);
    assert!(
        (progress - 0.0).abs() < 0.001,
        "progress at t=0.49 should be ~0.0 (hold), got {progress}"
    );
}

#[test]
fn keyframe_timing_update_at_animate_segment() {
    let mut tf = TimingFunction::keyframes(hold_then_linear());

    // During the animate segment (0.5–1.0s), progress should go 0.0→1.0
    let (progress, _) = tf.update_at(0.5);
    assert!(
        (progress - 0.0).abs() < 0.01,
        "progress at t=0.5 should be ~0.0, got {progress}"
    );

    let (progress, _) = tf.update_at(0.75);
    assert!(
        (progress - 0.5).abs() < 0.01,
        "progress at t=0.75 should be ~0.5, got {progress}"
    );

    let (progress, _) = tf.update_at(1.0);
    assert!(
        (progress - 1.0).abs() < 0.01,
        "progress at t=1.0 should be ~1.0, got {progress}"
    );
}

#[test]
fn keyframe_timing_done() {
    let tf = TimingFunction::keyframes(hold_then_linear());

    // Total duration is 1.0s
    assert!(!tf.done(0.0, 0.5), "should not be done at 0.5s");
    assert!(!tf.done(0.0, 0.99), "should not be done at 0.99s");
    assert!(tf.done(0.0, 1.0), "should be done at 1.0s");
    assert!(tf.done(0.0, 1.5), "should be done past total duration");
}

#[test]
fn keyframe_timing_done_with_start_offset() {
    let tf = TimingFunction::keyframes(hold_then_linear());

    // Animation starts at t=2.0, total duration is 1.0s
    assert!(!tf.done(2.0, 2.5), "should not be done 0.5s in");
    assert!(
        tf.done(2.0, 3.0),
        "should be done at start + total_duration"
    );
}

#[test]
fn keyframe_three_segments() {
    // Three segments: ease-in to 30%, hold at 30%, ease-out to 100%
    let mut tf = TimingFunction::keyframes(vec![
        KeyframeSegment {
            duration: 0.3,
            easing: Easing::linear(),
            start_progress: 0.0,
            end_progress: 0.3,
        },
        KeyframeSegment {
            duration: 0.2,
            easing: Easing::linear(),
            start_progress: 0.3,
            end_progress: 0.3, // hold
        },
        KeyframeSegment {
            duration: 0.5,
            easing: Easing::linear(),
            start_progress: 0.3,
            end_progress: 1.0,
        },
    ]);

    // End of first segment
    let (progress, _) = tf.update_at(0.3);
    assert!(
        (progress - 0.3).abs() < 0.01,
        "end of seg 0: expected ~0.3, got {progress}"
    );

    // Middle of hold segment
    let (progress, _) = tf.update_at(0.4);
    assert!(
        (progress - 0.3).abs() < 0.01,
        "mid hold: expected ~0.3, got {progress}"
    );

    // End of hold segment
    let (progress, _) = tf.update_at(0.5);
    assert!(
        (progress - 0.3).abs() < 0.01,
        "end of hold: expected ~0.3, got {progress}"
    );

    // Middle of final segment (0.75s = 0.5 into a 0.5s segment = 50%)
    let (progress, _) = tf.update_at(0.75);
    let expected = 0.3 + (1.0 - 0.3) * 0.5; // 0.65
    assert!(
        (progress - expected).abs() < 0.01,
        "mid final: expected ~{expected}, got {progress}"
    );

    // End
    let (progress, _) = tf.update_at(1.0);
    assert!(
        (progress - 1.0).abs() < 0.01,
        "end: expected ~1.0, got {progress}"
    );
}

#[test]
fn keyframe_normalized_time_output() {
    let mut tf = TimingFunction::keyframes(hold_then_linear());

    // The second return value should be normalized time (0.0–1.0)
    let (_, t) = tf.update_at(0.0);
    assert!((t - 0.0).abs() < 0.001);

    let (_, t) = tf.update_at(0.5);
    assert!((t - 0.5).abs() < 0.001);

    let (_, t) = tf.update_at(1.0);
    assert!((t - 1.0).abs() < 0.001);
}

#[test]
fn keyframe_empty_segments() {
    let mut tf = TimingFunction::keyframes(vec![]);

    // Empty keyframes should immediately resolve
    let (progress, _) = tf.update_at(0.0);
    assert!(
        (progress - 1.0).abs() < 0.001,
        "empty keyframes should return 1.0"
    );

    let tf = TimingFunction::keyframes(vec![]);
    assert!(
        tf.done(0.0, 0.0),
        "empty keyframes should be immediately done"
    );
}

#[test]
fn keyframe_single_segment() {
    let mut tf = TimingFunction::keyframes(vec![KeyframeSegment {
        duration: 1.0,
        easing: Easing::linear(),
        start_progress: 0.0,
        end_progress: 1.0,
    }]);

    // Should behave like a simple linear animation
    let (progress, _) = tf.update_at(0.0);
    assert!((progress - 0.0).abs() < 0.001);

    let (progress, _) = tf.update_at(0.5);
    assert!((progress - 0.5).abs() < 0.001);

    let (progress, _) = tf.update_at(1.0);
    assert!((progress - 1.0).abs() < 0.001);
}

#[test]
fn keyframe_with_easing_curves() {
    // Use ease_in for first segment — progress should lag behind linear
    let mut tf = TimingFunction::keyframes(vec![KeyframeSegment {
        duration: 1.0,
        easing: Easing::ease_in(),
        start_progress: 0.0,
        end_progress: 1.0,
    }]);

    let (progress, _) = tf.update_at(0.5);
    // ease_in at t=0.5 should be less than 0.5 (starts slow)
    assert!(
        progress < 0.5,
        "ease_in at t=0.5 should be < 0.5, got {progress}"
    );
    assert!(
        progress > 0.0,
        "ease_in at t=0.5 should be > 0.0, got {progress}"
    );
}

#[test]
fn keyframe_layer_integration() {
    // Test that keyframes work end-to-end with the engine
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(&layer).unwrap();

    let initial_pos = layer.position();

    // Hold for 0.5s, then animate to (100, 100) over 0.5s
    layer.set_position((100.0, 100.0), Transition::keyframes(hold_then_linear()));

    // Advance 0.4s — still in hold segment, position should not change
    engine.update(0.2);
    engine.update(0.2);
    assert_eq!(
        layer.position(),
        initial_pos,
        "position should not change during hold segment"
    );

    // Advance to 0.75s — halfway through animate segment
    engine.update(0.2);
    engine.update(0.15);
    let pos = layer.position();
    assert!(
        pos.x > 0.0 && pos.x < 100.0,
        "position should be mid-animation at t=0.75, got {:?}",
        pos
    );

    // Advance past end
    engine.update(0.5);
    engine.update(0.5);
    let pos = layer.position();
    assert!(
        (pos.x - 100.0).abs() < 1.0 && (pos.y - 100.0).abs() < 1.0,
        "position should be at target after animation, got {:?}",
        pos
    );
}

#[test]
fn keyframe_transition_constructor() {
    let t = Transition::keyframes(vec![KeyframeSegment {
        duration: 0.5,
        easing: Easing::ease_out(),
        start_progress: 0.0,
        end_progress: 1.0,
    }]);
    assert_eq!(t.delay, 0.0);
    match t.timing {
        TimingFunction::Keyframes(ref segments) => {
            assert_eq!(segments.len(), 1);
            assert_eq!(segments[0].duration, 0.5);
        }
        _ => panic!("expected Keyframes timing"),
    }
}
