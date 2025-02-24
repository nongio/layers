//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

use indextree::Arena;
use std::sync::{Arc, RwLock};
use tokio::{runtime::Handle, task::JoinError};

use crate::prelude::Point;

use super::{
    node::{RenderableFlags, SceneNode},
    storage::{TreeStorage, TreeStorageId},
    Engine, NodeRef,
};

impl Engine {
    pub fn set_node_flags(&self, node: NodeRef, flags: RenderableFlags) {
        self.scene.with_arena_mut(|arena| {
            let node = arena.get_mut(node.0).unwrap();
            let scene_node = node.get_mut();
            scene_node.insert_flags(flags);
        });
    }
}
pub struct Scene {
    pub(crate) nodes: TreeStorage<SceneNode>,
    pub size: RwLock<Point>,
}

impl Scene {
    fn new(width: f32, height: f32) -> Self {
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
    /// The new parent node is marked as needing layout (NEEDS_LAYOUT).
    pub(crate) fn append_node_to(&self, child: NodeRef, parent: NodeRef) {
        self.with_arena_mut(|nodes| {
            let child = *child;
            child.detach(nodes);
            parent.append(child, nodes);
            if let Some(scene_node) = nodes.get_mut(child) {
                let scene_node = scene_node.get_mut();
                scene_node.set_need_repaint(true);
            }

            let parent = *parent;
            if let Some(new_parent_node) = nodes.get_mut(parent) {
                let new_parent_node = new_parent_node.get_mut();
                //FIXME if the node position is Absolute, we should not need to layout the parent
                new_parent_node.set_need_layout(true);
            }
        });
    }
    /// Prepend the child node to the parent node.
    /// The child node is first detached from the scene and then prepended to the new parent.
    /// After prepending, the child node is marked as needing paint (NEEDS_PAINT).
    /// The parent node is marked as needing layout (NEEDS_LAYOUT).
    pub(crate) fn prepend_node_to(&self, child: NodeRef, parent: NodeRef) {
        self.with_arena_mut(|nodes| {
            let child = *child;
            child.detach(nodes);
            parent.prepend(child, nodes);
            if let Some(scene_node) = nodes.get_mut(child) {
                let scene_node = scene_node.get_mut();
                scene_node.set_need_repaint(true);
            }

            let parent = *parent;
            if let Some(new_parent_node) = nodes.get_mut(parent) {
                let new_parent_node = new_parent_node.get_mut();
                new_parent_node.set_need_layout(true);
            }
        });
    }
    /// Add a new node to the scene
    pub(crate) fn insert_node(&self, node: SceneNode, parent: Option<NodeRef>) -> NodeRef {
        let id = self.nodes.insert_sync(node);
        if let Some(parent) = parent {
            self.append_node_to(NodeRef(id), parent);
        }
        NodeRef(id)
    }

    // pub async fn get_node(
    //     &self,
    //     id: impl Into<TreeStorageId>,
    // ) -> Option<TreeStorageNode<SceneNode>> {
    //     let id = id.into();
    //     let scene_node = self.nodes.get(id).await;
    //     scene_node.filter(|node| !node.is_removed())
    // }

    // pub fn get_node_sync(
    //     &self,
    //     id: impl Into<TreeStorageId>,
    // ) -> Option<&indextree::Node<SceneNode>> {
    //     let id = id.into();

    //     self.with_arena(|arena| {
    //         let scene_node = arena.get(id);
    //         scene_node.filter(|node| !node.is_removed())
    //     })
    // }

    pub fn new_node(&self) -> NodeRef {
        let node = SceneNode::new();
        self.insert_node(node, None)
    }
    pub fn append_node(&self, parent: Option<NodeRef>) -> NodeRef {
        let node = SceneNode::new();
        self.insert_node(node, parent)
    }
    pub(crate) fn remove_node(&self, id: impl Into<TreeStorageId>) {
        let id = id.into();

        self.nodes.remove_at_sync(&id);
    }

    pub(crate) fn is_node_removed(&self, id: impl Into<TreeStorageId>) -> bool {
        let id = id.into();

        let nodes = self.nodes.data();
        let nodes = nodes.blocking_read();

        nodes
            .get(id)
            .map(|node| {
                if node.is_removed() {
                    true
                } else {
                    let node = node.get();
                    node.is_deleted()
                }
            })
            .unwrap_or(true)
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
            f(&arena_guard)
        });
        join.await
    }

    pub fn with_arena<T: Send + Sync>(&self, f: impl FnOnce(&Arena<SceneNode>) -> T) -> T {
        let arena_guard = self.nodes.data();

        let arena = arena_guard.blocking_read();
        f(&arena)
    }

    pub(crate) fn with_arena_mut<T>(&self, f: impl FnOnce(&mut Arena<SceneNode>) -> T) -> T {
        let arena_guard = self.nodes.data();
        let mut arena = arena_guard.blocking_write();
        f(&mut arena)
    }
}
