#![allow(dead_code)]

//! Contains all the logic for running the animations, execute the scheduled changes
//! to the models, and prepare the render tree
//! The scene is drawn in 4 stages:
//! - The *layout* step calculates the dimensions and position of a node and generates a transformation Matrix
//! - The *draw* step generates a displaylist
//! - The *render* step uses the displaylist to generate a texture of the node
//! - The *compose* step generates the final image using the textures
//!
//! The `Engine` is the main engine responsible for managing layers and rendering.
//!
//! # Usage:
//!
//! ```
//! use lay_rs::prelude::*;
//!
//! let engine = Engine::create(500.0, 500.0);
//! let layer = engine.new_layer();
//! engine.add_layer(&layer);
//! ```

#![allow(unused_imports)]

pub use node::SceneNode;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod debug_server;

mod stages;

pub(crate) mod command;
pub(crate) mod draw_to_picture;
pub(crate) mod scene;

pub mod animation;
pub(crate) mod node;
pub(crate) mod storage;

use core::fmt;

use node::ContainsPoint;

#[cfg(feature = "debugger")]
#[allow(unused_imports)]
use stages::send_debugger;

use stages::{nodes_for_layout, trigger_callbacks, update_node_single};
use taffy::prelude::*;

use std::{
    collections::HashMap,
    hash::Hash,
    ops::Deref,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

use once_cell::sync::Lazy;

use self::{
    animation::{Animation, Transition},
    // command::NoopChange,
    node::RenderableFlags,
    scene::Scene,
    stages::{
        cleanup_animations, cleanup_nodes, cleanup_transactions, execute_transactions,
        update_animations, update_layout_tree,
    },
    storage::{FlatStorage, FlatStorageId, TreeStorageId},
};
use crate::{
    drawing::render_node_tree,
    engine::node::SceneNodeRenderable,
    layers::layer::{model::PointerHandlerFunction, render_layer::RenderLayer, Layer},
    prelude::ContentDrawFunction,
    types::Point,
};

#[derive(Clone)]
pub struct Timestamp(f32);

/// A trait for objects that can be exectuded by the engine.
pub trait Command {
    fn execute(&self, progress: f32) -> RenderableFlags;
    fn value_id(&self) -> usize;
}

pub trait SyncCommand: Command + Sync + Send + std::fmt::Debug {}

/// A group trait for commands that may contain an animation.
trait CommandWithAnimation: SyncCommand {
    fn animation(&self) -> Option<Animation>;
}

#[derive(Clone, Debug)]
pub struct AnimatedNodeChange {
    pub change: Arc<dyn SyncCommand>,
    pub animation_id: Option<AnimationRef>,
    pub node_id: NodeRef,
}

/// A struct that contains the state of a given animation.
#[derive(Clone, Debug)]
pub struct AnimationState {
    pub animation: Animation,
    pub time: f32,
    pub(crate) progress: f32,
    pub(crate) is_started: bool,
    pub(crate) is_running: bool,
    pub(crate) is_finished: bool,
}

static TRANSACTION_CALLBACK_ID: AtomicUsize = AtomicUsize::new(0);

fn transaction_callack_id() -> usize {
    TRANSACTION_CALLBACK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

type DynCallback = Arc<dyn 'static + Send + Sync + Fn(&Layer, f32)>;

#[derive(Clone)]
pub struct TransactionCallback {
    callback: DynCallback,
    pub(crate) once: bool,
    pub(crate) id: usize,
}

impl<F: Fn(&Layer, f32) + Send + Sync + 'static> From<F> for TransactionCallback {
    fn from(f: F) -> Self {
        TransactionCallback {
            callback: Arc::new(f),
            once: true,
            id: transaction_callack_id(),
        }
    }
}

impl PartialEq for TransactionCallback {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
pub enum TransactionEventType {
    Start,
    Update,
    Finish,
}
#[derive(Clone)]
struct TransitionCallbacks {
    pub on_start: Vec<TransactionCallback>,
    pub on_finish: Vec<TransactionCallback>,
    pub on_update: Vec<TransactionCallback>,
}

impl TransitionCallbacks {
    pub fn new() -> Self {
        Self {
            on_start: Vec::new(),
            on_finish: Vec::new(),
            on_update: Vec::new(),
        }
    }

    pub fn remove(&mut self, tr: &TransactionCallback) {
        self.on_start.retain(|h| h != tr);
        self.on_finish.retain(|h| h != tr);
        self.on_update.retain(|h| h != tr);
    }
    pub fn cleanup_once_callbacks(&mut self) {
        self.on_start.retain(|h| !h.once);
        self.on_finish.retain(|h| !h.once);
        self.on_update.retain(|h| !h.once);
    }
}
impl Default for TransitionCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct PointerCallback {
    pub on_move: HashMap<usize, PointerHandlerFunction>,
    pub on_in: HashMap<usize, PointerHandlerFunction>,
    pub on_out: HashMap<usize, PointerHandlerFunction>,
    pub on_down: HashMap<usize, PointerHandlerFunction>,
    pub on_up: HashMap<usize, PointerHandlerFunction>,
}

impl PointerCallback {
    pub fn new() -> Self {
        Self {
            on_move: HashMap::new(),
            on_in: HashMap::new(),
            on_out: HashMap::new(),
            on_down: HashMap::new(),
            on_up: HashMap::new(),
        }
    }
    pub fn handlers(
        &self,
        event_type: &PointerEventType,
    ) -> std::collections::hash_map::Values<'_, usize, PointerHandlerFunction> {
        match event_type {
            PointerEventType::Down => self.on_down.values(),
            PointerEventType::Up => self.on_up.values(),
            PointerEventType::In => self.on_in.values(),
            PointerEventType::Out => self.on_out.values(),
            PointerEventType::Move => self.on_move.values(),
        }
    }
}
impl Default for PointerCallback {
    fn default() -> Self {
        Self::new()
    }
}
pub enum PointerEventType {
    Move,
    In,
    Out,
    Down,
    Up,
}
/// Public API for the Layers Engine
/// ## Usage: Setup a basic scene with a root layer
/// ```rust
/// use lay_rs::prelude::*;
///
/// let engine = Engine::create(800.0, 600.0);
/// let layer = engine.new_layer();
/// let engine = Engine::create(1024.0, 768.0);
/// let root_layer = engine.new_layer();
/// root_layer.set_position(Point { x: 0.0, y: 0.0 }, None);

/// root_layer.set_background_color(
///     PaintColor::Solid {
///         color: Color::new_rgba255(180, 180, 180, 255),
///     },
///    None,
/// );
/// root_layer.set_border_corner_radius(10.0, None);
/// root_layer.set_layout_style(taffy::Style {
///     position: taffy::Position::Absolute,
///     display: taffy::Display::Flex,
///     flex_direction: taffy::FlexDirection::Column,
///     justify_content: Some(taffy::JustifyContent::Center),
///     align_items: Some(taffy::AlignItems::Center),
///     ..Default::default()
/// });
/// engine.add_layer(&root_layer);
/// ```
/// ## Usage: Update the engine
/// ```rust
/// use lay_rs::prelude::*;
///
/// let engine = Engine::create(800.0, 600.0);
/// // setup the scene...
/// engine.update(0.016);
/// ```
pub struct Engine {
    pub id: usize,
    /// The scene is a tree of nodes
    pub(crate) scene: Arc<Scene>,
    /// The root node of the scene
    scene_root: RwLock<Option<NodeRef>>,

    layers: RwLock<HashMap<NodeRef, Layer>>,

    pub(crate) values_transactions: RwLock<HashMap<usize, usize>>,
    /// The transactions (node changes) that are scheduled to be executed
    transactions: FlatStorage<AnimatedNodeChange>,
    /// The animations that are scheduled to be executed
    animations: FlatStorage<AnimationState>,
    /// The current timestamp of the engine
    pub(crate) timestamp: RwLock<Timestamp>,
    /// The Taffy layout tree
    pub(crate) layout_tree: RwLock<TaffyTree>,
    /// The root node of the layout tree
    layout_root: RwLock<taffy::prelude::NodeId>,

    /// The indexmap of handlers for the transactions
    transaction_handlers: FlatStorage<TransitionCallbacks>,
    /// The indexmap of handlers for the values
    value_handlers: FlatStorage<TransitionCallbacks>,

    /// The indexmap of handlers for the pointer events
    pointer_handlers: FlatStorage<PointerCallback>,

    /// The damage rect for the current frame
    pub(crate) damage: Arc<RwLock<skia_safe::Rect>>,
    /// The current pointer position
    pointer_position: RwLock<skia::Point>,
    /// The node that is currently hovered by the pointer
    /// Press, Release and CursorIn, CursorOut events are triggered
    /// based on the current_hover_node
    current_hover_node: RwLock<Option<NodeRef>>,
}

#[derive(Clone, Copy, Debug)]
pub struct TransactionRef {
    pub(crate) id: usize,
    pub value_id: FlatStorageId,
    pub(crate) engine_id: usize,
}

#[allow(static_mut_refs)]
impl TransactionRef {
    pub(crate) fn engine(&self) -> Arc<Engine> {
        ENGINE_REGISTRY
            .get(self.engine_id)
            .expect("no engine found")
    }
    /// Add a callback that is triggered when the transaction is started.
    /// The callback is removed when the transaction is finished.
    ///
    /// # Arguments
    /// * `handler`: the callback function to be called
    /// * `once`: if true, the callback is removed after it is triggered
    pub fn on_start<F: Into<TransactionCallback>>(&self, handler: F, once: bool) -> &Self {
        self.engine().on_start(*self, handler, once);
        self
    }
    /// Add a callback that is triggered when the transaction is finished.
    /// The callback is removed when the transaction is finished.
    ///
    /// # Arguments
    /// * `handler`: the callback function to be called
    /// * `once`: if true, the callback is removed after it is triggered
    pub fn on_finish<F: Into<TransactionCallback>>(&self, handler: F, once: bool) -> &Self {
        self.engine().on_finish(*self, handler, once);
        self
    }
    /// Add a callback that is triggered when the transaction is updated.
    /// The callback is removed when the transaction is finished.
    ///
    /// # Arguments
    /// * `handler`: the callback function to be called
    /// * `once`: if true, the callback is removed after it is triggered
    pub fn on_update<F: Into<TransactionCallback>>(&self, handler: F, once: bool) -> &Self {
        self.engine().on_update(*self, handler, once);
        self
    }
    /// Alias for on_finish
    ///
    /// the callback is added with the once flag set to true
    /// ie. the callback is removed after it is triggered
    pub fn then<F: Into<TransactionCallback>>(&self, handler: F) -> &Self {
        self.engine().on_finish(*self, handler, true);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnimationRef(FlatStorageId);

impl std::fmt::Display for AnimationRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnimationRef({})", self.0)
    }
}
/// An identifier for a node in the three storage
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, std::cmp::Ord, Hash)]
pub struct NodeRef(pub TreeStorageId);

impl fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index: usize = self.0.into();
        write!(f, "NodeRef({})", index)
    }
}
#[derive(Clone)]
pub struct HandlerRef(FlatStorageId);

impl From<NodeRef> for TreeStorageId {
    fn from(node_ref: NodeRef) -> Self {
        node_ref.0
    }
}
impl From<&NodeRef> for TreeStorageId {
    fn from(node_ref: &NodeRef) -> Self {
        node_ref.0
    }
}

impl From<NodeRef> for usize {
    fn from(node_ref: NodeRef) -> Self {
        node_ref.0.into()
    }
}
impl From<TreeStorageId> for NodeRef {
    fn from(val: TreeStorageId) -> Self {
        NodeRef(val)
    }
}
impl From<&TreeStorageId> for NodeRef {
    fn from(val: &TreeStorageId) -> Self {
        NodeRef(*val)
    }
}

static UNIQ_POINTER_HANDLER_ID: AtomicUsize = AtomicUsize::new(0);

// Global instance of the engine registry
static ENGINE_REGISTRY: Lazy<EngineRegistry> = Lazy::new(EngineRegistry::new);

/// Thread-safe registry for managing Engine instances
struct EngineRegistry {
    engines: RwLock<HashMap<usize, Arc<Engine>>>,
    next_id: AtomicUsize,
}

impl EngineRegistry {
    fn new() -> Self {
        Self {
            engines: RwLock::new(HashMap::new()),
            next_id: AtomicUsize::new(0),
        }
    }

    fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    fn register(&self, engine: Arc<Engine>) -> usize {
        let id = engine.id;
        self.engines.write().unwrap().insert(id, engine);
        id
    }

    fn get(&self, id: usize) -> Option<Arc<Engine>> {
        self.engines.read().unwrap().get(&id).cloned()
    }
}

impl fmt::Debug for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Engine({})", self.id)
    }
}
impl Engine {
    fn new(id: usize, width: f32, height: f32) -> Self {
        let mut layout_tree = TaffyTree::new();
        let layout_root = RwLock::new(
            layout_tree
                .new_leaf(Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                })
                .unwrap(),
        );

        let scene = Scene::create(width, height);
        let scene_root = RwLock::new(None);
        let damage = Arc::new(RwLock::new(skia_safe::Rect::default()));

        let layers = RwLock::new(HashMap::new());

        Engine {
            id,
            scene,
            layers,
            transactions: FlatStorage::new(),
            animations: FlatStorage::new(),
            timestamp: RwLock::new(Timestamp(0.0)),
            transaction_handlers: FlatStorage::new(),
            value_handlers: FlatStorage::new(),
            values_transactions: RwLock::new(HashMap::new()),
            layout_tree: RwLock::new(layout_tree),
            layout_root,
            scene_root,
            damage,
            pointer_handlers: FlatStorage::new(),
            pointer_position: RwLock::new(skia::Point::default()),
            current_hover_node: RwLock::new(None),
        }
    }

    fn get_arc_ref(&self) -> Arc<Self> {
        ENGINE_REGISTRY.get(self.id).unwrap()
    }

    pub fn create(width: f32, height: f32) -> Arc<Self> {
        let id = ENGINE_REGISTRY.next_id();
        let new_engine = Arc::new(Engine::new(id, width, height));
        ENGINE_REGISTRY.register(new_engine.clone());

        new_engine
    }
    /// set the layer as the root of the scene and root of the layout tree
    pub fn scene_set_root(&self, layer: impl Into<Layer>) -> NodeRef {
        let layer: Layer = layer.into();
        let layout = layer.layout_id;
        let id = layer.id;
        // detach the node from the scene
        {
            self.scene.with_arena_mut(|arena| {
                id.0.detach(arena);
            });
        }

        // set the new root
        let mut scene_root = self.scene_root.write().unwrap();
        *scene_root = Some(id);
        *self.layout_root.write().unwrap() = layout;
        // let mut layout_tree = self.layout_tree.write().unwrap();

        // let change = Arc::new(NoopChange::new(id.0.into()));
        // self.schedule_change(id, change, None);
        id
    }

    /// Set the size of the scene
    pub fn scene_set_size(&self, width: f32, height: f32) {
        self.scene.set_size(width, height);
    }

    /// Create a new layer associated with the engine
    pub fn new_layer(&self) -> Layer {
        let mut layout_tree = self.layout_tree.write().unwrap();
        let layout_id = layout_tree.new_leaf(Style::default()).unwrap();

        let scene_node = SceneNode::new();

        let scene_node_id = self.scene.insert_node(scene_node, None);

        let layer = Layer::with_engine(self.get_arc_ref(), scene_node_id, layout_id);
        self.layers
            .write()
            .unwrap()
            .insert(scene_node_id, layer.clone());

        layer
    }

    pub fn get_layer<'a>(&self, node: impl Into<&'a NodeRef>) -> Option<Layer> {
        let node_id = node.into();
        self.layers.read().unwrap().get(node_id).cloned()
    }

    pub fn with_layers(&self, f: impl Fn(&HashMap<NodeRef, Layer>)) {
        f(&self.layers.read().unwrap());
    }
    /// Detach the layer's layout node from the layout tree
    fn layout_detach_layer(&self, layer: &Layer) {
        let layout = layer.layout_id;

        {
            // if the layer has an id, then remove it from the layout tree
            let mut layout_tree = self.layout_tree.write().unwrap();

            if let Some(layout_parent) = layout_tree.parent(layout) {
                layout_tree.remove_child(layout_parent, layout).unwrap();
            }
        }
    }

    /// Append the layer's layout node to the layout tree
    fn layout_append_layer(&self, layer: &Layer, parent: NodeRef) {
        let layout = layer.layout_id;
        let parent_layout = {
            self.get_layer(&parent)
                .map(|parent_layer| parent_layer.layout_id)
        };
        if parent_layout.is_none() {
            return;
        }
        let parent_layout = parent_layout.unwrap();
        let mut layout_tree = self.layout_tree.write().unwrap();
        layout_tree.add_child(parent_layout, layout).unwrap();
        let res = layout_tree.mark_dirty(parent_layout);
        if let Some(err) = res.err() {
            println!("layout err {}", err);
        }
    }

    /// Prepend the layer's layout node to the layout tree
    fn layout_prepend_layer(&self, layer: &Layer, parent: NodeRef) {
        let layout = layer.layout_id;
        let parent_layout = {
            self.get_layer(&parent)
                .map(|parent_layer| parent_layer.layout_id)
        };
        if parent_layout.is_none() {
            return;
        }
        let parent_layout = parent_layout.unwrap();
        let mut layout_tree = self.layout_tree.write().unwrap();
        layout_tree
            .insert_child_at_index(parent_layout, 0, layout)
            .unwrap();
        let res = layout_tree.mark_dirty(parent_layout);
        if let Some(err) = res.err() {
            println!("layout err {}", err);
        }
    }

    /// Append the layer's node to the scene tree and layout tree
    /// the layer is appended to the parent node if it is provided
    /// otherwise it is appended to the root of the scene
    pub fn append_layer<'a>(
        &self,
        layer_id: impl Into<&'a NodeRef>,
        parent: impl Into<Option<NodeRef>>,
    ) {
        let layer_id = layer_id.into();
        if let Some(layer) = self.get_layer(layer_id) {
            let parent = parent.into();
            self.layout_detach_layer(&layer);
            let new_parent = parent.or_else(|| {
                let scene_root = *self.scene_root.read().unwrap();
                scene_root
            });

            if new_parent.is_none() {
                // if we append to a scene without a root, we set the layer as the root
                self.scene_set_root(layer);
            } else {
                let new_parent = new_parent.unwrap();

                self.scene.append_node_to(layer.id, new_parent);
                self.layout_append_layer(&layer, new_parent);
            }
        }
    }

    /// Append the layer to the root of the scene
    /// alias for append_layer, without a parent
    pub fn add_layer<'a>(&self, layer: impl Into<&'a NodeRef>) {
        self.append_layer(layer, None)
    }

    /// Prepend the layer to the root of the scene or to a parent node
    /// if the parent is provided
    pub fn prepend_layer(&self, layer: impl Into<Layer>, parent: impl Into<Option<NodeRef>>) {
        let layer: Layer = layer.into();
        let layer_id = layer.id;
        let parent = parent.into();
        self.layout_detach_layer(&layer);

        self.layout_detach_layer(&layer);

        let new_parent = parent.or_else(|| {
            let scene_root = *self.scene_root.read().unwrap();
            scene_root
        });

        if new_parent.is_none() {
            // if we append to a scene without a root, we set the layer as the root
            self.scene_set_root(layer);
        } else {
            let new_parent = new_parent.unwrap();

            self.scene.prepend_node_to(layer_id, new_parent);
            self.layout_prepend_layer(&layer, new_parent);
        }
    }

    pub fn add_layer_to_positioned(&self, layer: impl Into<Layer>, parent: Option<NodeRef>) {
        // FIXME ensure that newly added layers are layouted
        // update...
        {
            execute_transactions(self);
            update_layout_tree(self);
            self.update_nodes();
        }

        let layer: Layer = layer.into();
        let position = layer.render_position();
        let parent_position = parent
            .and_then(|parent| {
                self.scene.with_arena(|arena| {
                    arena.get(parent.0).map(|parent| {
                        let b = parent.get().transformed_bounds();
                        Point { x: b.x(), y: b.y() }
                    })
                })
            })
            .unwrap_or_default();
        let new_position = Point {
            x: position.x - parent_position.x,
            y: position.y - parent_position.y,
        };

        self.append_layer(&layer.id, parent);

        layer.set_position(new_position, None);
        {
            execute_transactions(self);
            update_layout_tree(self);
            self.update_nodes();
        }

        // println!("current position {:?}", position);
        // println!("parent position {:?}", parent_position);
        // println!("new model position {:?}", new_position);
        // let new_position = layer.render_position();
        // println!("new render position {:?}", new_position);
    }

    pub fn mark_for_delete(&self, layer: NodeRef) {
        self.scene.with_arena_mut(|arena| {
            if let Some(node) = arena.get_mut(layer.into()) {
                let node = node.get_mut();
                node.mark_for_deletion();
            }
        });
    }

    /// Remove the layer and its subtree from the scene and layout tree
    /// This method is intended by the engine from the cleanup stage
    pub(crate) fn scene_remove_layer<'a>(&self, layer: impl Into<&'a NodeRef>) {
        // Avoid deadlocks by not holding scene + layout/layers locks simultaneously.
        let layer_id = *layer.into();

        // Snapshot the scene parent id (read-only scene access)
        let parent_id = self.scene.with_arena(|arena| {
            arena
                .get(layer_id.into())
                .map(|n| n.parent())
                .unwrap_or(None)
        });

        // Determine if the parent still exists and is not already marked for deletion.
        let (parent_exists, parent_marked_for_deletion) = parent_id
            .map(|pid| {
                self.scene.with_arena(|arena| {
                    if pid.is_removed(arena) {
                        return (false, true);
                    }
                    if let Some(parent_node) = arena.get(pid) {
                        (true, parent_node.get().is_deleted())
                    } else {
                        (false, true)
                    }
                })
            })
            .unwrap_or((false, false));

        // Snapshot layout ids via layers map (separate lock)
        let (layout_id_opt, parent_layout_id_opt) = {
            let layout_id_opt = self.get_layer(&layer_id).map(|l| l.layout_id);
            let parent_layout_id_opt = parent_id
                .and_then(|pid| self.get_layer(&NodeRef(pid)))
                .map(|pl| pl.layout_id);
            (layout_id_opt, parent_layout_id_opt)
        };

        // Update layout tree first (no scene lock held)
        if let Some(layout_id) = layout_id_opt {
            let mut layout = self.layout_tree.write().unwrap();
            if parent_exists && !parent_marked_for_deletion {
                if let Some(parent_layout_id) = parent_layout_id_opt {
                    let _ = layout.mark_dirty(parent_layout_id);
                }
            }
            // Remove the layout node unconditionally
            let _ = layout.remove(layout_id);
        }

        // Now mutate the scene (no layout lock held)
        self.scene.with_arena_mut(|arena| {
            if let Some(pid) = parent_id {
                if !pid.is_removed(arena) {
                    if let Some(parent_node) = arena.get_mut(pid) {
                        parent_node.get_mut().set_needs_layout(true);
                    }
                    // remove layers subtree
                    layer_id.remove_subtree(arena);
                }
            }
        });
    }
    pub fn scene(&self) -> Arc<Scene> {
        self.scene.clone()
    }
    pub fn scene_root(&self) -> Option<NodeRef> {
        *self.scene_root.read().unwrap()
    }

    pub fn node_children(&self, node_ref: &NodeRef) -> Vec<NodeRef> {
        let mut children = Vec::new();
        self.scene.with_arena(|arena| {
            node_ref.0.children(arena).for_each(|child| {
                children.push(NodeRef(child));
            });
        });
        children
    }

    pub fn node_descendants(&self, node_ref: &NodeRef) -> Vec<NodeRef> {
        let mut descendants = Vec::new();
        self.scene.with_arena(|arena| {
            node_ref.0.descendants(arena).for_each(|descendant| {
                descendants.push(NodeRef(descendant));
            });
        });
        descendants
    }

    pub fn render_layer<'a>(&self, node_ref: impl Into<&'a NodeRef>) -> Option<RenderLayer> {
        let node_ref = node_ref.into();
        self.scene.with_arena(|arena| {
            let node = arena.get(node_ref.into())?;
            let node = node.get().render_layer().clone();
            Some(node)
        })
    }

    pub fn renderable<'a>(&self, node_ref: impl Into<&'a NodeRef>) -> Option<SceneNodeRenderable> {
        let node_ref = node_ref.into();
        self.scene.with_renderable_arena_mut(|arena| {
            let index: FlatStorageId = node_ref.0.into();
            let node = arena.get(&index)?;
            let node = node.clone();
            Some(node)
        })
    }

    pub fn node_render_size<'a>(&self, node_ref: impl Into<&'a NodeRef>) -> (f32, f32) {
        let node_ref = node_ref.into();
        self.scene.with_arena(|a| {
            a.get(node_ref.0)
                .map(|node| {
                    let node = node.get();
                    (node.render_layer.size.width, node.render_layer.size.height)
                })
                .unwrap_or((0.0, 0.0))
        })
    }
    /// Get a copy of the node
    pub fn scene_get_node(
        &self,
        node_ref: impl Into<NodeRef>,
    ) -> Option<indextree::Node<SceneNode>> {
        let node_ref = node_ref.into();
        self.scene
            .with_arena(|arena| arena.get(node_ref.into()).cloned())
    }
    pub fn scene_get_node_parent(&self, node_ref: NodeRef) -> Option<NodeRef> {
        self.scene.with_arena(|arena| {
            let node = arena.get(node_ref.into())?;
            let parent = node.parent();
            parent.map(NodeRef)
        })
    }
    pub fn now(&self) -> f32 {
        self.timestamp.read().unwrap().0
    }
    pub fn add_animation_from_transition(
        &self,
        transition: &Transition,
        autostart: bool,
    ) -> AnimationRef {
        let start = self.now() + transition.delay;
        self.add_animation(
            Animation {
                start,
                // duration: transition.duration,
                timing: transition.timing,
            },
            autostart,
        )
    }
    pub fn add_animation(&self, animation: Animation, autostart: bool) -> AnimationRef {
        let aid = self.animations.insert(AnimationState {
            animation,
            progress: 0.0,
            time: 0.0,
            is_running: autostart,
            is_finished: false,
            is_started: false,
        });
        AnimationRef(aid)
    }
    pub fn start_animation(&self, animation: AnimationRef, delay: f32) {
        self.animations.with_data_mut(|animations| {
            if let Some(animation_state) = animations.get_mut(&animation.0) {
                animation_state.animation.start = self.timestamp.read().unwrap().0 + delay;
                animation_state.is_running = true;
                animation_state.is_finished = false;
                animation_state.progress = 0.0;
            }
        });
    }

    pub fn get_transaction(&self, tref: TransactionRef) -> Option<AnimatedNodeChange> {
        self.transactions.get(&tref.id)
    }
    pub fn get_transaction_for_value(&self, value_id: usize) -> Option<AnimatedNodeChange> {
        // Avoid holding multiple locks at once to prevent deadlocks
        let tid = {
            let vt = self.values_transactions.read().unwrap();
            vt.get(&value_id).copied()
        };
        tid.and_then(|id| self.transactions.with_data(|d| d.get(&id).cloned()))
    }
    pub fn get_animation(&self, animation: AnimationRef) -> Option<AnimationState> {
        self.animations.with_data(|d| d.get(&animation.0).cloned())
    }
    pub fn schedule_change(
        &self,
        target_id: NodeRef,
        change: Arc<dyn SyncCommand>,
        animation_id: Option<AnimationRef>,
    ) -> TransactionRef {
        let value_id: usize = change.value_id();

        let animated_node_change = AnimatedNodeChange {
            change,
            animation_id,
            node_id: target_id,
        };
        let transaction_id = self.transactions.insert(animated_node_change);
        let mut values_transactions = self.values_transactions.write().unwrap();
        if let Some(existing_transaction) = values_transactions.get(&value_id) {
            self.cancel_transaction(TransactionRef {
                id: *existing_transaction,
                value_id,
                engine_id: self.id,
            });
        }
        values_transactions.insert(value_id, transaction_id);
        TransactionRef {
            id: transaction_id,
            value_id,
            engine_id: self.id,
        }
    }
    pub fn schedule_changes(
        &self,
        animated_changes: &[AnimatedNodeChange],
        animation: impl Into<Option<AnimationRef>>,
    ) -> Vec<TransactionRef> {
        let animation = animation.into();
        let mut inserted_transactions = Vec::with_capacity(animated_changes.len());
        for animated_node_change in animated_changes {
            // FIXME
            // let mut animated_node_change = animated_node_change.clone();
            // if animation.is_some() {
            //     animated_node_change.animation_id = animation;
            // }
            // let value_id = animated_node_change.change.value_id();
            // let transaction_id = self.transactions.insert(animated_node_change.clone());
            // let transaction = TransactionRef {
            //     id: transaction_id,
            //     value_id,
            //     engine_id: self.id,
            // };
            let transaction = self.schedule_change(
                animated_node_change.node_id,
                animated_node_change.change.clone(),
                animation,
            );
            inserted_transactions.push(transaction);
        }
        inserted_transactions
    }
    pub fn attach_animation(&self, transaction: TransactionRef, animation: AnimationRef) {
        self.transactions.with_data_mut(|transactions| {
            if let Some(transaction) = transactions.get_mut(&transaction.value_id) {
                transaction.animation_id = Some(animation);
                // FIXME should cancel existing animation?
            }
        });
    }
    pub fn cancel_animation(&self, animation: AnimationRef) {
        self.animations.with_data_mut(|d| {
            d.remove(&animation.0);
        });
    }
    pub fn cancel_transaction(&self, transaction: TransactionRef) {
        self.transactions.with_data_mut(|d| {
            d.remove(&transaction.id);
        });
    }
    pub fn step_time(&self, dt: f32) {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);
    }
    #[profiling::function]
    pub fn update(&self, dt: f32) -> bool {
        let timestamp = {
            let mut timestamp = self.timestamp.write().unwrap();
            let t = Timestamp(timestamp.0 + dt);
            *timestamp = t.clone();
            t
        };

        // 1.1 Update animations to the current timestamp
        let (started_animations, finished_animations) = update_animations(self, &timestamp);

        // 1.2 Execute transactions using the updated animations
        let (updated_nodes, finished_transitions, _needs_redraw) = execute_transactions(self);

        let needs_draw = !updated_nodes.is_empty();

        // 2.0 update the layout tree using taffy
        update_layout_tree(self);

        // 3.0 update render nodes and trigger repaint

        let mut damage = self.update_nodes();

        // 4.0 trigger the callbacks for the listeners on the transitions
        trigger_callbacks(self, &started_animations);

        // 5.0 cleanup the animations marked as done and
        // transactions already exectured
        cleanup_animations(self, finished_animations);
        cleanup_transactions(self, finished_transitions);

        // 6.0 cleanup the nodes that are marked as removed
        let removed_damage = cleanup_nodes(self);

        damage.join(removed_damage);

        let mut current_damage = self.damage.write().unwrap();
        current_damage.join(damage);

        #[cfg(feature = "debugger")]
        {
            let scene_root = self.scene_root.read().unwrap().unwrap();
            send_debugger(self.scene.clone(), scene_root);
        }

        needs_draw || !damage.is_empty()
    }
    #[profiling::function]
    pub fn update_nodes(&self) -> skia_safe::Rect {
        let layout = self.layout_tree.read().unwrap();
        let mut total_damage = skia_safe::Rect::default();
        let node = self.scene_root.read().unwrap();
        if let Some(root_id) = *node {
            // Phase 1: Collect nodes and compute their depth
            let nodes_post_order: Vec<_> = self.scene.with_arena(|arena| {
                let mut result = Vec::new();
                for edge in root_id.traverse(arena) {
                    if let indextree::NodeEdge::End(id) = edge {
                        result.push(id);
                    }
                }
                result
            });

            // Phase 2: Group nodes by depth to ensure parent dependencies
            let depth_groups = self.scene.with_arena(|arena| {
                let mut depth_map: std::collections::HashMap<usize, Vec<indextree::NodeId>> =
                    std::collections::HashMap::new();

                for &node_id in &nodes_post_order {
                    let depth = node_id.ancestors(arena).skip(1).count();
                    depth_map.entry(depth).or_default().push(node_id);
                }

                let mut groups: Vec<_> = depth_map.into_iter().collect();
                // Sort by depth ascending so parents (depth 0) are processed first
                groups.sort_by_key(|(depth, _)| *depth);
                groups
            });

            // Phase 3: Process each depth level from root to leaves
            // Parents are processed before children so cumulative transforms are correct.
            let mut parents_changed: std::collections::HashSet<indextree::NodeId> =
                std::collections::HashSet::new();
            for (_depth, nodes_at_depth) in depth_groups.into_iter() {
                // Update nodes at this depth in parallel
                let nad: &Vec<_> = nodes_at_depth.as_ref();
                let results: Vec<_> = nad
                    .iter()
                    .map(|node_id| {
                        let (parent_render_layer, parent_changed) =
                            self.scene.with_arena(|arena| {
                                let parent_id = arena[*node_id].parent();
                                let parent_layer = parent_id
                                    .and_then(|pid| arena.get(pid))
                                    .map(|parent_node| parent_node.get().render_layer().clone());
                                let parent_changed = parent_id
                                    .map(|pid| parents_changed.contains(&pid))
                                    .unwrap_or(false);
                                (parent_layer, parent_changed)
                            });

                        let result = update_node_single(
                            self,
                            &layout,
                            *node_id,
                            parent_render_layer.as_ref(),
                            parent_changed,
                        );
                        if !result.damage.is_empty() {
                            self.mark_image_cached_ancestors_for_repaint(*node_id);
                        }

                        (*node_id, result)
                    })
                    .collect();

                // Phase 4: Accumulate child damages to parents (sequential for each depth)
                for (node_id, result) in results.iter() {
                    if result.propagate_to_children {
                        parents_changed.insert(*node_id);
                    }
                    total_damage.join(result.damage);
                }
            }
        }

        total_damage
    }

    /// Helper method to propagate damage up the tree
    fn propagate_damage_to_ancestors(&self, node_id: indextree::NodeId, damage: skia_safe::Rect) {
        self.scene.with_arena_mut(|arena| {
            let ancestors: Vec<_> = node_id.ancestors(arena).skip(1).collect();

            for ancestor_id in ancestors {
                if let Some(ancestor_node) = arena.get_mut(ancestor_id) {
                    let ancestor = ancestor_node.get_mut();

                    // Update the bounds_with_children to include child damage
                    let child_damage_in_local = {
                        // Transform damage from global to ancestor's local space
                        let inverse_transform = ancestor.render_layer.transform_33.invert();
                        if let Some(inv) = inverse_transform {
                            inv.map_rect(damage).0
                        } else {
                            damage
                        }
                    };

                    ancestor
                        .render_layer
                        .bounds_with_children
                        .join(child_damage_in_local);
                    ancestor
                        .render_layer
                        .global_transformed_bounds_with_children
                        .join(damage);

                    ancestor.set_needs_repaint(true);
                    // Mark ancestor for potential repaint in the renderable arena
                    // self.scene.with_renderable_arena_mut(|renderable_arena| {
                    //     if let Some(ancestor_renderable) = renderable_arena.get_mut(ancestor_id) {
                    //         let ancestor_renderable = ancestor_renderable.get_mut();
                    //         ancestor_renderable.repaint_damage.join(damage);
                    //     }
                    // });
                }
            }
        });
    }
    fn mark_image_cached_ancestors_for_repaint(&self, node_id: indextree::NodeId) {
        self.scene.with_arena_mut(|arena| {
            let ancestor_ids: Vec<_> = node_id.ancestors(arena).skip(1).collect();
            for ancestor_id in ancestor_ids {
                if let Some(ancestor_node) = arena.get_mut(ancestor_id) {
                    let ancestor = ancestor_node.get_mut();
                    ancestor.set_needs_repaint(true);
                    if ancestor.is_image_cached() {
                        ancestor.increase_frame();
                    }
                }
            }
        });
    }
    pub fn get_node_layout_style(&self, node: taffy::NodeId) -> Style {
        let layout = self.layout_tree.read().unwrap();
        layout.style(node).unwrap().clone()
    }
    pub fn set_node_layout_style(&self, node: taffy::NodeId, style: Style) {
        let mut layout = self.layout_tree.write().unwrap();
        layout.set_style(node, style).unwrap();
    }

    pub fn set_node_layout_size(&self, node: taffy::NodeId, size: crate::types::Size) -> bool {
        let mut layout = self.layout_tree.write().unwrap();
        let mut style = layout.style(node).unwrap().clone();
        let new_size = taffy::geometry::Size {
            width: size.width,
            height: size.height,
        };
        if style.size != new_size {
            style.size = new_size;
            layout.set_style(node, style).unwrap();
            return true;
        }
        false
    }

    pub fn scene_layer_at(&self, point: Point) -> Option<NodeRef> {
        let mut result = None;
        self.scene.with_arena(|arena| {
            for node in arena.iter() {
                let scene_node = node.get();
                if scene_node.contains(point) {
                    let nodeid = arena.get_node_id(node).map(NodeRef);
                    result = nodeid;
                }
            }
        });
        result
    }
    #[allow(clippy::unwrap_or_default)]
    fn add_transaction_handler(
        &self,
        transaction: TransactionRef,
        event_type: TransactionEventType,
        handler: TransactionCallback,
    ) {
        let mut ch = self
            .transaction_handlers
            .get(&transaction.id)
            .unwrap_or_else(TransitionCallbacks::new);

        match event_type {
            TransactionEventType::Start => ch.on_start.push(handler),
            TransactionEventType::Finish => ch.on_finish.push(handler),
            TransactionEventType::Update => ch.on_update.push(handler),
        };

        self.transaction_handlers.insert_with_id(ch, transaction.id);
    }

    #[allow(clippy::unwrap_or_default)]
    fn add_value_handler(
        &self,
        value_id: usize,
        event_type: TransactionEventType,
        handler: TransactionCallback,
    ) {
        let mut ch = self
            .value_handlers
            .get(&value_id)
            .unwrap_or_else(TransitionCallbacks::new);

        match event_type {
            TransactionEventType::Start => ch.on_start.push(handler),
            TransactionEventType::Finish => ch.on_finish.push(handler),
            TransactionEventType::Update => ch.on_update.push(handler),
        };

        self.value_handlers.insert_with_id(ch, value_id);
    }

    pub fn on_start<F: Into<TransactionCallback>>(
        &self,
        transaction: TransactionRef,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_transaction_handler(transaction, TransactionEventType::Start, handler);
    }

    pub fn on_finish<F: Into<TransactionCallback>>(
        &self,
        transaction: TransactionRef,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_transaction_handler(transaction, TransactionEventType::Finish, handler);
    }

    pub fn on_update<F: Into<TransactionCallback>>(
        &self,
        transaction: TransactionRef,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_transaction_handler(transaction, TransactionEventType::Update, handler);
    }
    pub fn on_update_value<F: Into<TransactionCallback>>(
        &self,
        value_id: usize,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_value_handler(value_id, TransactionEventType::Update, handler);
    }
    pub fn on_start_value<F: Into<TransactionCallback>>(
        &self,
        value_id: usize,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_value_handler(value_id, TransactionEventType::Start, handler);
    }
    pub fn on_finish_value<F: Into<TransactionCallback>>(
        &self,
        value_id: usize,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_value_handler(value_id, TransactionEventType::Finish, handler);
    }
    #[allow(clippy::unwrap_or_default)]
    pub(crate) fn add_pointer_handler<F: Into<PointerHandlerFunction>>(
        &self,
        layer_node: NodeRef,
        event_type: PointerEventType,
        handler: F,
    ) -> usize {
        let node_id = layer_node.0.into();
        let mut pointer_callback = self
            .pointer_handlers
            .get(&node_id)
            .unwrap_or_else(PointerCallback::new);
        let handler = handler.into();
        let handler_id = UNIQ_POINTER_HANDLER_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        match event_type {
            PointerEventType::Move => {
                pointer_callback.on_move.insert(handler_id, handler);
            }
            PointerEventType::In => {
                pointer_callback.on_in.insert(handler_id, handler);
            }
            PointerEventType::Out => {
                pointer_callback.on_out.insert(handler_id, handler);
            }
            PointerEventType::Down => {
                pointer_callback.on_down.insert(handler_id, handler);
            }
            PointerEventType::Up => {
                pointer_callback.on_up.insert(handler_id, handler);
            }
        }

        self.pointer_handlers
            .insert_with_id(pointer_callback, node_id);

        handler_id
    }

    pub fn remove_pointer_handler(&self, layer_node: NodeRef, handler_id: usize) {
        let node_id = layer_node.0.into();
        if let Some(mut pointer_callback) = self.pointer_handlers.get(&node_id) {
            pointer_callback.on_move.remove(&handler_id);
            pointer_callback.on_in.remove(&handler_id);
            pointer_callback.on_out.remove(&handler_id);
            pointer_callback.on_down.remove(&handler_id);
            pointer_callback.on_up.remove(&handler_id);

            self.pointer_handlers
                .insert_with_id(pointer_callback, node_id);
        }
    }

    pub fn remove_all_pointer_handlers(&self, layer_node: NodeRef) {
        let node_id = layer_node.0.into();
        if let Some(mut pointer_callback) = self.pointer_handlers.get(&node_id) {
            pointer_callback.on_move.clear();
            pointer_callback.on_in.clear();
            pointer_callback.on_out.clear();
            pointer_callback.on_down.clear();
            pointer_callback.on_up.clear();

            self.pointer_handlers
                .insert_with_id(pointer_callback, node_id);
        }
    }
    fn bubble_up_event(&self, node_ref: NodeRef, event_type: &PointerEventType) {
        let ancestors: Vec<_> = self.scene.with_arena(|arena| {
            let node_id = node_ref.0;
            node_id
                .ancestors(arena)
                .filter(|node| !node.is_removed(arena))
                .collect()
        });
        let pos = *self.pointer_position.read().unwrap();
        for ancestor in ancestors.iter().rev() {
            if let Some(pointer_handler) = self.pointer_handlers.get(&(*ancestor).into()) {
                // trigger node's own handlers
                let layer = self.get_layer(&NodeRef(*ancestor)).unwrap();
                for handler in pointer_handler.handlers(event_type) {
                    handler.0(&layer, pos.x, pos.y);
                }
            }
        }
    }
    /// Sends pointer move event to the engine
    pub fn pointer_move(&self, p: &skia::Point, root_id: impl Into<Option<TreeStorageId>>) -> bool {
        *self.pointer_position.write().unwrap() = *p;

        // get the starting node
        let mut root_id = root_id.into();

        if root_id.is_none() {
            let root = *self.scene_root.read().unwrap().unwrap();
            root_id = Some(root);
        }

        let root_id = root_id.unwrap();

        let (current_node, in_node, out_node) = self.scene.with_arena(|arena| {
            let descendants: Vec<TreeStorageId> = root_id.descendants(arena).collect();

            let mut new_hover = None;
            for node_id in descendants.iter().rev() {
                let node_id = *node_id;
                let node = arena.get(node_id).unwrap().get();
                if node.hidden() || !node.pointer_events() {
                    continue;
                }
                if node.contains_point(p) {
                    new_hover = Some(NodeRef(node_id));
                    break;
                }
            }
            let mut in_node = None;
            let mut out_node = None;

            if let Some(new_hover_node) = new_hover {
                let mut current_hover = self.current_hover_node.write().unwrap();
                let old_hover = current_hover.replace(new_hover_node);

                if old_hover != new_hover {
                    in_node = Some(new_hover_node);
                    if old_hover.is_some() {
                        out_node = old_hover;
                    }
                }
            } else {
                let mut current_hover = self.current_hover_node.write().unwrap();
                if let Some(old_hover) = current_hover.take() {
                    out_node = Some(old_hover);
                }
            }
            (new_hover, in_node, out_node)
        });
        if let Some(node) = current_node {
            self.bubble_up_event(node, &PointerEventType::Move);
        }
        if let Some(node) = in_node {
            self.bubble_up_event(node, &PointerEventType::In);
        }
        if let Some(node) = out_node {
            self.bubble_up_event(node, &PointerEventType::Out);
        }
        current_node.is_some()
    }
    pub fn pointer_button_down(&self) {
        if let Some(node) = *self.current_hover_node.read().unwrap() {
            self.bubble_up_event(node, &PointerEventType::Down);
        }
    }
    pub fn pointer_button_up(&self) {
        if let Some(node) = *self.current_hover_node.read().unwrap() {
            self.bubble_up_event(node, &PointerEventType::Up);
        }
    }
    pub fn current_hover(&self) -> Option<NodeRef> {
        *self.current_hover_node.read().unwrap()
    }
    pub fn get_pointer_position(&self) -> skia::Point {
        *self.pointer_position.read().unwrap()
    }

    pub fn layer_as_content(&self, layer: &Layer) -> ContentDrawFunction {
        let engine_ref = self.get_arc_ref();
        let layer_id = layer.id;
        let draw_function = move |c: &skia::Canvas, w: f32, h: f32| {
            let scene = engine_ref.scene.clone();
            scene.with_arena(|arena| {
                scene.with_renderable_arena(|renderable_arena| {
                    render_node_tree(layer_id, arena, renderable_arena, c, 1.0);
                });
            });
            skia::Rect::from_xywh(0.0, 0.0, w, h)
        };
        ContentDrawFunction::from(draw_function)
    }
    pub fn damage(&self) -> skia_safe::Rect {
        *self.damage.read().unwrap()
    }
    pub fn clear_damage(&self) {
        let mut damage = self.damage.write().unwrap();
        *damage = skia_safe::Rect::default();
    }
    pub fn add_damage(&self, rect: skia_safe::Rect) {
        let mut damage = self.damage.write().unwrap();
        damage.join(rect);
    }
}

impl Deref for NodeRef {
    type Target = TreeStorageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
