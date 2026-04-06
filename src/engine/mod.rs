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
//! use layers::prelude::*;
//!
//! let engine = Engine::create(500.0, 500.0);
//! let layer = engine.new_layer();
//! engine.add_layer(&layer);
//! ```

#![allow(unused_imports)]

pub use node::SceneNode;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::error;

mod debug_server;

mod stages;

pub(crate) mod command;
pub(crate) mod draw_to_picture;
pub mod scene;

pub mod animation;
pub(crate) mod node;
pub mod occlusion;
pub(crate) mod storage;
pub mod task;

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
        atomic::{AtomicBool, AtomicUsize, Ordering},
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
use skia_safe::RoundOut;

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

fn transaction_callback_id() -> usize {
    TRANSACTION_CALLBACK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

type DynTransitionCallback = Arc<dyn 'static + Send + Sync + Fn(&Layer, f32)>;
type DynAnimationCallback = Arc<dyn 'static + Send + Sync + Fn(f32)>;

#[derive(Clone)]
pub struct TransactionCallback {
    callback: DynTransitionCallback,
    pub(crate) once: bool,
    pub(crate) id: usize,
}

#[derive(Clone)]
pub struct AnimationCallback {
    callback: DynAnimationCallback,
    pub(crate) once: bool,
    pub(crate) id: usize,
}

impl<F: Fn(&Layer, f32) + Send + Sync + 'static> From<F> for TransactionCallback {
    fn from(f: F) -> Self {
        TransactionCallback {
            callback: Arc::new(f),
            once: true,
            id: transaction_callback_id(),
        }
    }
}

impl PartialEq for TransactionCallback {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<F: Fn(f32) + Send + Sync + 'static> From<F> for AnimationCallback {
    fn from(f: F) -> Self {
        AnimationCallback {
            callback: Arc::new(f),
            once: true,
            id: transaction_callback_id(),
        }
    }
}

impl PartialEq for AnimationCallback {
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
struct AnimationCallbacks {
    pub on_start: Vec<AnimationCallback>,
    pub on_update: Vec<AnimationCallback>,
    pub on_finish: Vec<AnimationCallback>,
}

impl AnimationCallbacks {
    pub fn new() -> Self {
        AnimationCallbacks {
            on_start: Vec::new(),
            on_update: Vec::new(),
            on_finish: Vec::new(),
        }
    }

    pub fn remove(&mut self, callback: &AnimationCallback) {
        self.on_start.retain(|c| c != callback);
        self.on_update.retain(|c| c != callback);
        self.on_finish.retain(|c| c != callback);
    }
    pub fn cleanup_once_callbacks(&mut self) {
        self.on_start.retain(|c| !c.once);
        self.on_update.retain(|c| !c.once);
        self.on_finish.retain(|c| !c.once);
    }
}

impl Default for AnimationCallbacks {
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
/// use layers::prelude::*;
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
/// use layers::prelude::*;
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
    /// The indexmap of handlers for the animations
    animation_handlers: FlatStorage<AnimationCallbacks>,

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

    /// Cached list of nodes eligible for pointer hit-testing.
    /// Built by traversing from root, skipping hidden subtrees,
    /// and collecting nodes with pointer_events enabled.
    hit_test_node_list: RwLock<Vec<TreeStorageId>>,
    /// Flag indicating the hit_test_node_list cache needs rebuild.
    /// Set when visibility or tree structure changes.
    hit_test_node_list_dirty: AtomicBool,

    /// Cached traversal orders, rebuilt only when the tree structure changes.
    /// `nodes_post_order` stores all nodes in post-order (children before parents).
    cached_nodes_post_order: RwLock<Vec<indextree::NodeId>>,
    /// `depth_groups` stores nodes grouped by depth (root-first), used by `update_nodes`.
    cached_depth_groups: RwLock<Vec<(usize, Vec<indextree::NodeId>)>>,
    /// Flag indicating the traversal caches need rebuild (set on tree structure changes).
    traversal_cache_dirty: AtomicBool,
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
pub struct AnimationRef {
    pub(crate) id: FlatStorageId,
    pub(crate) engine_id: usize,
}

#[allow(static_mut_refs)]
impl AnimationRef {
    pub(crate) fn engine(&self) -> Arc<Engine> {
        ENGINE_REGISTRY
            .get(self.engine_id)
            .expect("no engine found")
    }

    /// Add a callback that is triggered when the animation is started.
    pub fn on_start<F: Into<AnimationCallback>>(&self, handler: F, once: bool) -> &Self {
        self.engine().on_animation_start(*self, handler, once);
        self
    }

    /// Add a callback that is triggered when the animation is updated.
    pub fn on_update<F: Into<AnimationCallback>>(&self, handler: F, once: bool) -> &Self {
        self.engine().on_animation_update(*self, handler, once);
        self
    }

    /// Add a callback that is triggered when the animation is finished.
    pub fn on_finish<F: Into<AnimationCallback>>(&self, handler: F, once: bool) -> &Self {
        self.engine().on_animation_finish(*self, handler, once);
        self
    }
}

impl std::fmt::Display for AnimationRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnimationRef({})", self.id)
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
            animation_handlers: FlatStorage::new(),
            values_transactions: RwLock::new(HashMap::new()),
            layout_tree: RwLock::new(layout_tree),
            layout_root,
            scene_root,
            damage,
            pointer_handlers: FlatStorage::new(),
            pointer_position: RwLock::new(skia::Point::default()),
            current_hover_node: RwLock::new(None),
            hit_test_node_list: RwLock::new(Vec::new()),
            hit_test_node_list_dirty: AtomicBool::new(true), // Start dirty so first update populates cache
            cached_nodes_post_order: RwLock::new(Vec::new()),
            cached_depth_groups: RwLock::new(Vec::new()),
            traversal_cache_dirty: AtomicBool::new(true),
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

    /// Returns true if the layer's scene node still exists and has not been removed.
    /// Use this to detect stale layer handles before performing scene operations.
    pub fn is_layer_alive(&self, node: &NodeRef) -> bool {
        self.scene.with_arena(|arena| {
            arena
                .get((*node).into())
                .map(|n| !n.is_removed())
                .unwrap_or(false)
        })
    }

    pub fn with_layers(&self, f: impl Fn(&HashMap<NodeRef, Layer>)) {
        f(&self.layers.read().unwrap());
    }

    /// Find a layer by its key string. Returns the first matching layer,
    /// or `None` if no layer with that key exists.
    pub fn find_layer_by_key(&self, key: &str) -> Option<Layer> {
        let layers = self.layers.read().unwrap();
        layers.values().find(|l| l.key() == key).cloned()
    }
    /// Detach the layer's layout node from the layout tree
    fn layout_detach_layer(&self, layer: &Layer) {
        let layout = layer.layout_id;

        {
            // if the layer has an id, then remove it from the layout tree
            let mut layout_tree = self.layout_tree.write().unwrap();

            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(layout_parent) = layout_tree.parent(layout) {
                    if let Err(e) = layout_tree.remove_child(layout_parent, layout) {
                        error!("Failed to remove layout child (node may be freed): {}", e);
                    }
                }
            }))
            .is_err()
            {
                error!("layout_detach_layer panicked (likely invalid layout node)");
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
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Err(e) = layout_tree.add_child(parent_layout, layout) {
                error!("Failed to add layout child (node may be freed): {}", e);
            }
            let res = layout_tree.mark_dirty(parent_layout);
            if let Some(err) = res.err() {
                error!("Failed to mark layout dirty: {}", err);
            }
        }))
        .is_err()
        {
            error!("layout_append_layer panicked (likely invalid layout node)");
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
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Err(e) = layout_tree.insert_child_at_index(parent_layout, 0, layout) {
                error!("Failed to insert layout child (node may be freed): {}", e);
            }
            let res = layout_tree.mark_dirty(parent_layout);
            if let Some(err) = res.err() {
                error!("Failed to mark layout dirty: {}", err);
            }
        }))
        .is_err()
        {
            error!("layout_prepend_layer panicked (likely invalid layout node)");
        }
    }

    /// Append the layer's node to the scene tree and layout tree
    /// the layer is appended to the parent node if it is provided
    /// otherwise it is appended to the root of the scene
    ///
    /// Returns `Err(LayerError::StaleNode)` if the layer handle is stale.
    pub fn append_layer<'a>(
        &self,
        layer_id: impl Into<&'a NodeRef>,
        parent: impl Into<Option<NodeRef>>,
    ) -> Result<(), crate::layers::error::LayerError> {
        let layer_id = layer_id.into();
        let layer = self
            .get_layer(layer_id)
            .ok_or(crate::layers::error::LayerError::StaleNode)?;

        let parent = parent.into();
        self.layout_detach_layer(&layer);
        let new_parent = parent.or_else(|| {
            let scene_root = *self.scene_root.read().unwrap();
            scene_root
        });

        if let Some(new_parent) = new_parent {
            // Verify the parent node is still alive before appending
            if !self.is_layer_alive(&new_parent) {
                return Err(crate::layers::error::LayerError::StaleNode);
            }
            self.scene.append_node_to(layer.id, new_parent);
            self.layout_append_layer(&layer, new_parent);
        } else {
            self.scene_set_root(layer);
        }
        self.invalidate_hit_test_node_list();
        self.invalidate_traversal_cache();
        Ok(())
    }

    /// Append the layer to the root of the scene
    /// alias for append_layer, without a parent
    pub fn add_layer<'a>(
        &self,
        layer: impl Into<&'a NodeRef>,
    ) -> Result<(), crate::layers::error::LayerError> {
        self.append_layer(layer, None)
    }

    /// Prepend the layer to the root of the scene or to a parent node
    /// if the parent is provided
    ///
    /// Returns `Err(LayerError::StaleNode)` if the layer handle is stale.
    pub fn prepend_layer(
        &self,
        layer: impl Into<Layer>,
        parent: impl Into<Option<NodeRef>>,
    ) -> Result<(), crate::layers::error::LayerError> {
        let layer: Layer = layer.into();
        if !self.is_layer_alive(&layer.id) {
            return Err(crate::layers::error::LayerError::StaleNode);
        }
        let layer_id = layer.id;
        let parent = parent.into();
        self.layout_detach_layer(&layer);

        let new_parent = parent.or_else(|| {
            let scene_root = *self.scene_root.read().unwrap();
            scene_root
        });

        if let Some(new_parent) = new_parent {
            // Verify the parent node is still alive before prepending
            if !self.is_layer_alive(&new_parent) {
                return Err(crate::layers::error::LayerError::StaleNode);
            }
            self.scene.prepend_node_to(layer_id, new_parent);
            self.layout_prepend_layer(&layer, new_parent);
        } else {
            self.scene_set_root(layer);
        }
        self.invalidate_hit_test_node_list();
        self.invalidate_traversal_cache();
        Ok(())
    }

    /// Append the layer to the parent, adjusting its position to preserve its
    /// current global position relative to the new parent.
    ///
    /// Returns `Err(LayerError::StaleNode)` if the layer handle is stale.
    pub fn add_layer_to_positioned(
        &self,
        layer: impl Into<Layer>,
        parent: Option<NodeRef>,
    ) -> Result<(), crate::layers::error::LayerError> {
        {
            execute_transactions(self);
            update_layout_tree(self);
            self.update_nodes();
        }

        let layer: Layer = layer.into();
        if !self.is_layer_alive(&layer.id) {
            return Err(crate::layers::error::LayerError::StaleNode);
        }
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

        self.append_layer(&layer.id, parent)?;

        layer.set_position(new_position, None);
        {
            execute_transactions(self);
            update_layout_tree(self);
            self.update_nodes();
        }
        Ok(())
    }

    pub fn mark_for_delete(&self, layer: NodeRef) {
        self.cleanup_pointer_handlers_for_subtree(layer);
        self.scene.with_arena_mut(|arena| {
            if let Some(node) = arena.get_mut(layer.into()) {
                if node.is_removed() {
                    return;
                }
                let node = node.get_mut();
                node.mark_for_deletion();
            }
        });
        self.invalidate_traversal_cache();
    }

    /// Remove the layer and its subtree from the scene and layout tree
    /// This method is intended by the engine from the cleanup stage
    pub(crate) fn scene_remove_layer<'a>(&self, layer: impl Into<&'a NodeRef>) {
        // Avoid deadlocks by not holding scene + layout/layers locks simultaneously.
        let layer_id = *layer.into();

        // Cleanup pointer handlers before removal to avoid leaks.
        self.cleanup_pointer_handlers_for_subtree(layer_id);

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
        // Remove the layer from the layers map so stale handles can no longer be found.
        self.layers
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&layer_id);

        // Invalidate hit test node list since tree structure changed
        self.invalidate_hit_test_node_list();
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
            let node = arena.get(node_ref.into()).filter(|n| !n.is_removed())?;
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
                .filter(|n| !n.is_removed())
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
    /// Returns the number of transactions currently scheduled to be executed.
    /// Use this to determine whether another call to `update` is needed.
    pub fn pending_transactions_count(&self) -> usize {
        self.transactions.with_data(|d| d.len())
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
                timing: transition.timing.clone(),
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
        AnimationRef {
            id: aid,
            engine_id: self.id,
        }
    }
    pub fn start_animation(&self, animation: AnimationRef, delay: f32) {
        self.animations.with_data_mut(|animations| {
            if let Some(animation_state) = animations.get_mut(&animation.id) {
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
        self.animations.with_data(|d| d.get(&animation.id).cloned())
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
            d.remove(&animation.id);
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
    /// Update the engine state by dt seconds
    /// Returns true if a redraw is needed
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

        // Early exit: if no animations started/finished and no nodes were
        // updated, skip the expensive layout + render-node traversal.
        // Also skip the early exit when the tree structure changed (new nodes
        // added/removed) — those nodes need their render layers initialised.
        let tree_changed = self.traversal_cache_dirty.load(Ordering::Relaxed);
        if !needs_draw
            && !tree_changed
            && started_animations.is_empty()
            && finished_animations.is_empty()
        {
            let removed_damage = cleanup_nodes(self);
            if !removed_damage.is_empty() {
                let mut current_damage = self.damage.write().unwrap();
                current_damage.join(removed_damage);
                return true;
            }
            return false;
        }

        // 2.0 cleanup nodes marked for removal before layout/render
        // so traversal caches and arena access don't hit freed nodes.
        let removed_damage = cleanup_nodes(self);

        // 3.0 update the layout tree using taffy
        update_layout_tree(self);

        // 4.0 update render nodes and trigger repaint

        let mut damage = self.update_nodes();
        damage.join(removed_damage);

        // 5.0 trigger the callbacks for the listeners on the transitions
        trigger_callbacks(self, &started_animations);

        // 6.0 cleanup the animations marked as done and
        // transactions already executed
        cleanup_animations(self, finished_animations);
        cleanup_transactions(self, finished_transitions);

        let mut current_damage = self.damage.write().unwrap();
        current_damage.join(damage);

        #[cfg(feature = "debugger")]
        {
            send_debugger(self);
        }

        needs_draw || !damage.is_empty()
    }
    #[profiling::function]
    pub fn update_nodes(&self) -> skia_safe::Rect {
        let layout = self.layout_tree.read().unwrap();
        let mut total_damage = skia_safe::Rect::default();
        let node = self.scene_root.read().unwrap();
        let Some(root_id) = *node else {
            return total_damage;
        };

        // Rebuild traversal caches only when tree structure changed
        if self.traversal_cache_dirty.load(Ordering::Relaxed) {
            drop(node); // release scene_root read lock before rebuild
            self.rebuild_traversal_cache();
        }

        let nodes_post_order = self.cached_nodes_post_order.read().unwrap();
        let depth_groups = self.cached_depth_groups.read().unwrap();

        // Phase 3: Process each depth level from root to leaves
        // Parents are processed before children so cumulative transforms are correct.
        let mut parents_changed: std::collections::HashSet<indextree::NodeId> =
            std::collections::HashSet::new();
        for (_depth, nodes_at_depth) in depth_groups.iter() {
            let results: Vec<_> = nodes_at_depth
                .iter()
                .map(|node_id| {
                    let (parent_render_layer, parent_changed) = self.scene.with_arena(|arena| {
                        let parent_id = arena.get(*node_id).and_then(|n| n.parent());
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

        // Phase 5: Bubble up bounds from children to parents
        for node_id in nodes_post_order.iter() {
            self.bubble_up_bounds_to_parent(*node_id);
        }

        // Phase 6: Clear all backdrop blur regions before rebuilding
        self.scene.with_arena_mut(|arena| {
            for node_id in nodes_post_order.iter() {
                if let Some(node) = arena.get_mut(*node_id) {
                    node.get_mut().render_layer.backdrop_blur_region = None;
                }
            }
        });

        // Phase 7: Bubble up backdrop blur regions from children to parents
        for node_id in nodes_post_order.iter() {
            self.bubble_up_backdrop_blur_regions(*node_id);
        }

        // Phase 8: Include backdrop blur regions in damage if damage is not empty
        if !total_damage.is_empty() {
            self.scene.with_arena(|arena| {
                if let Some(root_node) = arena.get(*root_id) {
                    if let Some(backdrop_rrects) =
                        &root_node.get().render_layer.backdrop_blur_region
                    {
                        for rrect in backdrop_rrects {
                            total_damage.join(rrect.rect());
                        }
                    }
                }
            });
        }

        // Phase 9: Rebuild hit test node list if dirty
        if self.hit_test_node_list_dirty.load(Ordering::Relaxed) {
            self.rebuild_hit_test_node_list(root_id.0);
        }

        total_damage
    }

    /// Rebuild the hit test node list by traversing from root,
    /// skipping hidden subtrees, and collecting nodes with pointer_events enabled.
    fn rebuild_hit_test_node_list(&self, root_id: TreeStorageId) {
        let nodes = self.scene.with_arena(|arena| {
            let mut result = Vec::new();
            Self::collect_hit_test_nodes(root_id, arena, &mut result);
            result
        });
        *self.hit_test_node_list.write().unwrap() = nodes;
        self.hit_test_node_list_dirty
            .store(false, Ordering::Relaxed);
    }

    /// Recursively collect hit-testable node IDs, skipping entire hidden subtrees.
    fn collect_hit_test_nodes(
        node_id: TreeStorageId,
        arena: &indextree::Arena<SceneNode>,
        result: &mut Vec<TreeStorageId>,
    ) {
        let Some(node) = arena.get(node_id) else {
            return;
        };
        if node.is_removed() {
            return;
        }
        let scene_node = node.get();

        // Skip entire subtree if this node is hidden
        if scene_node.hidden() {
            return;
        }

        // Add this node if it has pointer_events enabled
        if scene_node.pointer_events() {
            result.push(node_id);
        }

        // Recurse into children
        for child_id in node_id.children(arena) {
            Self::collect_hit_test_nodes(child_id, arena, result);
        }
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

    /// Bubble up a child's bounds to its parent's bounds_with_children.
    /// Called after processing each depth level so parents accumulate children's bounds.
    fn bubble_up_bounds_to_parent(&self, node_id: indextree::NodeId) {
        self.scene.with_arena_mut(|arena| {
            // Get the child's local_transformed_bounds_with_children (in parent's coordinate space)
            let child_bounds_in_parent_space = arena
                .get(node_id)
                .map(|n| n.get().render_layer.local_transformed_bounds_with_children);

            let Some(child_bounds) = child_bounds_in_parent_space else {
                return;
            };

            // Get the parent and update its bounds
            let parent_id = arena.get(node_id).and_then(|n| n.parent());
            let Some(parent_id) = parent_id else {
                return;
            };

            // Stop bubbling across hidden ancestors.
            // If the immediate parent is hidden, descendants should not contribute
            // backdrop regions outside that hidden subtree.
            if let Some(parent_node) = arena.get(parent_id) {
                if parent_node.get().hidden() {
                    return;
                }
            }

            if let Some(parent_node) = arena.get_mut(parent_id) {
                let parent = parent_node.get_mut();

                // Union child bounds into parent's bounds_with_children (local space)
                parent.render_layer.bounds_with_children.join(child_bounds);

                // Recompute local_transformed_bounds_with_children by transforming
                // the entire bounds_with_children through local_transform
                let (local_transformed_bwc, _) = parent
                    .render_layer
                    .local_transform
                    .to_m33()
                    .map_rect(parent.render_layer.bounds_with_children);
                parent.render_layer.local_transformed_bounds_with_children = local_transformed_bwc;

                // Update global_transformed_bounds_with_children
                let (global_bwc, _) = parent
                    .render_layer
                    .transform_33
                    .map_rect(parent.render_layer.bounds_with_children);
                parent.render_layer.global_transformed_bounds_with_children = global_bwc;
            }
        });
    }

    /// Bubble up backdrop blur regions from children to parents.
    /// Transforms child's backdrop blur regions (including its own if it has BackgroundBlur)
    /// into parent's coordinate space and merges them into parent's backdrop_blur_region.
    /// Uses Vec<RRect> for thread safety, converted to Path during rendering.
    /// Skips hidden nodes and their subtrees.
    fn bubble_up_backdrop_blur_regions(&self, node_id: indextree::NodeId) {
        self.scene.with_arena_mut(|arena| {
            // Collect child's backdrop blur rrects, blend mode, and rounded bounds
            let (child_blend_mode, child_rbounds, child_rrects, child_transform) = {
                let Some(child_node) = arena.get(node_id) else {
                    return;
                };
                let child = child_node.get();

                // Early exit: skip if child is hidden (entire subtree is hidden)
                if child.hidden() {
                    return;
                }

                // Early exit: skip if child has no backdrop blur and no backdrop rrects from descendants
                let has_backdrop_blur =
                    child.render_layer.blend_mode == crate::types::BlendMode::BackgroundBlur;
                if !has_backdrop_blur && child.render_layer.backdrop_blur_region.is_none() {
                    return;
                }

                (
                    child.render_layer.blend_mode,
                    child.render_layer.rbounds,
                    child.render_layer.backdrop_blur_region.clone(),
                    child.render_layer.local_transform.to_m33(),
                )
            };

            let parent_id = arena.get(node_id).and_then(|n| n.parent());
            let Some(parent_id) = parent_id else {
                return;
            };

            if let Some(parent_node) = arena.get_mut(parent_id) {
                let parent = parent_node.get_mut();

                // Get or create parent's rrects vector
                let parent_rrects = parent
                    .render_layer
                    .backdrop_blur_region
                    .get_or_insert_with(Vec::new);

                // If child itself has BackgroundBlur, add its rounded bounds
                if child_blend_mode == crate::types::BlendMode::BackgroundBlur {
                    // Transform child's rrect to parent's coordinate space
                    // Since we can't directly transform rrects, transform the rect and preserve radii
                    let (transformed_rect, _) = child_transform.map_rect(child_rbounds.rect());
                    let radii = [
                        child_rbounds.radii(skia_safe::rrect::Corner::UpperLeft),
                        child_rbounds.radii(skia_safe::rrect::Corner::UpperRight),
                        child_rbounds.radii(skia_safe::rrect::Corner::LowerRight),
                        child_rbounds.radii(skia_safe::rrect::Corner::LowerLeft),
                    ];
                    let transformed_rrect =
                        skia_safe::RRect::new_rect_radii(transformed_rect, &radii);
                    parent_rrects.push(transformed_rrect);
                }

                // Merge child's backdrop rrects (from its descendants) transformed to parent space
                if let Some(child_rrects) = child_rrects {
                    for rrect in child_rrects {
                        let (transformed_rect, _) = child_transform.map_rect(rrect.rect());
                        let radii = [
                            rrect.radii(skia_safe::rrect::Corner::UpperLeft),
                            rrect.radii(skia_safe::rrect::Corner::UpperRight),
                            rrect.radii(skia_safe::rrect::Corner::LowerRight),
                            rrect.radii(skia_safe::rrect::Corner::LowerLeft),
                        ];
                        let transformed =
                            skia_safe::RRect::new_rect_radii(transformed_rect, &radii);
                        parent_rrects.push(transformed);
                    }
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
        let layout = self.layout_tree.read().unwrap_or_else(|e| e.into_inner());
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| layout.style(node).cloned()))
        {
            Ok(Ok(style)) => style,
            Ok(Err(e)) => {
                error!("Failed to get layout style (node may be freed): {}", e);
                Style::default()
            }
            Err(_) => {
                error!("get_node_layout_style panicked (likely invalid layout node)");
                Style::default()
            }
        }
    }
    pub fn set_node_layout_style(&self, node: taffy::NodeId, style: Style) {
        let mut layout = self.layout_tree.write().unwrap();
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Skip if style unchanged — avoids marking the taffy node dirty
            // which would trigger a full layout recomputation.
            if let Ok(existing) = layout.style(node) {
                if *existing == style {
                    return;
                }
            }
            if let Err(e) = layout.set_style(node, style) {
                error!("Failed to set layout style (node may be freed): {}", e);
            }
        }))
        .is_err()
        {
            error!("set_node_layout_style panicked (likely invalid layout node)");
        }
    }

    pub fn set_node_layout_size(&self, node: taffy::NodeId, size: crate::types::Size) -> bool {
        let mut layout = self.layout_tree.write().unwrap();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Some(existing_style) = layout.style(node).ok().cloned() else {
                error!("Failed to get node layout size (node may be freed)");
                return false;
            };
            let mut style = existing_style;
            let new_size = taffy::geometry::Size {
                width: size.width,
                height: size.height,
            };
            if style.size != new_size {
                style.size = new_size;
                if layout.set_style(node, style).is_err() {
                    error!("Failed to set node layout style (node may be freed)");
                    return false;
                }
                return true;
            }
            false
        })) {
            Ok(result) => result,
            Err(_) => {
                error!("set_node_layout_size panicked (likely invalid layout node)");
                false
            }
        }
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
    /// Prints the current transaction handlers, grouped by transaction id, to help debug leaks.
    pub fn debug_print_transaction_handlers(&self) {
        self.transaction_handlers.with_data(|handlers| {
            println!("Transaction handlers registered: {}", handlers.len());
            for (transaction_id, callbacks) in handlers.iter() {
                println!(
                    "  id={}: start={}, update={}, finish={}",
                    transaction_id,
                    callbacks.on_start.len(),
                    callbacks.on_update.len(),
                    callbacks.on_finish.len()
                );
            }
        });
    }
    /// Prints the current value handlers, grouped by value id, to help debug leaks.
    pub fn debug_print_value_handlers(&self) {
        self.value_handlers.with_data(|handlers| {
            println!("Value handlers registered: {}", handlers.len());
            for (value_id, callbacks) in handlers.iter() {
                println!(
                    "  id={}: start={}, update={}, finish={}",
                    value_id,
                    callbacks.on_start.len(),
                    callbacks.on_update.len(),
                    callbacks.on_finish.len()
                );
            }
        });
    }
    /// Prints the current pointer handlers, grouped by node id, to help debug leaks.
    pub fn debug_print_pointer_handlers(&self) {
        self.pointer_handlers.with_data(|handlers| {
            println!("Pointer handlers registered: {}", handlers.len());
            for (node_id, callbacks) in handlers.iter() {
                println!(
                    "  id={}: move={}, in={}, out={}, down={}, up={}",
                    node_id,
                    callbacks.on_move.len(),
                    callbacks.on_in.len(),
                    callbacks.on_out.len(),
                    callbacks.on_down.len(),
                    callbacks.on_up.len()
                );
            }
        });
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

    pub fn clear_value_handlers(&self, value_id: usize) {
        self.value_handlers.remove_at(&value_id);
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

    fn add_animation_handler(
        &self,
        animation_id: FlatStorageId,
        event_type: TransactionEventType,
        handler: AnimationCallback,
    ) {
        let mut ch = self
            .animation_handlers
            .get(&animation_id)
            .unwrap_or_default();
        match event_type {
            TransactionEventType::Start => ch.on_start.push(handler),
            TransactionEventType::Finish => ch.on_finish.push(handler),
            TransactionEventType::Update => ch.on_update.push(handler),
        };

        self.animation_handlers.insert_with_id(ch, animation_id);
    }

    pub fn on_animation_start<F: Into<AnimationCallback>>(
        &self,
        animation: AnimationRef,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_animation_handler(animation.id, TransactionEventType::Start, handler);
    }

    pub fn on_animation_update<F: Into<AnimationCallback>>(
        &self,
        animation: AnimationRef,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_animation_handler(animation.id, TransactionEventType::Update, handler);
    }

    pub fn on_animation_finish<F: Into<AnimationCallback>>(
        &self,
        animation: AnimationRef,
        handler: F,
        once: bool,
    ) {
        let mut handler = handler.into();
        handler.once = once;
        self.add_animation_handler(animation.id, TransactionEventType::Finish, handler);
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
    /// Remove pointer handlers for the provided node and its descendants.
    fn cleanup_pointer_handlers_for_subtree(&self, layer: NodeRef) {
        let pointer_ids_to_remove: Vec<FlatStorageId> = self.scene.with_arena(|arena| {
            if layer.0.is_removed(arena) {
                return Vec::new();
            }
            let mut ids: Vec<FlatStorageId> = vec![layer.0.into()];
            ids.extend(layer.0.descendants(arena).map(|id| {
                let id: usize = id.into();
                id
            }));
            ids
        });

        for pid in pointer_ids_to_remove {
            self.pointer_handlers.remove_at(&pid);
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
        let root_id = root_id.into();

        // Use cached hit test node list (already filtered for hidden subtrees and pointer_events)
        let hit_test_nodes = self.hit_test_node_list.read().unwrap();

        let (current_node, in_node, out_node) = self.scene.with_arena(|arena| {
            let mut new_hover = None;
            // Iterate in reverse for back-to-front (topmost first) hit testing
            for node_id in hit_test_nodes.iter().rev() {
                let node_id = *node_id;
                // Skip nodes not under the specified root (if provided)
                if let Some(rid) = root_id {
                    if !node_id.ancestors(arena).any(|a| a == rid) {
                        continue;
                    }
                }
                let Some(node) = arena.get(node_id) else {
                    continue;
                };
                if node.is_removed() {
                    continue;
                }
                let node = node.get();
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
            let nodes = scene.nodes.data();
            let renderables = scene.renderables.data();

            // Try-lock to avoid blocking while a writer holds (or waits on) the arenas.
            if let (Ok(nodes), Ok(renderables)) = (nodes.try_read(), renderables.try_read()) {
                render_node_tree(layer_id, &nodes, &renderables, c, 1.0, None, None);
            }
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

    /// Compute occlusion culling for the given root node.
    ///
    /// Traverses the subtree front-to-back and marks nodes that are fully
    /// hidden behind opaque layers. Results are stored on the `Scene` and
    /// used by `render_node_tree` to skip occluded nodes.
    ///
    /// Call this after `update()` for each root node you intend to draw.
    pub fn compute_occlusion(&self, root: NodeRef) {
        let occluded = self
            .scene
            .with_arena(|arena| occlusion::compute_occlusion(root, arena));
        self.scene.add_occlusion(root, occluded);
    }

    /// Clear all cached occlusion data.
    ///
    /// Call this before recomputing occlusion for a new frame to avoid
    /// stale entries from previous roots.
    pub fn clear_occlusion(&self) {
        self.scene.clear_occlusion();
    }
    pub fn add_damage(&self, rect: skia_safe::Rect) {
        let mut damage = self.damage.write().unwrap();
        damage.join(rect);
    }

    /// Mark the hit test node list as dirty, requiring rebuild on next update.
    /// Called when visibility or tree structure changes.
    pub fn invalidate_hit_test_node_list(&self) {
        self.hit_test_node_list_dirty.store(true, Ordering::Relaxed);
    }

    /// Mark the traversal order caches as dirty, requiring rebuild on next
    /// `update_nodes()`.  Called when the tree structure changes (add/remove/reparent).
    pub fn invalidate_traversal_cache(&self) {
        self.traversal_cache_dirty.store(true, Ordering::Relaxed);
    }

    /// Rebuild the cached post-order and depth-grouped traversal lists from the
    /// current scene tree.  Only called when `traversal_cache_dirty` is set.
    fn rebuild_traversal_cache(&self) {
        let node = self.scene_root.read().unwrap();
        let Some(root_id) = *node else {
            *self.cached_nodes_post_order.write().unwrap() = Vec::new();
            *self.cached_depth_groups.write().unwrap() = Vec::new();
            self.traversal_cache_dirty.store(false, Ordering::Relaxed);
            return;
        };

        let (post_order, depth_groups) = self.scene.with_arena(|arena| {
            let mut post_order = Vec::new();
            let mut depth_map: std::collections::HashMap<usize, Vec<indextree::NodeId>> =
                std::collections::HashMap::new();

            for edge in root_id.traverse(arena) {
                if let indextree::NodeEdge::End(id) = edge {
                    post_order.push(id);
                }
            }

            for &node_id in &post_order {
                let depth = node_id.ancestors(arena).skip(1).count();
                depth_map.entry(depth).or_default().push(node_id);
            }

            let mut groups: Vec<_> = depth_map.into_iter().collect();
            groups.sort_by_key(|(depth, _)| *depth);
            (post_order, groups)
        });

        *self.cached_nodes_post_order.write().unwrap() = post_order;
        *self.cached_depth_groups.write().unwrap() = depth_groups;
        self.traversal_cache_dirty.store(false, Ordering::Relaxed);
    }
}

impl Deref for NodeRef {
    type Target = TreeStorageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
