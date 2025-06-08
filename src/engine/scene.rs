//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

use crate::prelude::Point;
use indextree::{Arena, NodeId};
use std::sync::{Arc, RwLock};

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
    depth_groups_cache: RwLock<Option<(NodeId, Vec<(usize, Vec<NodeId>)>)>>,
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
            depth_groups_cache: RwLock::new(None),
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

    pub(crate) fn update_depth_recursive(
        nodes: &mut Arena<SceneNode>,
        node_id: NodeId,
        depth: usize,
    ) {
            let parent_depth = nodes.get(parent_id).map(|n| n.get().depth).unwrap_or(0);
        self.invalidate_depth_groups_cache();
            let parent_depth = nodes.get(parent_id).map(|n| n.get().depth).unwrap_or(0);
        self.invalidate_depth_groups_cache();
        self.invalidate_depth_groups_cache();
        self.invalidate_depth_groups_cache();

    pub(crate) fn invalidate_depth_groups_cache(&self) {
        *self.depth_groups_cache.write().unwrap() = None;
    }

    pub(crate) fn depth_groups(&self, root: NodeId) -> Vec<(usize, Vec<NodeId>)> {
        let mut cache = self.depth_groups_cache.write().unwrap();
        if let Some((cached_root, groups)) = cache.clone() {
            if cached_root == root {
                return groups;
            }
        }

        let groups = self.with_arena(|arena| {
            let mut depth_map: std::collections::HashMap<usize, Vec<NodeId>> =
                std::collections::HashMap::new();
            for edge in root.traverse(arena) {
                if let indextree::NodeEdge::End(id) = edge {
                    if let Some(node) = arena.get(id) {
                        let depth = node.get().depth;
                        depth_map.entry(depth).or_default().push(id);
                    }
                }
            }
            let mut groups: Vec<_> = depth_map.into_iter().collect();
            groups.sort_by(|a, b| b.0.cmp(&a.0));
            groups
        });
        *cache = Some((root, groups.clone()));
        groups
    }
        let children: Vec<_> = node_id.children(nodes).collect();
        for child in children {
            Self::update_depth_recursive(nodes, child, depth + 1);
        }
    }

    /// Append the child node to the parent node.
    ///
    /// The child node is first detached from the scene and then appended the new parent.
    /// After appending, the child node is marked as needing paint (NEEDS_PAINT).
    /// The new parent node is marked as needing layout (NEEDS_LAYOUT).
    pub(crate) fn append_node_to(&self, child: NodeRef, parent: NodeRef) {
        self.with_arena_mut(|nodes| {
            let child = *child;
            let parent_id = *parent;
            child.detach(nodes);
            parent_id.append(child, nodes);
            let parent_depth = nodes.get(parent_id).map(|n| n.get().depth).unwrap_or(0);
            Self::update_depth_recursive(nodes, child, parent_depth + 1);
            if let Some(scene_node) = nodes.get_mut(child) {
                let scene_node = scene_node.get_mut();
                scene_node.set_needs_repaint(true);
            }

            if let Some(new_parent_node) = nodes.get_mut(parent_id) {
                let new_parent_node = new_parent_node.get_mut();
                //FIXME if the node position is Absolute, we should not need to layout the parent
                new_parent_node.set_needs_layout(true);
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
            let parent_id = *parent;
            child.detach(nodes);
            parent_id.prepend(child, nodes);
            let parent_depth = nodes.get(parent_id).map(|n| n.get().depth).unwrap_or(0);
            Self::update_depth_recursive(nodes, child, parent_depth + 1);
            if let Some(scene_node) = nodes.get_mut(child) {
                let scene_node = scene_node.get_mut();
                scene_node.set_needs_repaint(true);
            }

            if let Some(new_parent_node) = nodes.get_mut(parent_id) {
                let new_parent_node = new_parent_node.get_mut();
                new_parent_node.set_needs_layout(true);
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

    pub(crate) fn remove_node(&self, id: impl Into<TreeStorageId>) {
        let id = id.into();

        self.nodes.remove_at_sync(&id);
    }

    pub(crate) fn is_node_removed(&self, id: impl Into<TreeStorageId>) -> bool {
        let id = id.into();

        let nodes = self.nodes.data();
        let nodes = nodes.read().unwrap();

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
    // pub async fn with_arena_async<T, F>(&self, f: F) -> Result<T, JoinError>
    // where
    //     T: Send + Sync + 'static,
    //     F: FnOnce(&Arena<SceneNode>) -> T + Send + 'static,
    // {
    //     let arena_guard = self.nodes.data();
    //     let handle = Handle::current();
    //     let join = tokio::task::spawn_blocking(move || {
    //         let arena_guard = handle.block_on(arena_guard.read());
    //         f(&arena_guard)
    //     });
    //     join.await
    // }

    pub fn with_arena<T: Send + Sync>(&self, f: impl FnOnce(&Arena<SceneNode>) -> T) -> T {
        let arena_guard = self.nodes.data();

        let arena = arena_guard.read().unwrap();
        f(&arena)
    }

    pub(crate) fn with_arena_mut<T>(&self, f: impl FnOnce(&mut Arena<SceneNode>) -> T) -> T {
        let arena_guard = self.nodes.data();
        let mut arena = arena_guard.write().unwrap();
        f(&mut arena)
    }
}
