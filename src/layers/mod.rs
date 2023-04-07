//! Internal models representing the Layers and their animatable properties.
use crate::easing::Interpolate;

use crate::engine::{
    animations::*, command::*, node::*, Command, CommandWithTransition, NodeRef, WithTransition,
};

macro_rules! change_model {
    ($variable_name:ident, $variable_type:ty, $flags:expr) => {
        paste::paste! {
            pub fn [<set_ $variable_name>](
                &self,
                value: impl Into<$variable_type>,
                transition: Option<Transition<Easing>>,
            )  -> Transaction {
                let value:$variable_type = value.into();
                let flags = $flags;

                let change: Arc<ModelChange<$variable_type>> = Arc::new(ModelChange {
                    value_change: self.model.$variable_name.to(value.clone(), transition),
                    flag: flags,
                });
                let mut tr = crate::engine::TransactionRef(0);
                let id:Option<NodeRef> = *self.id.read().unwrap();
                if let Some(id) = id {
                    tr = self.engine.schedule_change(id, change.clone());
                } else {
                    self.model.$variable_name.set(value.clone());
                }
                let transaction = Transaction {
                    engine: self.engine.clone(),
                    id: tr,
                };
                transaction
            }
            pub fn $variable_name(&self) -> $variable_type {
                self.model.$variable_name.value()
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

pub(crate) use change_model;
use taffy::prelude::Node;

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
            Layers::Layer(layer) => *layer.id.read().unwrap(),
            Layers::TextLayer(layer) => *layer.id.read().unwrap(),
        }
    }
    pub fn set_id(&self, id: NodeRef) {
        match self {
            Layers::Layer(layer) => layer.set_id(id),
            Layers::TextLayer(layer) => layer.set_id(id),
        }
    }
    pub fn layout_node(&self) -> Node {
        match self {
            Layers::Layer(layer) => layer.layout,
            Layers::TextLayer(layer) => layer.layout,
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
