use crate::easing::{interpolate, Interpolable};

use crate::engine::{
    animations::*, command::*, node::*, Command, CommandWithTransition, WithTransition,
};

macro_rules! change_attr {
    ($variable_name:ident, $variable_type:ty, $flags:expr) => {
        paste::paste! {
            pub fn [<set_ $variable_name>](
                &self,
                value: $variable_type,
                transition: Option<Transition<Easing>>,
            ) -> Arc<ModelChange<$variable_type>> {
                let maybe_engine = self.engine.read().unwrap().clone();

                let change: Arc<ModelChange<$variable_type>> = Arc::new(ModelChange {
                    value_change: self.$variable_name.to(value.clone(), transition),
                    flag: $flags,
                });
                if let Some((id, engine)) = maybe_engine {
                    engine.add_change(id, change.clone());
                } else {
                    self.$variable_name.set(value.clone());
                }
                change
            }
        }
    };
}

// macro_rules! api_change_attr {
//     ($base_type_export:ty, $variable_name:ident, $variable_type:ty) => {
//         paste::paste! {
//             use engine::animations::{Transition, Easing};
//             #[no_mangle]
//             pub extern "C" fn [<layer_set_ $variable_name>](
//                     obj: *const $base_type_export,
//                     value: $variable_type,
//                     t: Transition<Easing>,
//                 ) {
//                         let obj = unsafe { &*obj };
//                         obj.[<set_ $variable_name>](value, Some(t));
//             }
//         }
//     };
// }
// pub(crate) use api_change_attr;
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
