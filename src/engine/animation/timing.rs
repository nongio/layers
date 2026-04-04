use super::{spring::Spring, Easing};

/// A single segment within a keyframe animation.
///
/// Each segment spans a `duration` in seconds and maps a range of
/// overall animation progress (`start_progress`..`end_progress`)
/// through its own `easing` curve.
#[derive(Clone, Copy, Debug)]
pub struct KeyframeSegment {
    /// How long this segment lasts, in seconds.
    pub duration: f32,
    /// The bezier easing curve applied within this segment.
    pub easing: Easing,
    /// The animation progress value at the start of this segment (0.0–1.0).
    pub start_progress: f32,
    /// The animation progress value at the end of this segment (0.0–1.0).
    pub end_progress: f32,
}

#[derive(Clone, Debug)]
/// Possible timing functions for an animation.
pub enum TimingFunction {
    Easing(Easing, f32),
    Spring(Spring),
    /// Piecewise timing: a sequence of segments, each with its own
    /// duration, easing, and progress range.
    Keyframes(Vec<KeyframeSegment>),
}

impl TimingFunction {
    pub fn linear(duration: f32) -> Self {
        TimingFunction::Easing(Easing::linear(), duration)
    }
    pub fn ease_in(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in(), duration)
    }
    pub fn ease_out(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_out(), duration)
    }
    pub fn ease_in_out(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in_out(), duration)
    }
    pub fn ease_out_quad(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_out_quad(), duration)
    }
    pub fn ease_in_quad(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in_quad(), duration)
    }
    pub fn ease_in_out_quad(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in_out_quad(), duration)
    }
    pub fn ease_out_cubic(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_out_cubic(), duration)
    }
    pub fn ease_in_cubic(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in_cubic(), duration)
    }
    pub fn ease_in_out_cubic(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in_out_cubic(), duration)
    }
    pub fn ease_out_quart(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_out_quart(), duration)
    }
    pub fn ease_in_quart(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in_quart(), duration)
    }
    pub fn ease_in_out_quart(duration: f32) -> Self {
        TimingFunction::Easing(Easing::ease_in_out_quart(), duration)
    }
    pub fn spring(duration: f32, bounce: f32) -> Self {
        TimingFunction::Spring(Spring::with_duration_and_bounce(duration, bounce))
    }

    pub fn spring_with_initial_velocity(duration: f32, bounce: f32, initial_velocity: f32) -> Self {
        TimingFunction::Spring(Spring::with_duration_bounce_and_velocity(
            duration,
            bounce,
            initial_velocity,
        ))
    }

    /// Build a keyframes timing function from a list of segments.
    pub fn keyframes(segments: Vec<KeyframeSegment>) -> Self {
        TimingFunction::Keyframes(segments)
    }

    pub fn update_at(&mut self, elapsed: f32) -> (f32, f32) {
        match self {
            TimingFunction::Easing(Easing { x1, x2, y1, y2 }, duration) => {
                let t = elapsed / *duration;
                let t = t.clamp(0.0, 1.0);
                let ease = bezier_easing::bezier_easing(*x1, *y1, *x2, *y2).unwrap();
                (ease(t), t)
            }
            TimingFunction::Spring(solver) => (solver.update_at(elapsed), elapsed),
            TimingFunction::Keyframes(segments) => {
                let total_duration: f32 = segments.iter().map(|s| s.duration).sum();
                if total_duration <= 0.0 || segments.is_empty() {
                    return (1.0, 1.0);
                }
                let t = elapsed / total_duration;
                let t_clamped = t.clamp(0.0, 1.0);

                // Find which segment the elapsed time falls into
                let mut accumulated = 0.0_f32;
                for seg in segments.iter() {
                    let seg_end = accumulated + seg.duration;
                    if elapsed < seg_end || (elapsed >= seg_end && seg_end >= total_duration) {
                        // We're in this segment
                        let seg_elapsed = elapsed - accumulated;
                        let seg_t = if seg.duration > 0.0 {
                            (seg_elapsed / seg.duration).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };
                        // Apply the segment's easing curve
                        let eased = bezier_easing::bezier_easing(
                            seg.easing.x1,
                            seg.easing.y1,
                            seg.easing.x2,
                            seg.easing.y2,
                        )
                        .unwrap()(seg_t);
                        // Map eased value to the segment's progress range
                        let progress =
                            seg.start_progress + (seg.end_progress - seg.start_progress) * eased;
                        return (progress, t_clamped);
                    }
                    accumulated = seg_end;
                }
                // Past all segments — return final progress
                let last = segments.last().unwrap();
                (last.end_progress, t_clamped)
            }
        }
    }
    pub fn done(&self, start: f32, current: f32) -> bool {
        match self {
            TimingFunction::Easing(_, duration) => current - start >= *duration,
            TimingFunction::Spring(solver) => solver.done(current - start),
            TimingFunction::Keyframes(segments) => {
                let total_duration: f32 = segments.iter().map(|s| s.duration).sum();
                current - start >= total_duration
            }
        }
    }
}

impl Default for TimingFunction {
    fn default() -> Self {
        TimingFunction::Easing(Easing::default(), 0.3)
    }
}

impl From<Easing> for TimingFunction {
    fn from(easing: Easing) -> Self {
        TimingFunction::Easing(easing, 0.3)
    }
}
