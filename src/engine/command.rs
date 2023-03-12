//! # Scheduled changes for models
//! A Renderable defers changes to its properties, scheduling them into the Engine.
//! Changes are stored in a HashMap storage, that allows for id based read/write as well thread safe parallel iterations.
//! The changes can includes an optional Transition description used by the engine to generate runnable animations. Animations are separated from the changes to allow grouping of multiple changes in sync.
//! A Change when executed returns a set of bit flags to mark the affected Renderable for Layout, Paint or render.
//! On every update the Engine step forward the animations and applies the changes to the Renderables.

/// Changes to models are scheduled to be applied at before the rendering steps
use super::{
    animations::{Easing, SyncValue, Transition},
    node::RenderableFlags,
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
