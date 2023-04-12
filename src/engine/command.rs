//! # Scheduled changes for models
//! A Renderable defers changes to its properties, scheduling them into the Engine.
//! Changes are stored in a HashMap storage, that allows for id based read/write as well thread safe parallel iterations.
//! The changes can include an optional Transition description used by the engine to generate runnable animations.
//! Animations are separated from the changes to allow grouping of multiple changes in sync.
//! A Change when executed returns a set of bit flags to mark the affected Renderable for Layout, Paint or render.
//! On every update the Engine step forward the animations and applies the changes to the Renderables.

use std::sync::Arc;

use crate::easing::Interpolate;

/// Changes to models are scheduled to be applied at before the rendering steps
use super::{
    animations::{Easing, SyncValue, Transition},
    node::RenderableFlags,
    Command, CommandWithTransition, Engine, TransactionRef, WithTransition,
};

/// A representation of a change to a property, including an optional transition

#[derive(Clone, Debug)]
pub struct ValueChange<V: Sync> {
    pub from: V,
    pub to: V,
    pub target: SyncValue<V>,
    pub transition: Option<Transition<Easing>>,
}

/// A representation of a change to a model proprty, including what subsequent
/// rendering steps are required
#[derive(Clone, Debug)]
pub struct ModelChange<T: Sync> {
    pub value_change: ValueChange<T>,
    pub flag: RenderableFlags,
}

/// Objects implementing this trait expose a function `to(...)` that returns
/// a `ValueChange` object
pub trait AnimatableValue<V: Sync> {
    fn to(&self, to: V, transition: Option<Transition<Easing>>) -> ValueChange<V>;
}

#[derive(Clone)]
pub struct Transaction {
    pub(crate) engine: Arc<Engine>,
    pub id: TransactionRef,
}

impl Transaction {
    pub fn on_start<F: Fn(f32) + Send + Sync + 'static>(&self, handler: F) {
        self.engine.on_start(self.id, handler);
    }
    pub fn on_update<F: Fn(f32) + Send + Sync + 'static>(&self, handler: F) {
        self.engine.on_update(self.id, handler);
    }
}

pub struct NoopChange(usize);
impl NoopChange {
    pub fn new(id: usize) -> Self {
        Self(id)
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

impl WithTransition for NoopChange {
    fn transition(&self) -> Option<Transition<Easing>> {
        None
    }
}

impl CommandWithTransition for NoopChange {}

impl<T: Sync> WithTransition for ModelChange<T> {
    fn transition(&self) -> Option<Transition<Easing>> {
        self.value_change.transition
    }
}

impl<I: Interpolate + Sync + Clone + 'static> Command for ModelChange<I> {
    fn execute(&self, progress: f32) -> RenderableFlags {
        let ModelChange {
            value_change, flag, ..
        } = &self;

        value_change
            .target
            .set(value_change.from.interpolate(&value_change.to, progress));
        *flag
    }
    fn value_id(&self) -> usize {
        self.value_change.target.id
    }
}

impl<T: Interpolate + Sync + Send + Clone + Sized + 'static> CommandWithTransition
    for ModelChange<T>
{
}

macro_rules! change_model {
    ($variable_name:ident, $variable_type:ty, $flags:expr) => {
        paste::paste! {
            pub fn [<set_ $variable_name>](
                &self,
                value: impl Into<$variable_type>,
                transition: Option<Transition<Easing>>,
            )  -> &Self {
                let value:$variable_type = value.into();
                let flags = $flags;

                let change: Arc<ModelChange<$variable_type>> = Arc::new(ModelChange {
                    value_change: self.model.$variable_name.to(value.clone(), transition),
                    flag: flags,
                });
                // let mut tr = crate::engine::TransactionRef(0);
                let id:Option<NodeRef> = *self.id.read().unwrap();
                if let Some(id) = id {
                    self.engine.schedule_change(id, change.clone());
                } else {
                    self.model.$variable_name.set(value.clone());
                }
                // let transaction = Transaction {
                //     engine: self.engine.clone(),
                //     id: tr,
                // };
                &self
            }
            pub fn $variable_name(&self) -> $variable_type {
                self.model.$variable_name.value()
            }
        }
    };
}

pub(crate) use change_model;