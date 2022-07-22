use crate::{
    easing::Interpolable,
    ecs::animations::{Easing, Transition, ValueChange},
};

pub mod layer;
pub mod text;

#[derive(Clone, Debug)]
pub struct ModelChange<T: Interpolable + Sync> {
    pub id: usize,
    pub value_change: ValueChange<T>,
    pub need_repaint: bool,
}

pub trait ChangeWithTransition {
    fn transition(&self) -> Option<Transition<Easing>>;
    fn value_change_id(&self) -> usize;
}
impl<T: Interpolable + Sync> ChangeWithTransition for ModelChange<T> {
    fn transition(&self) -> Option<Transition<Easing>> {
        self.value_change.transition
    }
    fn value_change_id(&self) -> usize {
        self.value_change.target.id
    }
}
