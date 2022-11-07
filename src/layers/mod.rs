use crate::easing::{interpolate, Interpolable};

use crate::engine::{
    animations::*, command::*, node::*, Command, CommandWithTransition, WithTransition,
};

macro_rules! change_attr {
    ($variable_name:ident, $type:ty, $flag:expr) => {
        pub fn $variable_name(
            &self,
            value: $type,
            transition: Option<Transition<Easing>>,
        ) -> Arc<ModelChange<$type>> {
            let maybe_engine = self.engine.read().unwrap().clone();

            let change: Arc<ModelChange<$type>> = Arc::new(ModelChange {
                value_change: self.$variable_name.to(value.clone(), transition),
                flag: $flag,
            });
            if let Some((id, engine)) = maybe_engine {
                engine.add_change(id, change.clone());
            } else {
                self.$variable_name.set(value.clone());
            }
            change
        }
    };
}
pub(crate) use change_attr;

pub mod layer;
pub mod text;

impl<T: Interpolable + Sync> WithTransition for ModelChange<T> {
    fn transition(&self) -> Option<Transition<Easing>> {
        self.value_change.transition
    }
}

impl<T: Interpolable + Sync + Clone + Sized + 'static> Command for ModelChange<T> {
    fn execute(&self, progress: f64) -> RenderableFlags {
        let ModelChange {
            value_change, flag, ..
        } = &self;
        value_change.target.set(interpolate(
            value_change.from.clone(),
            value_change.to.clone(),
            progress,
        ));
        *flag
    }
    fn value_id(&self) -> usize {
        self.value_change.target.id
    }
}

impl<T: Interpolable + Sync + Send + Clone + Sized + 'static> CommandWithTransition
    for ModelChange<T>
{
}
