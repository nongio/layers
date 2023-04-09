use std::sync::Arc;

use crate::engine::node::RenderNode;

use super::{model::ModelLayer, Layer};

impl From<Layer> for Arc<dyn RenderNode> {
    fn from(layer: Layer) -> Self {
        layer.model
    }
}
impl RenderNode for ModelLayer {}

// Convertion helpers

impl From<ModelLayer> for Arc<dyn RenderNode> {
    fn from(model: ModelLayer) -> Self {
        Arc::new(model)
    }
}
