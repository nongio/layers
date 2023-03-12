// use std::sync::Arc;

use crate::easing::Interpolate;

use crate::engine::{
    animations::*, command::*, node::*, Command, CommandWithTransition, NodeRef, WithTransition,
};

macro_rules! change_attr {
    ($variable_name:ident, $variable_type:ty, $flags:expr) => {
        paste::paste! {
            pub fn [<set_ $variable_name>](
                &self,
                value: impl Into<$variable_type>,
                transition: Option<Transition<Easing>>,
            )  -> Arc<ModelChange<$variable_type>> {
                let value:$variable_type = value.into();
                let flags = $flags;

                let change: Arc<ModelChange<$variable_type>> = Arc::new(ModelChange {
                    value_change: self.model.$variable_name.to(value.clone(), transition),
                    flag: flags,
                });
                let id:Option<NodeRef> = *self.id.read().unwrap();
                if let Some(id) = id {
                    self.engine.add_change(id, change.clone());
                } else {
                    self.model.$variable_name.set(value.clone());
                }
                change
            }
        }
    };
}

impl<T: Sync> WithTransition for ModelChange<T> {
    fn transition(&self) -> Option<Transition<Easing>> {
        self.value_change.transition
    }
}

impl<I: Interpolate + Sync + Clone + 'static> Command for ModelChange<I> {
    fn execute(&self, progress: f64) -> RenderableFlags {
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

pub(crate) use change_attr;

use self::layer::Layer;
use self::text::TextLayer;

pub mod layer;
pub mod text;

#[derive(Clone)]
pub enum Layers {
    Layer(layer::Layer),
    TextLayer(text::TextLayer),
}
impl Layers {
    pub fn id(&self) -> Option<NodeRef> {
        match self {
            Layers::Layer(layer) => layer.id.read().unwrap().clone(),
            Layers::TextLayer(layer) => layer.id.read().unwrap().clone(),
        }
    }
    pub fn set_id(&self, id: NodeRef) {
        match self {
            Layers::Layer(layer) => layer.set_id(id),
            Layers::TextLayer(layer) => layer.set_id(id),
        }
    }
}


impl From<Layer> for Layers {
    fn from(layer: Layer) -> Self {
        Layers::Layer(layer)
    }
}
impl From<TextLayer> for Layers {
    fn from(layer: TextLayer) -> Self {
        Layers::TextLayer(layer)
    }
}
