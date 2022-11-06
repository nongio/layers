use std::sync::{Arc, RwLock};

use super::{
    node::{RenderNode, SceneNode},
    storage::{TreeStorage, TreeStorageId, TreeStorageNode},
    Engine,
};

pub struct Scene {
    pub nodes: TreeStorage<SceneNode>,
    pub root: RwLock<Option<TreeStorageId>>,
    pub engine: RwLock<Option<Arc<Engine>>>,
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
    fn insert_node(&self, node: &SceneNode) -> TreeStorageId {
        let id = self.nodes.insert(node.clone());

        let nodes = self.nodes.data();
        let mut nodes = nodes.write().unwrap();
        let mut root = self.root.write().unwrap();
        if let Some(root_id) = *root {
            root_id.append(id, &mut nodes)
        } else {
            *root = Some(id);
        }
        id
    }

    pub fn get_node(&self, id: TreeStorageId) -> Option<TreeStorageNode<SceneNode>> {
        self.nodes.get(id)
    }

    pub fn add<R: Into<Arc<dyn RenderNode>>>(&self, renderable: R) -> TreeStorageId {
        let renderable: Arc<dyn RenderNode> = renderable.into();
        let node = SceneNode::with_renderable(renderable.clone());
        let id = self.insert_node(&node);

        if let Some(engine) = self.engine.read().unwrap().clone() {
            renderable.set_engine(engine, id);
        }
        id
    }
    pub fn set_engine(&self, engine: Arc<Engine>) {
        self.engine.write().unwrap().replace(engine);
    }
}

impl Default for Scene {
    fn default() -> Self {
        let nodes = TreeStorage::new();

        Scene {
            nodes,
            root: RwLock::new(None),
            engine: RwLock::new(None),
        }
    }
}
