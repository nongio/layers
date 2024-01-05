use core::fmt;

pub mod timing;

use self::timing::TimingFunction;

/// Transition is a data structure that contains the information needed to
/// create an animation that can start at a later time.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub duration: f32,
    pub delay: f32,
    // easing
    pub timing: TimingFunction,
}
/// Default Transition, 0.3 seconds, no delay, ease out
impl Default for Transition {
    fn default() -> Self {
        Transition {
            duration: 0.3,
            delay: 0.0,
            timing: TimingFunction::default(),
        }
    }
}
/// Animation is a data structure that contains the information needed to
/// animate a property.
#[derive(Clone)]
pub struct Animation {
    pub start: f32,
    pub duration: f32,
    pub timing: TimingFunction,
}

// getter for Animation value
impl Animation {
    pub fn value_at(&self, t: f32) -> (f32, f32) {
        let Animation {
            start,
            duration,
            timing,
        } = self;

        let mut t = (t - start) / duration;
        t = t.clamp(0.0, 1.0);
        (timing.value_at(t), t)
    }
}

impl fmt::Debug for Animation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}->{:?})", self.start, self.duration)
    }
}
