use core::fmt;

pub mod spring;
pub mod timing;
use timing::Easing;

use self::timing::TimingFunction;

/// Transition is a data structure that contains the information needed to
/// create an animation that can start at a later time.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    // pub duration: f32,
    pub delay: f32,
    // easing
    pub timing: TimingFunction,
}
/// Default Transition, 0.3 seconds, no delay, ease out
impl Default for Transition {
    fn default() -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::Easing(Easing::default(), 0.3),
        }
    }
}

impl Transition {
    pub fn linear(duration: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::linear(duration),
        }
    }
    pub fn ease_in(duration: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::ease_in(duration),
        }
    }
    pub fn ease_out(duration: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::ease_out(duration),
        }
    }
    pub fn ease_in_out(duration: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::ease_in_out(duration),
        }
    }
    pub fn ease_out_quad(duration: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::ease_out_quad(duration),
        }
    }
    pub fn ease_in_quad(duration: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::ease_in_quad(duration),
        }
    }
    pub fn ease_in_out_quad(duration: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::ease_in_out_quad(duration),
        }
    }
    pub fn spring(duration: f32, bounce: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::spring(duration, bounce),
        }
    }
    pub fn spring_with_velocity(duration: f32, bounce: f32, velocity: f32) -> Self {
        Transition {
            delay: 0.0,
            timing: TimingFunction::spring_with_initial_velocity(duration, bounce, velocity),
        }
    }
}
/// Animation is a data structure that contains the information needed to
/// animate a property.
#[derive(Clone, Default)]
pub struct Animation {
    pub start: f32,
    // pub duration: f32,
    pub timing: TimingFunction,
}

// getter for Animation value
impl Animation {
    pub fn update_at(&mut self, current_time: f32) -> (f32, f32) {
        let Animation {
            start,
            // duration,
            timing,
        } = self;
        let elapsed = current_time - *start;
        // t = t.clamp(0.0, 1.0);
        // println!("[{} / {}] ({}, {})", time, duration, t, timing.value_at(t));
        timing.update_at(elapsed)
    }
    pub fn done(&self, current_time: f32) -> bool {
        self.timing.done(self.start, current_time)
    }
}

impl fmt::Debug for Animation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "([{:.5}]->{:?})", self.start, self.timing)
    }
}
