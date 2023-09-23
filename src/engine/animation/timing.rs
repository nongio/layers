use crate::easing::bezier_easing_function;

/// This module contains the animation related data structures
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Easing {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

// default for Easing
impl Default for Easing {
    fn default() -> Self {
        // Ease out
        Easing {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TimingFunction {
    Easing(Easing),
}

impl TimingFunction {
    pub fn value_at(&self, t: f32) -> f32 {
        match self {
            TimingFunction::Easing(Easing { x1, x2, y1, y2 }) => {
                bezier_easing_function(*x1, *x2, *y1, *y2, t)
            }
        }
    }
}

impl Default for TimingFunction {
    fn default() -> Self {
        TimingFunction::Easing(Easing::default())
    }
}

impl From<Easing> for TimingFunction {
    fn from(easing: Easing) -> Self {
        TimingFunction::Easing(easing)
    }
}
