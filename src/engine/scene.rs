//! The Scene is the data structure to represent the tree of nodes to be rendered.
//! It enables the traversing and manipulation of the nodes.
//!
//! The scene is a tree of renderable nodes (implementing the `Renderable` trait).
//! The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

use crate::{
    engine::storage::{FlatStorage, FlatStorageData},
    layers::layer::render_layer::RenderLayer,
    prelude::Point,
};
use indextree::Arena;
use serde::Serialize;
use std::sync::{Arc, RwLock};

use super::{
    node::{RenderableFlags, SceneNode, SceneNodeRenderable},
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
    pub(crate) renderables: FlatStorage<SceneNodeRenderable>,

    pub size: RwLock<Point>,
}

impl Scene {
    fn new(width: f32, height: f32) -> Self {
        let nodes = TreeStorage::new();
        let renderables = FlatStorage::new();
        Self {
            nodes,
            renderables,
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
            if parent.is_removed(nodes) {
                //     //
                return;
            }
            let child_id = *child;
            child_id.detach(nodes);
            parent.append(child_id, nodes);
            if let Some(scene_node) = nodes.get_mut(child_id) {
                let scene_node = scene_node.get_mut();
                scene_node.set_needs_repaint(true);
            }
            let parent = *parent;
            if let Some(new_parent_node) = nodes.get_mut(parent) {
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
            child.detach(nodes);
            parent.prepend(child, nodes);
            if let Some(scene_node) = nodes.get_mut(child) {
                let scene_node = scene_node.get_mut();
                scene_node.set_needs_repaint(true);
            }
            let parent = *parent;
            if let Some(new_parent_node) = nodes.get_mut(parent) {
                let new_parent_node = new_parent_node.get_mut();
                new_parent_node.set_needs_layout(true);
            }
        });
    }
    /// Add a new node to the scene
    pub(crate) fn insert_node(&self, node: SceneNode, parent: Option<NodeRef>) -> NodeRef {
        let id = self.nodes.insert_sync(node);
        let renderable = SceneNodeRenderable::new();
        self.renderables.insert_with_id(renderable, id.into());

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
        self.renderables.remove_at(&id.into());
    }

    pub(crate) fn is_node_removed(&self, id: impl Into<TreeStorageId>) -> bool {
        let id = id.into();

        let nodes = self.nodes.data();
        let nodes = nodes.read().unwrap();

        let node_removed = nodes
            .get(id)
            .map(|node| {
                if node.is_removed() {
                    true
                } else {
                    let scene_node = node.get();
                    scene_node.is_deleted()
                }
            })
            .unwrap_or(true);

        node_removed
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

    /// Try to run `f` with a read guard; returns `None` if the lock is contended.
    pub fn try_with_arena<T: Send + Sync>(
        &self,
        f: impl FnOnce(&Arena<SceneNode>) -> T,
    ) -> Option<T> {
        let arena_guard = self.nodes.data();
        let arena = arena_guard.try_read().ok()?;
        Some(f(&arena))
    }

    pub(crate) fn with_arena_mut<T>(&self, f: impl FnOnce(&mut Arena<SceneNode>) -> T) -> T {
        let arena_guard = self.nodes.data();
        let mut arena = arena_guard.write().unwrap();
        f(&mut arena)
    }

    /// Try to run `f` with a write guard; returns `None` if the lock is contended.
    pub fn try_with_arena_mut<T>(&self, f: impl FnOnce(&mut Arena<SceneNode>) -> T) -> Option<T> {
        let arena_guard = self.nodes.data();
        let mut arena = arena_guard.try_write().ok()?;
        Some(f(&mut arena))
    }

    pub fn with_renderable_arena<T: Send + Sync>(
        &self,
        f: impl FnOnce(&FlatStorageData<SceneNodeRenderable>) -> T,
    ) -> T {
        self.renderables.with_data(|arena| f(arena))
    }

    /// Try to run `f` with a read guard on the renderables arena.
    pub fn try_with_renderable_arena<T: Send + Sync>(
        &self,
        f: impl FnOnce(&FlatStorageData<SceneNodeRenderable>) -> T,
    ) -> Option<T> {
        self.renderables
            .data()
            .try_read()
            .ok()
            .map(|arena| f(&arena))
    }

    pub(crate) fn with_renderable_arena_mut<T>(
        &self,
        f: impl FnOnce(&mut FlatStorageData<SceneNodeRenderable>) -> T,
    ) -> T {
        self.renderables.with_data_mut(|arena| f(arena))
    }

    /// Try to run `f` with a write guard on the renderables arena.
    pub fn try_with_renderable_arena_mut<T>(
        &self,
        f: impl FnOnce(&mut FlatStorageData<SceneNodeRenderable>) -> T,
    ) -> Option<T> {
        self.renderables
            .data()
            .try_write()
            .ok()
            .map(|mut arena| f(&mut arena))
    }

    /// Returns a serializable snapshot of the scene, including the root size and node hierarchy.
    pub fn snapshot(&self) -> SceneSnapshot {
        let size = {
            let guard = self.size.read().unwrap();
            SceneDimensions {
                width: guard.x,
                height: guard.y,
            }
        };

        let nodes = self.with_arena(collect_scene_roots);

        SceneSnapshot { size, nodes }
    }

    /// Serializes the current scene snapshot into a pretty formatted JSON string for debugging.
    pub fn serialize_state_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.snapshot())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneSnapshot {
    pub size: SceneDimensions,
    pub nodes: Vec<SceneNodeSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneDimensions {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneNodeSnapshot {
    pub id: usize,
    pub key: String,
    pub hidden: bool,
    pub pointer_events: bool,
    pub image_cached: bool,
    pub picture_cached: bool,
    pub needs_layout: bool,
    pub needs_repaint: bool,
    pub opacity: f32,
    pub local_bounds: RectSnapshot,
    pub global_bounds: RectSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub render_layer: RenderLayer,
    pub children: Vec<SceneNodeSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RectSnapshot {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn collect_scene_roots(arena: &Arena<SceneNode>) -> Vec<SceneNodeSnapshot> {
    let mut roots = Vec::new();
    for node in arena.iter() {
        if node.is_removed() || node.parent().is_some() {
            continue;
        }

        if let Some(node_id) = arena.get_node_id(node) {
            if let Some(snapshot) = SceneNodeSnapshot::from_arena(arena, node_id) {
                roots.push(snapshot);
            }
        }
    }

    roots
}

impl RectSnapshot {
    fn from_rect(rect: &skia_safe::Rect) -> Self {
        Self {
            x: rect.left(),
            y: rect.top(),
            width: rect.width(),
            height: rect.height(),
        }
    }
}

impl SceneNodeSnapshot {
    fn from_arena(arena: &Arena<SceneNode>, id: TreeStorageId) -> Option<Self> {
        let node = arena.get(id)?;
        if node.is_removed() {
            return None;
        }

        let scene_node = node.get();
        if scene_node.is_deleted() {
            return None;
        }

        let children = id
            .children(arena)
            .filter_map(|child| SceneNodeSnapshot::from_arena(arena, child))
            .collect();

        let render_layer = scene_node.render_layer().clone();
        let node_id: usize = id.into();

        let mut key = render_layer.key.clone();
        if key.is_empty() {
            key = format!("node-{}", node_id);
        }

        let content = if render_layer.content.is_some() {
            Some("picture".to_string())
        } else if render_layer.content_draw_func.is_some() {
            Some("dynamic".to_string())
        } else {
            None
        };

        Some(Self {
            id: node_id,
            key,
            hidden: scene_node.hidden(),
            pointer_events: scene_node.pointer_events(),
            image_cached: scene_node.is_image_cached(),
            picture_cached: scene_node.is_picture_cached(),
            needs_layout: scene_node.needs_layout(),
            needs_repaint: scene_node.needs_repaint(),
            opacity: render_layer.opacity,
            local_bounds: RectSnapshot::from_rect(&render_layer.local_transformed_bounds),
            global_bounds: RectSnapshot::from_rect(&render_layer.global_transformed_bounds),
            content,
            render_layer,
            children,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;

    #[test]
    fn serialize_state_pretty_exposes_tree_structure() {
        let engine = Engine::create(200.0, 100.0);
        let root = engine.new_layer();
        root.set_key("root");
        engine.scene_set_root(root.clone());

        let child = engine.new_layer();
        child.set_key("child");
        let child_id = child.id();
        root.add_sublayer(&child_id);

        engine.update(0.0);

        let snapshot = engine.scene.snapshot();
        assert_eq!(snapshot.nodes.len(), 1);
        assert_eq!(snapshot.nodes[0].key, "root");
        assert_eq!(snapshot.nodes[0].render_layer.key, "root");
        assert_eq!(snapshot.nodes[0].children.len(), 1);
        assert_eq!(snapshot.nodes[0].children[0].key, "child");
        assert_eq!(snapshot.nodes[0].children[0].render_layer.key, "child");

        let json = engine
            .scene
            .serialize_state_pretty()
            .expect("failed to serialize scene");
        let value: serde_json::Value = serde_json::from_str(&json).expect("invalid json");

        let nodes = value["nodes"].as_array().expect("nodes missing");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["key"], "root");
        assert_eq!(nodes[0]["render_layer"]["key"], "root");

        let children = nodes[0]["children"].as_array().expect("children missing");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["key"], "child");
        assert_eq!(children[0]["render_layer"]["key"], "child");
    }
}
