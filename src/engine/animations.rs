use core::fmt;

use std::marker::Sync;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use super::command::{AnimatableValue, ValueChange};
use crate::easing::bezier_easing_function;

/// This module contains the animation related data structures

/// A trait for interpolating across time
pub trait TimingFunction {
    fn value_at(&self, t: f64) -> f64;
}

impl TimingFunction for Easing {
    fn value_at(&self, t: f64) -> f64 {
        let Easing { x1, x2, y1, y2 } = *self;
        bezier_easing_function(x1, x2, y1, y2, t)
    }
}

#[repr(C)]
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Transition<T: TimingFunction> {
    pub duration: f64,
    pub delay: f64,
    // easing
    pub timing: T,
}

impl Default for Transition<Easing> {
    fn default() -> Self {
        Transition {
            duration: 0.0,
            delay: 0.0,
            timing: Easing::default(),
        }
    }
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
        } = self;

        let mut t = (t - start) / duration;
        t = t.clamp(0.0, 1.0);
        (timing.value_at(t), t >= 1.0)
    }
}

impl fmt::Debug for Animation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}->{:?})", self.start, self.duration)
    }
}

static SYNC_VALUE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct SyncValue<V: Sync> {
    pub id: usize,
    value: Arc<RwLock<V>>,
}

impl<V: Sync + Clone> SyncValue<V> {
    pub fn new(value: V) -> SyncValue<V> {
        let value = Arc::new(RwLock::new(value));
        Self {
            id: SYNC_VALUE_COUNTER.fetch_add(1, Ordering::SeqCst),
            value,
        }
    }

    pub fn value(&self) -> V {
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, value: V) {
        *self.value.write().unwrap() = value;
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

impl<V: Sync + Clone> AnimatableValue<V> for SyncValue<V> {
    fn to(&self, to: V, transition: Option<Transition<Easing>>) -> ValueChange<V> {
        ValueChange {
            from: self.value(),
            to,
            target: self.clone(),
            transition,
        }
    }
}
