//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

use indextree::Arena;
use std::sync::{Arc, RwLock};
use taffy::prelude::NodeId as TaffyNode;
use tokio::{runtime::Handle, task::JoinError};

use crate::prelude::{Layer, Point};

use super::{
    node::{RenderableFlags, SceneNode},
    storage::{TreeStorage, TreeStorageId, TreeStorageNode},
    NodeRef,
};
pub struct Scene {
    pub(crate) nodes: TreeStorage<SceneNode>,
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
        self.with_arena_mut(|nodes| {
            let child = *child;
            child.detach(nodes);
            parent.append(child, nodes);
            let scene_node = nodes.get(child).unwrap().get();
            scene_node.insert_flags(RenderableFlags::NEEDS_PAINT);
        });
    }
    /// Add a new node to the scene
    fn insert_node(&self, node: &SceneNode, parent: Option<NodeRef>) -> NodeRef {
        let handle = Handle::current();
        let id = tokio::task::block_in_place(|| handle.block_on(self.nodes.insert(node.clone())));
        if let Some(parent) = parent {
            self.append_node_to(NodeRef(id), parent);
        }
        NodeRef(id)
    }

    pub async fn get_node(
        &self,
        id: impl Into<TreeStorageId>,
    ) -> Option<TreeStorageNode<SceneNode>> {
        let id = id.into();
        let scene_node = self.nodes.get(id).await;
        scene_node.filter(|node| !node.is_removed())
    }

    pub fn get_node_sync(
        &self,
        id: impl Into<TreeStorageId>,
    ) -> Option<TreeStorageNode<SceneNode>> {
        let id = id.into();
        let handle = Handle::current();
        let _ = handle.enter();
        let scene_node = tokio::task::block_in_place(|| handle.block_on(self.nodes.get(id)));
        scene_node.filter(|node| !node.is_removed())
    }

    pub fn add<R: Into<Layer>>(&self, renderable: R, layout: TaffyNode) -> NodeRef {
        let renderable: Layer = renderable.into();
        let node = SceneNode::with_renderable_and_layout(renderable, layout);
        self.insert_node(&node, None)
    }
    pub fn append<R: Into<Layer>>(
        &self,
        parent: Option<NodeRef>,
        renderable: R,
        layout: TaffyNode,
    ) -> NodeRef {
        let renderable: Layer = renderable.into();
        let node = SceneNode::with_renderable_and_layout(renderable, layout);
        self.insert_node(&node, parent)
    }
    pub(crate) fn remove(&self, id: impl Into<TreeStorageId>) {
        let id = id.into();
        let handle = Handle::current();
        tokio::task::block_in_place(|| handle.block_on(self.nodes.remove_at(&id)));
    }

    pub async fn with_arena_async<T, F>(&self, f: F) -> Result<T, JoinError>
    where
        T: Send + Sync + 'static,
        F: FnOnce(&Arena<SceneNode>) -> T + Send + 'static,
    {
        let arena_guard = self.nodes.data();
        let handle = Handle::current();
        let join = tokio::task::spawn_blocking(move || {
            let arena_guard = handle.block_on(arena_guard.read());
            f(&*arena_guard)
        });
        join.await
    }

    pub fn with_arena<T: Send + Sync>(&self, f: impl FnOnce(&Arena<SceneNode>) -> T) -> T {
        let arena_guard = self.nodes.data();
        let handle = Handle::current();
        tokio::task::block_in_place(|| {
            let arena_guard = handle.block_on(arena_guard.read());
            f(&*arena_guard)
        })
    }

    pub(crate) fn with_arena_mut<T>(&self, f: impl FnOnce(&mut Arena<SceneNode>) -> T) -> T {
        let arena_guard = self.nodes.data();
        let handle = Handle::current();
        tokio::task::block_in_place(|| {
            let mut arena_guard = handle.block_on(arena_guard.write());
            f(&mut *arena_guard)
        })
    }
}
