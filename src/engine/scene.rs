//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

// use stretch::Stretch;

use crate::layers::layer::ModelLayer;

use super::{
    node::{RenderNode, SceneNode},
    storage::{TreeStorage, TreeStorageId, TreeStorageNode},
    // Engine,
    NodeRef,
};

pub struct Scene {
    pub nodes: TreeStorage<SceneNode>,
    pub root: RwLock<TreeStorageId>,
    // pub engine: RwLock<Option<Arc<Engine>>>,
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

    /// Add a new node to the scene by default append it to root
    fn insert_node(&self, node: &SceneNode) -> NodeRef {
        let id = self.nodes.insert(node.clone());

        let nodes = self.nodes.data();
        let mut nodes = nodes.write().unwrap();
        let root = self.root.read().unwrap();
        root.append(id, &mut nodes);

        NodeRef(id)
    }

    pub fn get_node(&self, id: TreeStorageId) -> Option<TreeStorageNode<SceneNode>> {
        self.nodes.get(id)
    }
    pub fn append_node_to(&self, children: NodeRef, parent: NodeRef) {
        let nodes = self.nodes.data();
        let mut nodes = nodes.write().unwrap();
        parent.append(*children, &mut nodes);
    }
    pub fn set_root<R: Into<Arc<dyn RenderNode>>>(&self, renderable: R) -> NodeRef {
        let renderable: Arc<dyn RenderNode> = renderable.into();
        let node = SceneNode::with_renderable(renderable.clone());
        let id = self.insert_node(&node);

        let mut root = self.root.write().unwrap();
        *root = id.0;

        id
    }
    pub fn add<R: Into<Arc<dyn RenderNode>>>(&self, renderable: R) -> NodeRef {
        let renderable: Arc<dyn RenderNode> = renderable.into();
        let node = SceneNode::with_renderable(renderable.clone());
        let id = self.insert_node(&node);

        id
    }
    // pub fn set_engine(&self, engine: Arc<Engine>) {
    //     self.engine.write().unwrap().replace(engine);
    // }
}

impl Default for Scene {
    fn default() -> Self {
        let nodes = TreeStorage::new();
        let root = ModelLayer::create();
        let node = SceneNode::with_renderable(root);
        let rootid = nodes.insert(node);

        Scene {
            nodes,
            root: RwLock::new(rootid),
            // engine: RwLock::new(None),
        }
    }
}
