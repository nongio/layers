// use crate::easing::bezier_easing_function;

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
    pub fn ease_out_quad() -> Self {
        Easing {
            x1: 0.25,
            y1: 0.46,
            x2: 0.45,
            y2: 0.94,
        }
    }
    pub fn ease_in_quad() -> Self {
        Easing {
            x1: 0.55,
            y1: 0.085,
            x2: 0.68,
            y2: 0.53,
        }
    }
    pub fn ease_in_out_quad() -> Self {
        Easing {
            x1: 0.455,
            y1: 0.03,
            x2: 0.515,
            y2: 0.955,
        }
    }
    pub fn ease_out_cubic() -> Self {
        Easing {
            x1: 0.215,
            y1: 0.61,
            x2: 0.355,
            y2: 1.0,
        }
    }
    pub fn ease_in_cubic() -> Self {
        Easing {
            x1: 0.55,
            y1: 0.055,
            x2: 0.675,
            y2: 0.19,
        }
    }
    pub fn ease_in_out_cubic() -> Self {
        Easing {
            x1: 0.645,
            y1: 0.045,
            x2: 0.355,
            y2: 1.0,
        }
    }
    pub fn ease_out_quart() -> Self {
        Easing {
            x1: 0.165,
            y1: 0.84,
            x2: 0.44,
            y2: 1.0,
        }
    }
    pub fn ease_in_quart() -> Self {
        Easing {
            x1: 0.895,
            y1: 0.03,
            x2: 0.685,
            y2: 0.22,
        }
    }
    pub fn ease_in_out_quart() -> Self {
        Easing {
            x1: 0.77,
            y1: 0.0,
            x2: 0.175,
            y2: 1.0,
        }
    }
    pub fn ease_out_quint() -> Self {
        Easing {
            x1: 0.23,
            y1: 1.0,
            x2: 0.32,
            y2: 1.0,
        }
    }
    pub fn ease_in_quint() -> Self {
        Easing {
            x1: 0.755,
            y1: 0.05,
            x2: 0.855,
            y2: 0.06,
        }
    }
    pub fn ease_in_out_quint() -> Self {
        Easing {
            x1: 0.86,
            y1: 0.0,
            x2: 0.07,
            y2: 1.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum TimingFunction {
    Easing(Easing),
}

impl TimingFunction {
    pub fn linear() -> Self {
        TimingFunction::Easing(Easing::linear())
    }
    pub fn ease_in() -> Self {
        TimingFunction::Easing(Easing::ease_in())
    }
    pub fn ease_out() -> Self {
        TimingFunction::Easing(Easing::ease_out())
    }
    pub fn ease_in_out() -> Self {
        TimingFunction::Easing(Easing::ease_in_out())
    }
    pub fn ease_out_quad() -> Self {
        TimingFunction::Easing(Easing::ease_out_quad())
    }
    pub fn ease_in_quad() -> Self {
        TimingFunction::Easing(Easing::ease_in_quad())
    }
    pub fn ease_in_out_quad() -> Self {
        TimingFunction::Easing(Easing::ease_in_out_quad())
    }
    pub fn ease_out_cubic() -> Self {
        TimingFunction::Easing(Easing::ease_out_cubic())
    }
    pub fn ease_in_cubic() -> Self {
        TimingFunction::Easing(Easing::ease_in_cubic())
    }
    pub fn ease_in_out_cubic() -> Self {
        TimingFunction::Easing(Easing::ease_in_out_cubic())
    }
    pub fn ease_out_quart() -> Self {
        TimingFunction::Easing(Easing::ease_out_quart())
    }
    pub fn ease_in_quart() -> Self {
        TimingFunction::Easing(Easing::ease_in_quart())
    }
    pub fn ease_in_out_quart() -> Self {
        TimingFunction::Easing(Easing::ease_in_out_quart())
    }
    pub fn value_at(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            TimingFunction::Easing(Easing { x1, x2, y1, y2 }) => {
                let ease = bezier_easing::bezier_easing(*x1, *y1, *x2, *y2).unwrap();
                ease(t)
                //bezier_easing_function(*x1 as f64, *x2 as f64, *y1 as f64, *y2 as f64, t as f64)
                //    as f32
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
