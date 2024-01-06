//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

use std::sync::Arc;
use taffy::prelude::Node;

use crate::prelude::{Layer, Point};

use super::{
    node::SceneNode,
    storage::{TreeStorage, TreeStorageId, TreeStorageNode},
    // Engine,
    NodeRef,
};
pub struct Scene {
    pub nodes: TreeStorage<SceneNode>,
    pub size: Point,
}

impl Scene {
    pub fn new(width: f32, height: f32) -> Self {
        let nodes = TreeStorage::new();
        Self {
            nodes,
            size: Point {
                x: width,
                y: height,
            },
        }
    }
    pub(crate) fn create(width: f32, height: f32) -> Arc<Scene> {
        Arc::new(Self::new(width, height))
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

    pub fn add<R: Into<Layer>>(&self, renderable: R, layout: Node) -> NodeRef {
        let renderable: Layer = renderable.into();
        let node = SceneNode::with_renderable_and_layout(renderable, layout);
        self.insert_node(&node, None)
    }
    pub fn append<R: Into<Layer>>(
        &self,
        parent: Option<NodeRef>,
        renderable: R,
        layout: Node,
    ) -> NodeRef {
        let renderable: Layer = renderable.into();
        let node = SceneNode::with_renderable_and_layout(renderable, layout);
        self.insert_node(&node, parent)
    }
    pub fn remove(&self, id: impl Into<TreeStorageId>) {
        let id = id.into();
        self.nodes.remove_at(&id);
    }
}
