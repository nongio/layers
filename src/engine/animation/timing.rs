use super::{spring::Spring, Easing};

#[derive(Clone, Copy, Debug)]
/// Possible timing functions for an animation.
pub enum TimingFunction {
    Easing(Easing, f32),
    Spring(Spring),
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

    pub fn update_at(&mut self, elapsed: f32) -> (f32, f32) {
        match self {
            TimingFunction::Easing(Easing { x1, x2, y1, y2 }, duration) => {
                let t = elapsed / *duration;
                let t = t.clamp(0.0, 1.0);
                let ease = bezier_easing::bezier_easing(*x1, *y1, *x2, *y2).unwrap();
                (ease(t), t)
            }
            TimingFunction::Spring(solver) => (solver.update_at(elapsed), elapsed),
        }
    }
    pub fn done(&self, start: f32, current: f32) -> bool {
        match self {
            TimingFunction::Easing(_, duration) => current - start >= *duration,
            TimingFunction::Spring(solver) => solver.done(current - start),
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
