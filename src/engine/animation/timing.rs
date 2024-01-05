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

/// Easing functions are used to interpolate between two values
/// over a period of time. The default easing function is ease out.
/// The easing is calculated using a bezier curve with 2 control points.
///
impl Easing {
    pub fn ease_out() -> Self {
        Easing::default()
    }
    pub fn ease_in() -> Self {
        Easing {
            x1: 0.42,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        }
    }
    pub fn ease_in_out() -> Self {
        Easing {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        }
    }
    pub fn linear() -> Self {
        Easing {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
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
                bezier_easing_function(*x1 as f64, *x2 as f64, *y1 as f64, *y2 as f64, t as f64)
                    as f32
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
