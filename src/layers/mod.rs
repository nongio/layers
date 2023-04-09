//! Internal models representing the Layers and their animatable properties.

use taffy::prelude::Node;

use crate::engine::NodeRef;

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
