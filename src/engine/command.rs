//! # Scheduled changes for models
//! A Node defers changes to its properties, scheduling them into the Engine.
//! Changes are stored in a HashMap storage, that allows for id based read/write as well thread safe parallel iterations.
//! The changes can include an optional Transition description used by the engine to generate runnable animations.
//! Animations are separated from the changes to allow grouping of multiple changes in sync.
//! A Change when executed returns a set of bit flags to mark the affected Node for Layout, Paint or render.
//! On every update the Engine step forward the animations and applies the changes to the Nodes.

use super::{
    animation::Transition, node::RenderableFlags, AnimationRef, Command, Engine, SyncCommand,
    TransactionCallback, TransactionRef,
};
use crate::easing::Interpolate;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

static ATTRIBUTE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct Attribute<V: Sync + std::fmt::Debug> {
    pub id: usize,
    value: Arc<RwLock<V>>,
}

impl<V: Sync + Clone + std::fmt::Debug> Attribute<V> {
    pub fn new(value: V) -> Attribute<V> {
        let value = Arc::new(RwLock::new(value));
        Self {
            id: ATTRIBUTE_COUNTER.fetch_add(1, Ordering::SeqCst),
            value,
        }
    }

    pub fn value(&self) -> V {
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, value: V) {
        *self.value.write().unwrap() = value;
    }

    pub fn to(&self, to: V, transition: Option<Transition>) -> AttributeChange<V> {
        AttributeChange {
            from: self.value(),
            to,
            target: self.clone(),
            transition,
        }
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn from_to(&self, from: V, to: V, transition: Option<Transition>) -> AttributeChange<V> {
        AttributeChange {
            from,
            to,
            target: self.clone(),
            transition,
        }
    }
}

/// A representation of a change to a property, including an optional transition

#[derive(Clone, Debug)]
pub struct AttributeChange<V: Sync + std::fmt::Debug> {
    pub from: V,
    pub to: V,
    pub target: Attribute<V>,
    pub transition: Option<Transition>,
}

/// Representation of a change to a model property, including what subsequent
/// rendering steps are required
#[derive(Clone, Debug)]
pub struct ModelChange<T: Sync + std::fmt::Debug> {
    pub value_change: AttributeChange<T>,
    pub flag: RenderableFlags,
}

#[derive(Clone)]
pub struct Transaction {
    pub(crate) engine: Arc<Engine>,
    pub id: TransactionRef,
}

impl Transaction {
    pub fn on_start<F: Into<TransactionCallback>>(&self, handler: F) {
        self.engine.on_start(self.id, handler, true);
    }
    pub fn on_update<F: Into<TransactionCallback>>(&self, handler: F) {
        self.engine.on_update(self.id, handler, true);
    }
    pub fn on_finish<F: Into<TransactionCallback>>(&self, handler: F) {
        self.engine.on_finish(self.id, handler, true);
    }
}

#[derive(Debug)]
pub struct NoopChange(usize);
impl NoopChange {
    pub fn new(value_id: usize) -> Self {
        Self(value_id)
    }
}
impl Command for NoopChange {
    fn execute(&self, _progress: f32) -> RenderableFlags {
        RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT
    }
    fn value_id(&self) -> usize {
        self.0
    }
}
impl SyncCommand for NoopChange {}

impl From<NoopChange> for Option<AnimationRef> {
    fn from(_: NoopChange) -> Self {
        None
    }
}

impl<I: Interpolate + PartialEq + std::fmt::Debug + Send + Sync + Clone> Command
    for ModelChange<I>
{
    fn execute(&self, progress: f32) -> RenderableFlags {
        let ModelChange {
            value_change, flag, ..
        } = &self;
        let value_to = value_change.from.interpolate(&value_change.to, progress);
        if value_change.from != value_to {
            value_change.target.set(value_to);
            *flag
        } else {
            RenderableFlags::empty()
        }
    }
    fn value_id(&self) -> usize {
        self.value_change.target.id
    }
}
impl<I: Interpolate + Sync + PartialEq + Send + Clone + std::fmt::Debug + 'static> SyncCommand
    for ModelChange<I>
{
}

macro_rules! change_model {
    ($variable_name:ident, $variable_type:ty, $flags:expr) => {
        paste::paste! {
            pub fn [<set_ $variable_name>](
                &self,
                value: impl Into<$variable_type>,
                transition: impl Into<Option<Transition>>,
            )  -> TransactionRef {
                let transition = transition.into();
                let value:$variable_type = value.into();

                let flags = $flags;

                let value_id = self.model.$variable_name.id;
                let change = Arc::new(ModelChange {
                    value_change: self.model.$variable_name.to(value.clone(), transition),
                    flag: flags,
                });
                // }

                let animation = transition.map(|t| {
                    // if there is a transition
                    let merged_timing = if let TimingFunction::Spring(mut spring) = t.timing {
                        // and the transition is a spring, check if there is already a running transaction
                        let velocity = self.engine.get_transaction_for_value(value_id)
                            .map(|running_transaction| {
                                if let Some(animation_id) = running_transaction.animation_id {
                                    let animation_state = self.engine.get_animation(animation_id).unwrap();
                                    let animation = animation_state.animation;
                                    match animation.timing {
                                        TimingFunction::Spring(s) => {
                                            let (_current_position, current_velocity) =
                                                s.update_pos_vel_at(animation_state.time);
                                            current_velocity
                                        }
                                        _ => 0.0,
                                    }
                                } else {
                                    0.0
                                }
                            }).unwrap_or(0.0);
                        spring.initial_velocity = velocity;
                        TimingFunction::Spring(spring)
                    } else {
                        t.timing
                    };
                    self.engine.add_animation(Animation {
                        timing: merged_timing,
                        start: t.delay + self.engine.now(),
                    }, true)
                });

                self.engine.schedule_change(self.id, change, animation)
            }
            pub fn $variable_name(&self) -> $variable_type {
                self.model.$variable_name.value()
            }
            pub fn [< $variable_name _value_id>](&self) -> usize {
                    self.model.$variable_name.id
            }
            pub fn [<change_ $variable_name>](&self, value: impl Into<$variable_type>,) -> AnimatedNodeChange {
                let flags = $flags;
                let value = value.into();
                let change = Arc::new(ModelChange {
                    value_change: self.model.$variable_name.to(value, None),
                    flag: flags,
                });
                AnimatedNodeChange {
                    animation_id: None,
                    change,
                    node_id: self.id,
                }
            }
        }
    };
}

pub(crate) use change_model;
