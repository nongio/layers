use crate::easing::Interpolable;

use super::{
    animations::{Easing, SyncValue, Transition},
    node::NodeFlags,
};

#[derive(Clone, Debug)]
pub struct ValueChange<V: Interpolable + Sync> {
    pub from: V,
    pub to: V,
    pub target: SyncValue<V>,
    pub transition: Option<Transition<Easing>>,
}

#[derive(Clone, Debug)]
pub struct ModelChange<T: Interpolable + Sync> {
    pub value_change: ValueChange<T>,
    pub flag: NodeFlags,
}

pub trait AnimatableValue<V: Interpolable + Sync> {
    fn to(&self, to: V, transition: Option<Transition<Easing>>) -> ValueChange<V>;
}
