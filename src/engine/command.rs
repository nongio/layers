// use crate::easing::Interpolable;

use super::{
    animations::{Easing, SyncValue, Transition},
    node::RenderableFlags,
};

#[derive(Clone, Debug)]
pub struct ValueChange<V: Sync> {
    pub from: V,
    pub to: V,
    pub target: SyncValue<V>,
    pub transition: Option<Transition<Easing>>,
}

#[derive(Clone, Debug)]
pub struct ModelChange<T: Sync> {
    pub value_change: ValueChange<T>,
    pub flag: RenderableFlags,
}

pub trait AnimatableValue<V: Sync> {
    fn to(&self, to: V, transition: Option<Transition<Easing>>) -> ValueChange<V>;
}
