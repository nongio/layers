//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

use std::sync::{Arc, RwLock};
use taffy::prelude::Node;

use crate::prelude::{Layer, Point};

use super::{
    node::{RenderableFlags, SceneNode},
    storage::{TreeStorage, TreeStorageId, TreeStorageNode},
    // Engine,
    NodeRef,
};
pub struct Scene {
    pub nodes: TreeStorage<SceneNode>,
    pub size: RwLock<Point>,
}

impl Scene {
    pub fn new(width: f32, height: f32) -> Self {
        let nodes = TreeStorage::new();
        Self {
            nodes,
            size: RwLock::new(Point {
                x: width,
                y: height,
            }),
        }
    }
    pub fn set_size(&self, width: f32, height: f32) {
        let mut size = self.size.write().unwrap();
        *size = Point {
            x: width,
            y: height,
        };
    }
    pub(crate) fn create(width: f32, height: f32) -> Arc<Scene> {
        Arc::new(Self::new(width, height))
    }

    /// Append the child node to the parent node.
    ///
    /// The child node is first detached from the scene and then appended the new parent.
    /// After appending, the child node is marked as needing paint (NEEDS_PAINT).
    pub(crate) fn append_node_to(&self, child: NodeRef, parent: NodeRef) {
        let nodes = self.nodes.data();
        let mut nodes = nodes.write().unwrap();

        let child = *child;
        child.detach(&mut nodes);
        parent.append(child, &mut nodes);
        let scene_node = nodes.get(child).unwrap().get();
        scene_node.insert_flags(RenderableFlags::NEEDS_PAINT);
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
    pub(crate) fn remove(&self, id: impl Into<TreeStorageId>) {
        let id = id.into();
        self.nodes.remove_at(&id);
    }
}
