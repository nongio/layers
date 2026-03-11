/// Errors returned by layer and engine operations.
#[derive(Debug, Clone)]
pub enum LayerError {
    /// The layer's scene-graph node has been removed or freed.
    ///
    /// This typically happens when a layer handle outlives the node it
    /// refers to — e.g. an animation callback fires after the layer
    /// was deleted.
    StaleNode,
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerError::StaleNode => write!(f, "layer node has been removed or freed"),
        }
    }
}

impl std::error::Error for LayerError {}
