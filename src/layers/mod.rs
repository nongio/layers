use crate::{
    easing::{interpolate, Interpolable},
    engine::{
        animations::{Easing, Transition},
        command::ModelChange,
        node::NodeFlags,
        Command, CommandWithTransition, WithTransition,
    },
};

use self::layer::ModelLayer;

pub mod layer;
pub mod text;

pub enum Nodes {
    Layer(ModelLayer),
}

impl<T: Interpolable + Sync> WithTransition for ModelChange<T> {
    fn transition(&self) -> Option<Transition<Easing>> {
        self.value_change.transition
    }
}

impl<T: Interpolable + Sync + Clone + Sized + 'static> Command for ModelChange<T> {
    fn execute(&self, progress: f64) -> NodeFlags {
        let ModelChange {
            value_change, flag, ..
        } = &self;
        *value_change.target.value.write().unwrap() =
            interpolate(value_change.from.clone(), value_change.to.clone(), progress);

        *flag
    }
}

impl<T: Interpolable + Sync + Send + Clone + Sized + 'static> CommandWithTransition
    for ModelChange<T>
{
}
