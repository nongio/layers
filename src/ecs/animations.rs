use core::fmt;
use std::marker::Sync;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};

use crate::easing::{bezier_easing_function, interpolate, Interpolable};

pub trait TimingFunction {
    fn value_at(&self, t: f64) -> f64;
}


impl TimingFunction for Easing {
    fn value_at(&self, t: f64) -> f64 {
        let Easing{x1, x2, y1, y2} = *self;
        bezier_easing_function(x1, x2, y1, y2, t)
    }
}



// getter for Animation value
impl<V:Interpolable> Animation<V> {
    
    pub fn value(&self, t: f64) -> V {
        let Animation {
            start,
            duration,
            timing,
            from,
            to,
        } = &*self;
        let mut t = (t - start) / duration;
        if t < 0.0 {
            t = 0.0;
        } else if t > 1.0 {
            t = 1.0;
        }
        t = timing.value_at(t);

        interpolate::<V>(*from, *to, t)
    }
}

pub trait Animatable<T> : Sync {
    fn value_at(&mut self, t: f64) -> T;
}


  
#[derive(Clone, Copy, Debug)]
pub struct Easing{
    pub x1:f64, 
    pub y1:f64, 
    pub x2:f64, 
    pub y2:f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Transition<T: TimingFunction> {
    pub duration: f64,
    pub delay: f64,
    // easing
    pub timing: T,
}
#[derive(Copy, Clone)]
pub struct Animation<V:Interpolable> {
    pub start: f64,
    pub duration: f64,
    pub from: V,
    pub to: V,
    pub timing: Easing,
}




pub struct AnimatedValue<V:Interpolable> {
    value: Arc<RwLock<V>>, 
    animation: Arc<RwLock<Option<Animation<V>>>>,
}

// implement the trait Clone for AnimatedValue
impl<V:Interpolable> Clone for AnimatedValue<V> {
    fn clone(&self) -> Self {
        AnimatedValue {
            value: self.value.clone(),
            animation: self.animation.clone(),
        }
    }
}
// impl<T: Interpolable> Deref for AnimatedValue<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }

impl<'a, V:Interpolable> AnimatedValue<V> {
    pub fn new(value: V) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            animation: Arc::new(RwLock::new(None)),
        }
    }
    pub fn to(&mut self, value:V) -> Self {
        let v = self.value.read().unwrap();
        // *v = value;
        let new_animation = Animation {
            start: 0.0,
            duration: 0.0,
            from: *v,
            to: value,
            timing: Easing{x1:0.0, y1:0.0, x2:1.0, y2:1.0},
        };
        *self.animation.write().unwrap() = Some(new_animation);
        self.clone()
    }
    pub fn to_animated(mut self, value:V, maybe_transition:Option<Transition<Easing>>) -> Self {
        match maybe_transition {
            None => {
                self.to(value);
            },
            Some(transition) => {
                let new_animation = Animation {
                    start: transition.delay,
                    duration: transition.duration,
                    from: *self.value.read().unwrap(),
                    to: value,
                    timing: transition.timing,
                };
                *self.animation.write().unwrap() = Some(new_animation);
            },
        }
        self.clone()
    }
    pub fn value(&self) -> V {
        *self.value.read().unwrap()
    }
    pub fn update_at(&self, t: f64) {
        let animation = *self.animation.read().unwrap();
        match animation {
            Some(anim) => {
                *self.value.write().unwrap() = anim.value(t);
            },
            None => (),
        }
    }
}


impl fmt::Debug for Animation<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}->{:?} [{:?}])", self.from, self.to, self.duration)
    }
}

// deubg formatter for AnimatedValue
impl fmt::Debug for AnimatedValue<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnimatedValue {{ value: {:?}, animation: {:?} }}", self.value(), self.animation)
    }
}


// impl Property {
    // pub fn new(value: Arc<Mutex<dyn Animatable>>) -> Self {
    //     Self {
    //         value: value.clone(),
    //         // transition: None,
    //     }
    // }

    // setter method for target_value
    // pub fn to(&mut self, value:dyn Copy, when: Option<f64>) {
    //     let start_time = when.unwrap_or(0.0);
    //     let animation = match (self.transition) {
    //         Some(transition) => {
    //             let start = start_time + transition.delay;
    //             let duration = transition.duration;
    //             let timing = transition.timing;
    //             let animation = Animation {
    //                 start: 0.0,
    //                 duration,
    //                 from: *self.value,
    //                 to: value,
    //                 timing,
    //             };
    //             Some(animation)
    //         }
    //         None => None,
    //     };
    // }

    // pub fn transition(mut self, t: Transition<Easing>) -> Self {
    //     self.transition = Some(t);
    //     self
    // }
// }