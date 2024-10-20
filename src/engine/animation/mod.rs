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

impl Transition {
    pub fn linear(duration: f32) -> Self {
        Transition {
            duration,
            delay: 0.0,
            timing: TimingFunction::linear(),
        }
    }
    pub fn ease_in(duration: f32) -> Self {
        Transition {
            duration,
            delay: 0.0,
            timing: TimingFunction::ease_in(),
        }
    }
    pub fn ease_out(duration: f32) -> Self {
        Transition {
            duration,
            delay: 0.0,
            timing: TimingFunction::ease_out(),
        }
    }
    pub fn ease_in_out(duration: f32) -> Self {
        Transition {
            duration,
            delay: 0.0,
            timing: TimingFunction::ease_in_out(),
        }
    }
    pub fn ease_out_quad(duration: f32) -> Self {
        Transition {
            duration,
            delay: 0.0,
            timing: TimingFunction::ease_out_quad(),
        }
    }
    pub fn ease_in_quad(duration: f32) -> Self {
        Transition {
            duration,
            delay: 0.0,
            timing: TimingFunction::ease_in_quad(),
        }
    }
    pub fn ease_in_out_quad(duration: f32) -> Self {
        Transition {
            duration,
            delay: 0.0,
            timing: TimingFunction::ease_in_out_quad(),
        }
    }
}
/// Animation is a data structure that contains the information needed to
/// animate a property.
#[derive(Clone, Default)]
pub struct Animation {
    pub start: f32,
    pub duration: f32,
    pub timing: TimingFunction,
}

// getter for Animation value
impl Animation {
    pub fn value_at(&self, time: f32) -> (f32, f32) {
        let Animation {
            start,
            duration,
            timing,
        } = self;
        let mut t = (time - start) / duration;
        t = t.clamp(0.0, 1.0);
        // println!("[{} / {}] ({}, {})", time, duration, t, timing.value_at(t));
        (timing.value_at(t), t)
    }
}

impl fmt::Debug for Animation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}->{:?})", self.start, self.duration)
    }
}
