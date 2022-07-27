use core::fmt;

use std::marker::Sync;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};

use crate::easing::{bezier_easing_function, Interpolable};

// A trait for interpolating across time
pub trait TimingFunction {
    fn value_at(&self, t: f64) -> f64;
}

impl TimingFunction for Easing {
    fn value_at(&self, t: f64) -> f64 {
        let Easing { x1, x2, y1, y2 } = *self;
        bezier_easing_function(x1, x2, y1, y2, t)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Easing {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
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
pub struct Transition<T: TimingFunction> {
    pub duration: f64,
    pub delay: f64,
    // easing
    pub timing: T,
}

#[derive(Clone)]

pub struct Animation {
    pub start: f64,
    pub duration: f64,
    pub timing: Easing,
}

// getter for Animation value
impl Animation {
    pub fn value(&self, t: f64) -> (f64, bool) {
        let Animation {
            start,
            duration,
            timing,
        } = &*self;

        let mut t = (t - start) / duration;
        if t < 0.0 {
            t = 0.0;
        } else if t > 1.0 {
            t = 1.0;
        }
        (timing.value_at(t), t >= 1.0)
    }
}

impl fmt::Debug for Animation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}->{:?})", self.start, self.duration)
    }
}

#[derive(Clone, Debug)]
pub struct ValueChange<V: Interpolable + Sync> {
    pub from: V,
    pub to: V,
    pub target: AnimatedValue<V>,
    pub transition: Option<Transition<Easing>>,
}

static OBJECT_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct AnimatedValue<V: Interpolable + Sync> {
    pub id: usize,
    pub value: Arc<RwLock<V>>,
}

pub trait AnimatedValueTrait<V: Interpolable + Sync> {
    fn to(&self, to: V, transition: Option<Transition<Easing>>) -> ValueChange<V>;
}

impl<V: Interpolable + Sync + Clone> AnimatedValue<V> {
    pub fn new(value: V) -> AnimatedValue<V> {
        let id = OBJECT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let value = Arc::new(RwLock::new(value));
        Self { id, value }
    }

    pub fn value(&self) -> V {
        self.value.read().unwrap().clone()
    }

    pub fn to(&self, to: V, transition: Option<Transition<Easing>>) -> ValueChange<V> {
        ValueChange {
            from: self.value(),
            to,
            target: self.clone(),
            transition,
        }
    }
}

impl<V: Interpolable + Sync + Clone> AnimatedValueTrait<V> for AnimatedValue<V> {
    fn to(&self, to: V, transition: Option<Transition<Easing>>) -> ValueChange<V> {
        ValueChange {
            from: self.value(),
            to,
            target: self.clone(),
            transition,
        }
    }
}
