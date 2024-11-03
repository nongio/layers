#![allow(dead_code)]

//! Contains all the logic for running the animations, execute the scheduled changes
//! to the models, and prepare the render tree
//! The scene is drawn in 4 stages:
//! - The *layout* step calculates the dimensions and position of a node and generates a transformation Matrix
//! - The *draw* step generates a displaylist
//! - The *render* step uses the displaylist to generate a texture of the node
//! - The *compose* step generates the final image using the textures
//! The `LayersEngine` is the main engine responsible for managing layers and rendering.
//!
//! # Usage:
//!
//! ```
//! let engine = LayersEngine::new();
//! engine.add_layer(Layer::new());
//! engine.render();
//! ```
pub use layers_engine::LayersEngine;
pub use node::SceneNode;

mod layers_engine;
mod stages;

pub(crate) mod command;
pub(crate) mod draw_to_picture;
pub(crate) mod rendering;
pub(crate) mod scene;

pub mod animation;
pub(crate) mod node;
pub(crate) mod storage;

use core::fmt;
use indextree::NodeId;
use layers_engine::{initialize_engines, ENGINES, INIT};
use node::ContainsPoint;

#[cfg(feature = "debugger")]
use layers_debug_server::DebugServerError;
#[cfg(feature = "debugger")]
#[allow(unused_imports)]
use stages::send_debugger;

use stages::{nodes_for_layout, trigger_callbacks, update_node};
use taffy::prelude::*;

use std::{
    collections::HashMap,
    hash::Hash,
    ops::Deref,
    sync::{atomic::AtomicUsize, Arc, RwLock},
};

use self::{
    animation::{Animation, Transition},
    command::NoopChange,
    node::{DrawCacheManagement, RenderableFlags},
    scene::Scene,
    stages::{
        cleanup_animations, cleanup_nodes, cleanup_transactions, execute_transactions,
        update_animations, update_layout_tree,
    },
    storage::{FlatStorage, FlatStorageId, TreeStorageId, TreeStorageNode},
};
use crate::{
    layers::layer::{model::PointerHandlerFunction, Layer},
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

pub(crate) struct Engine {
    pub id: usize,
    /// The scene is a tree of nodes
    pub(crate) scene: Arc<Scene>,
    /// The root node of the scene
    scene_root: RwLock<Option<NodeRef>>,

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
    pointer_position: RwLock<Point>,
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

impl TransactionRef {
    pub(crate) fn engine(&self) -> Arc<Engine> {
        let engines = unsafe {
            INIT.call_once(initialize_engines);
            ENGINES.as_ref().unwrap()
        };
        let engines = engines.read().unwrap();
        engines.get(&self.engine_id).unwrap().clone()
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
impl From<NodeRef> for usize {
    fn from(node_ref: NodeRef) -> Self {
        node_ref.0.into()
    }
}

static UNIQ_POINTER_HANDLER_ID: AtomicUsize = AtomicUsize::new(0);

impl Engine {
    fn new(id: usize, width: f32, height: f32) -> Self {
        // rayon::ThreadPoolBuilder::new()
        //     .num_threads(2)
        //     .build_global()
        //     .unwrap();
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
        Engine {
            id,
            scene,
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
            pointer_position: RwLock::new(Point::default()),
            current_hover_node: RwLock::new(None),
        }
    }
    pub fn create(id: usize, width: f32, height: f32) -> Arc<Self> {
        let new_engine = Self::new(id, width, height);
        Arc::new(new_engine)
    }
    pub fn scene_set_root(&self, layer: impl Into<Layer>) -> NodeRef {
        let layer: Layer = layer.into();
        let layout = layer.layout_node_id;

        // append layer if it is not already in the scene
        let id = layer.id().unwrap_or_else(|| {
            let id = self.scene.add(layer.clone(), layout);
            layer.set_id(id);
            id
        });
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

        let change = Arc::new(NoopChange::new(id.0.into()));
        self.schedule_change(id, change, None);
        id
    }

    pub fn scene_add_layer(&self, layer: impl Into<Layer>, parent: Option<NodeRef>) -> NodeRef {
        let layer: Layer = layer.into();
        let layout = layer.layout_node_id;

        let new_parent = parent.or_else(|| {
            let scene_root = *self.scene_root.read().unwrap();
            scene_root
        });

        {
            let mut layout_tree = self.layout_tree.write().unwrap();
            if layer.id().is_some() {
                if let Some(layout_parent) = layout_tree.parent(layout) {
                    layout_tree.remove_child(layout_parent, layout).unwrap();
                }
            }
        }
        let layer_id = if new_parent.is_none() {
            // if we append to a scene without a root, we set the layer as the root
            self.scene_set_root(layer)
        } else {
            let new_parent = new_parent.unwrap();
            let id = layer.id().unwrap_or_else(|| {
                let id = self.scene.add(layer.clone(), layout);
                layer.set_id(id);
                id
            });

            let new_parent_node = self.scene.get_node(new_parent).unwrap();
            let new_parent_node = new_parent_node.get();
            new_parent_node.set_need_layout(true);

            let parent_layout = new_parent_node.layout_node_id;
            self.scene.append_node_to(id, new_parent);
            {
                let mut layout_tree = self.layout_tree.write().unwrap();
                layout_tree.add_child(parent_layout, layout).unwrap();
                let res = layout_tree.mark_dirty(parent_layout);
                if let Some(err) = res.err() {
                    println!("layout err {}", err);
                }
            }
            id
        };
        layer_id
    }
    pub fn scene_add_layer_to_positioned(
        &self,
        layer: impl Into<Layer>,
        parent: Option<NodeRef>,
    ) -> NodeRef {
        // FIXME ensure that newly added layers are layouted
        // update...
        {
            execute_transactions(self);
            nodes_for_layout(self);
            update_layout_tree(self);
            self.update_nodes();
        }

        let layer: Layer = layer.into();
        let position = layer.render_position();
        let parent_position = parent
            .and_then(|parent| {
                self.scene_get_node(&parent).map(|parent| {
                    let b = parent.get().transformed_bounds();
                    Point { x: b.x(), y: b.y() }
                })
            })
            .unwrap_or_default();
        let new_position = Point {
            x: position.x - parent_position.x,
            y: position.y - parent_position.y,
        };

        let node = self.scene_add_layer(layer.clone(), parent);
        layer.set_position(new_position, None);
        {
            execute_transactions(self);
            nodes_for_layout(self);
            update_layout_tree(self);
            self.update_nodes();
        }

        // println!("current position {:?}", position);
        // println!("parent position {:?}", parent_position);
        // println!("new model position {:?}", new_position);
        // let new_position = layer.render_position();
        // println!("new render position {:?}", new_position);

        node
    }
    pub fn mark_for_delete(&self, layer: NodeRef) {
        let node = self.scene.get_node(layer).unwrap();
        let node = node.get();
        node.mark_for_deletion();
    }
    pub(crate) fn scene_remove_layer(&self, layer: impl Into<Option<NodeRef>>) {
        let layer_id: Option<NodeRef> = layer.into();
        if let Some(layer_id) = layer_id {
            {
                if let Some(node) = self.scene.get_node(layer_id) {
                    let parent = node.parent();
                    let node = node.get();
                    self.scene.remove(layer_id);
                    if let Some(parent) = parent {
                        if let Some(parent) = self.scene.get_node(parent) {
                            let parent = parent.get();
                            parent.set_need_layout(true);

                            let mut layout = self.layout_tree.write().unwrap();
                            let res = layout.mark_dirty(parent.layout_node_id);
                            if let Some(err) = res.err() {
                                println!("layout err {}", err);
                            }
                        }
                    }
                    // remove layout node
                    let mut layout_tree = self.layout_tree.write().unwrap();
                    layout_tree.remove(node.layout_node_id).unwrap();
                }
            }
        }
    }
    pub fn scene_get_node(&self, node: &NodeRef) -> Option<TreeStorageNode<SceneNode>> {
        self.scene.get_node(*node)
    }
    pub fn scene_get_node_parent(&self, node: &NodeRef) -> Option<NodeRef> {
        let node = self.scene.get_node(*node)?;
        let parent = node.parent();
        parent.map(NodeRef)
    }
    pub fn now(&self) -> f32 {
        self.timestamp.read().unwrap().0
    }

    pub fn add_animation_from_transition(
        &self,
        transition: Transition,
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
        AnimationRef(self.animations.insert(AnimationState {
            animation,
            progress: 0.0,
            time: 0.0,
            is_running: autostart,
            is_finished: false,
            is_started: false,
        }))
    }
    pub fn start_animation(&self, animation: AnimationRef, delay: f32) {
        let animations = self.animations.data();
        let mut animations = animations.write().unwrap();
        if let Some(animation_state) = animations.get_mut(&animation.0) {
            animation_state.animation.start = self.timestamp.read().unwrap().0 + delay;
            animation_state.is_running = true;
            animation_state.is_finished = false;
            animation_state.progress = 0.0;
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
            let mut animated_node_change = animated_node_change.clone();
            if animation.is_some() {
                animated_node_change.animation_id = animation;
            }
            let value_id = animated_node_change.change.value_id();
            let transaction_id = self.transactions.insert(animated_node_change.clone());
            let transaction = TransactionRef {
                id: transaction_id,
                value_id,
                engine_id: self.id,
            };
            inserted_transactions.push(transaction);
        }
        inserted_transactions
    }

    pub fn get_transaction_for_value(&self, value_id: usize) -> Option<AnimatedNodeChange> {
        self.values_transactions
            .read()
            .unwrap()
            .get(&value_id)
            .and_then(|id| self.transactions.with_data(|d| d.get(id).cloned()))
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
        let node = self.scene.nodes.get(target_id.0);
        if node.is_some() {
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
        } else {
            println!(
                "Attention! Adding a change on a node not found {:?}",
                target_id
            );
            TransactionRef {
                id: 0,
                value_id: FlatStorageId::default(),
                engine_id: self.id,
            }
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
        let transactions = self.transactions.data();
        let mut transactions = transactions.write().unwrap();
        if let Some(transaction) = transactions.get_mut(&transaction.value_id) {
            transaction.animation_id = Some(animation);
            // FIXME should cancel existing animation?
        }
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

        // merge the updated nodes with the nodes that are part of the layout calculation
        nodes_for_layout(self);

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
        // iterate in parallel over the nodes and
        // repaint if necessary
        let layout = self.layout_tree.read().unwrap();
        let arena = self.scene.nodes.data();
        let arena = arena.read().unwrap();

        let mut damage = skia_safe::Rect::default();

        let node = self.scene_root.read().unwrap();

        if let Some(root_id) = *node {
            let (_, _, d) = update_node(&arena, &layout, root_id.0, None, false);
            damage = d;
        }

        damage
    }
    pub fn get_node_layout_style(&self, node: taffy::NodeId) -> Style {
        let layout = self.layout_tree.read().unwrap();
        layout.style(node).unwrap().clone()
    }
    pub fn set_node_layout_style(&self, node: taffy::NodeId, style: Style) {
        let mut layout = self.layout_tree.write().unwrap();
        layout.set_style(node, style).unwrap();
    }

    pub fn set_node_layout_size(&self, node: taffy::NodeId, size: crate::types::Size) {
        let mut layout = self.layout_tree.write().unwrap();
        let mut style = layout.style(node).unwrap().clone();
        let new_size = taffy::geometry::Size {
            width: size.width,
            height: size.height,
        };
        if style.size != new_size {
            style.size = new_size;
            layout.set_style(node, style).unwrap();
        }

        // println!("{:?} set_node_layout_size: {:?}", node, style.size);
        // layout.set_style(node, style).unwrap();
    }

    pub fn layer_at(&self, point: Point) -> Option<NodeRef> {
        let arena = self.scene.nodes.data();
        let arena = arena.read().unwrap();
        let mut result = None;
        for node in arena.iter() {
            let scene_node = node.get();
            if scene_node.contains(point) {
                let nodeid = arena.get_node_id(node).map(NodeRef);
                result = nodeid;
            }
        }
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
    fn bubble_up_event(&self, node_id: NodeRef, event_type: &PointerEventType) {
        if let Some(node) = self.scene.get_node(node_id.0) {
            if node.is_removed() {
                return;
            }
            let layer = node.get().layer.clone();

            if let Some(pointer_handler) = self.pointer_handlers.get(&node_id.0.into()) {
                let pos = *self.pointer_position.read().unwrap();
                // trigger node's own handlers
                for handler in pointer_handler.handlers(event_type) {
                    handler.0(layer.clone(), pos.x, pos.y);
                }
            }
            if let Some(parent_id) = node.parent() {
                self.bubble_up_event(NodeRef(parent_id), event_type);
            }
        }
    }
    /// Sends pointer move event to the engine
    pub fn pointer_move(
        &self,
        point: impl Into<Point>,
        root_id: impl Into<Option<NodeId>>,
    ) -> bool {
        let p = point.into();
        let mut root_id = root_id.into();

        if root_id.is_none() {
            // update engine pointer position
            *self.pointer_position.write().unwrap() = p;

            // get scene root node
            let root = *self.scene_root.read().unwrap().unwrap();
            root_id = Some(root);
        }
        let root_id = root_id.unwrap();
        let (root_node, children) = self.scene.with_arena(|arena| {
            let root_node = arena.get(root_id).unwrap().get().clone();
            let children: Vec<NodeId> = root_id.children(arena).collect();
            (root_node, children)
        });

        let root_node_hover = root_node
            .pointer_hover
            .load(std::sync::atomic::Ordering::SeqCst);

        let mut hover_self = false;
        let hidden = root_node.layer.hidden();
        let pointer_events = root_node.layer.pointer_events();
        if !hidden && pointer_events && root_node.contains(p) {
            hover_self = true;
            root_node
                .pointer_hover
                .store(true, std::sync::atomic::Ordering::SeqCst);
            self.current_hover_node
                .write()
                .unwrap()
                .replace(NodeRef(root_id));

            if !root_node_hover {
                if let Some(pointer_handler) = self.pointer_handlers.get(&root_id.into()) {
                    for handler in pointer_handler.on_in.values() {
                        handler.0(root_node.layer.clone(), p.x, p.y);
                    }
                }
            }
            if let Some(pointer_handler) = self.pointer_handlers.get(&root_id.into()) {
                for handler in pointer_handler.on_move.values() {
                    handler.0(root_node.layer.clone(), p.x, p.y);
                }
            }
        } else if root_node_hover {
            root_node
                .pointer_hover
                .store(false, std::sync::atomic::Ordering::SeqCst);
            self.current_hover_node.write().unwrap().take();
            if let Some(pointer_handler) = self.pointer_handlers.get(&root_id.into()) {
                for handler in pointer_handler.on_out.values() {
                    handler.0(root_node.layer.clone(), p.x, p.y);
                }
            }
        }
        let mut hover_children = false;
        if !hidden {
            for node_id in children {
                if !hover_children {
                    if self.pointer_move(p, node_id) {
                        hover_children = true;
                    }
                } else {
                    let node = self.scene.get_node(node_id).unwrap().get().clone();
                    if node.change_hover(false) {
                        if let Some(pointer_handler) = self.pointer_handlers.get(&node_id.into()) {
                            for handler in pointer_handler.on_out.values() {
                                handler.0(root_node.layer.clone(), p.x, p.y);
                            }
                        }
                    }
                }
            }
        }
        hover_self || hover_children
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
    pub fn get_pointer_position(&self) -> Point {
        *self.pointer_position.read().unwrap()
    }
}

impl Deref for NodeRef {
    type Target = TreeStorageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "debugger")]
impl layers_debug_server::DebugServer for Engine {
    fn handle_message(&self, result: std::result::Result<String, DebugServerError>) {
        match result {
            Ok(msg) => {
                if let Ok((command, node_id)) =
                    serde_json::from_str::<(String, NodeId)>(msg.as_str())
                {
                    match command.as_str() {
                        "highlight" => {
                            self.scene.with_arena(|arena| {
                                let node = arena.get(node_id).unwrap();
                                let scene_node: &crate::engine::node::SceneNode = node.get();
                                scene_node.set_debug_info(true);
                            });
                        }
                        "unhighlight" => {
                            self.scene.with_arena(|arena| {
                                let node = arena.get(node_id).unwrap();
                                let scene_node: &crate::engine::node::SceneNode = node.get();
                                scene_node.set_debug_info(false);
                            });
                        }

                        _ => {
                            println!("Unknown command: {}", command);
                        }
                    }
                }
            }
            Err(_) => {
                eprintln!("error receiving websocket msg");
            }
        }
    }
}
