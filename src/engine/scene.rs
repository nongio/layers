//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

use std::sync::Arc;
use taffy::prelude::Node;

use crate::layers::Layers;

use super::{
    node::SceneNode,
    storage::{TreeStorage, TreeStorageId, TreeStorageNode},
    // Engine,
    NodeRef,
};
pub struct Scene {
    pub nodes: TreeStorage<SceneNode>,
}

impl Scene {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub(crate) fn create() -> Arc<Scene> {
        Arc::new(Self::new())
    }

    pub fn append_node_to(&self, children: NodeRef, parent: NodeRef) {
        let nodes = self.nodes.data();
        let mut nodes = nodes.write().unwrap();
        parent.append(*children, &mut nodes);
    }
    /// Add a new node to the scene
    fn insert_node(&self, node: &SceneNode, parent: Option<NodeRef>) -> NodeRef {
        let id = self.nodes.insert(node.clone());
        if let Some(parent) = parent {
            self.append_node_to(NodeRef(id), parent);
        }
        NodeRef(id)
    }

    pub fn get_node(&self, id: impl Into<TreeStorageId>) -> Option<TreeStorageNode<SceneNode>> {
        let id = id.into();
        self.nodes.get(id)
    }

    pub fn add<R: Into<Arc<Layers>>>(&self, renderable: R, layout: Node) -> NodeRef {
        let renderable: Arc<Layers> = renderable.into();
        let node = SceneNode::with_renderable_and_layout(renderable, layout);
        self.insert_node(&node, None)
    }
    pub fn append<R: Into<Arc<Layers>>>(
        &self,
        parent: Option<NodeRef>,
        renderable: R,
        layout: Node,
    ) -> NodeRef {
        let renderable: Arc<Layers> = renderable.into();
        let node = SceneNode::with_renderable_and_layout(renderable, layout);
        self.insert_node(&node, parent)
    }
    pub fn remove(&self, id: impl Into<TreeStorageId>) {
        let id = id.into();
        self.nodes.remove_at(&id);
    }
}

impl Default for Scene {
    fn default() -> Self {
        let nodes = TreeStorage::new();

        Scene { nodes }
    }
}
