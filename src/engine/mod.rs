#![allow(dead_code)]

//! Contains all the logic for running the animations, execute the scheduled changes
//! to the models, and prepare the render tree
//! The scene is drawn in 4 stages:
//! - The *layout* step calculates the dimensions and position of a node and generates a transformation Matrix
//! - The *draw* step generates a displaylist
//! - The *render* step uses the displaylist to generate a texture of the node
//! - The *compose* step generates the final image using the textures

mod draw_to_picture;
mod stages;

pub(crate) mod command;
pub(crate) mod rendering;
pub mod storage;

pub(crate) mod scene;

pub mod animation;
pub mod node;

use indextree::NodeId;
use node::ContainsPoint;
use taffy::prelude::*;

use core::fmt;
use std::{
    collections::HashMap,
    hash::Hash,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc, RwLock,
    },
};

use crate::{
    layers::layer::{model::PointerHandlerFunction, state::LayerDataProps, Layer, ModelLayer},
    types::Point,
};

use self::{
    animation::{Animation, Transition},
    command::NoopChange,
    node::{DrawCacheManagement, RenderableFlags, SceneNode},
    scene::Scene,
    stages::{
        cleanup_animations, cleanup_nodes, cleanup_transactions, execute_transactions,
        trigger_callbacks, update_animations, update_layout_tree, update_nodes,
    },
    storage::{FlatStorage, FlatStorageId, TreeStorageId, TreeStorageNode},
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
struct AnimatedNodeChange {
    pub change: Arc<dyn SyncCommand>,
    animation_id: Option<AnimationRef>,
    node_id: NodeRef,
}

/// A struct that contains the state of an animation.
/// The f32 is the current progress of the animation.
/// The bool is a flag that indicates if the animation is finished.
/// the progres can not be used to determine if the animation is finished
/// because the animation could be reversed or looped
#[derive(Clone)]
struct AnimationState {
    pub(crate) animation: Animation,
    pub(crate) progress: f32,
    pub(crate) is_running: bool,
    pub(crate) is_finished: bool,
}

type FnTransactionCallback = Arc<dyn 'static + Fn(f32) + Send + Sync>;

pub enum TransactionEventType {
    Start,
    Update,
    Finish,
}
#[derive(Clone)]
struct TransitionCallbacks {
    pub on_start: Vec<FnTransactionCallback>,
    pub on_finish: Vec<FnTransactionCallback>,
    pub on_update: Vec<FnTransactionCallback>,
}

impl TransitionCallbacks {
    pub fn new() -> Self {
        Self {
            on_start: Vec::new(),
            on_finish: Vec::new(),
            on_update: Vec::new(),
        }
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
}
impl Default for PointerCallback {
    fn default() -> Self {
        Self::new()
    }
}
pub(crate) enum PointerEventType {
    Move,
    In,
    Out,
    Down,
    Up,
}

pub(crate) struct Engine {
    pub(crate) scene: Arc<Scene>,
    scene_root: RwLock<Option<NodeRef>>,
    transactions: FlatStorage<AnimatedNodeChange>,
    animations: FlatStorage<AnimationState>,
    pub(crate) timestamp: RwLock<Timestamp>,
    transaction_handlers: FlatStorage<TransitionCallbacks>,
    pub(crate) layout_tree: RwLock<TaffyTree>,
    layout_root: RwLock<taffy::prelude::NodeId>,
    pub(crate) damage: RwLock<skia_safe::Rect>,
    // pointer handlers
    pointer_position: RwLock<Point>,
    current_hover_node: RwLock<Option<NodeId>>,
    pointer_handlers: FlatStorage<PointerCallback>,
}
#[derive(Clone, Copy, Debug)]
pub struct TransactionRef(pub FlatStorageId);

#[derive(Clone, Copy, Debug)]
pub struct AnimationRef(FlatStorageId);

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

/// Main struct to interact with the engine
/// ## Usage: Setup a basic scene with a root layer
/// ```rust
/// use layers::prelude::*;
///
/// let engine = LayersEngine::new(800.0, 600.0);
/// let layer = engine.new_layer();
/// let engine = LayersEngine::new(1024.0, 768.0);
/// let root_layer = engine.new_layer();
/// root_layer.set_position(Point { x: 0.0, y: 0.0 });

/// root_layer.set_background_color(
///     PaintColor::Solid {
///         color: Color::new_rgba255(180, 180, 180, 255),
///     }
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
/// engine.scene_add_layer(root_layer.clone());
/// ```
/// ## Usage: Update the engine
/// ```rust
/// use layers::prelude::*;
///
/// let engine = LayersEngine::new(800.0, 600.0);
/// // setup the scene...
/// engine.update(0.016);
/// ```
#[derive(Clone)]
pub struct LayersEngine {
    pub(crate) engine: Arc<Engine>,
}

impl LayersEngine {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            engine: Engine::create(width, height),
        }
    }
    pub fn set_scene_size(&self, width: f32, height: f32) {
        self.engine.scene.set_size(width, height);
    }
    pub fn new_layer(&self) -> Layer {
        let model = Arc::new(ModelLayer::default());

        let mut lt = self.engine.layout_tree.write().unwrap();

        let layout = lt
            .new_leaf(Style {
                position: Position::Absolute,
                ..Default::default()
            })
            .unwrap();

        Layer {
            engine: self.engine.clone(),
            model,
            id: Arc::new(RwLock::new(None)),
            key: Arc::new(RwLock::new(String::new())),
            layout_node_id: layout,
            hidden: Arc::new(AtomicBool::new(false)),
            image_cache: Arc::new(AtomicBool::new(false)),
            state: Arc::new(RwLock::new(LayerDataProps::default())),
        }
    }
    pub fn new_animation(&self, transition: Transition) -> AnimationRef {
        self.engine.add_animation_from_transition(transition)
    }
    pub fn attach_animation(&self, transaction: TransactionRef, animation: AnimationRef) {
        self.engine.attach_animation(transaction, animation);
    }
    pub fn update(&self, dt: f32) -> bool {
        self.engine.update(dt)
    }
    pub fn scene_add_layer(&self, layer: impl Into<Layer>) -> NodeRef {
        self.engine.scene_add_layer(layer, None)
    }
    pub fn scene_add_layer_to(&self, layer: impl Into<Layer>, parent: Option<NodeRef>) -> NodeRef {
        self.engine.scene_add_layer(layer, parent)
    }

    pub fn scene_remove_layer(&self, layer: impl Into<Option<NodeRef>>) {
        self.engine.scene_remove_layer(layer)
    }

    pub fn scene_set_root(&self, layer: impl Into<Layer>) -> NodeRef {
        self.engine.scene_set_root(layer)
    }
    pub fn scene_get_node(&self, node: NodeRef) -> Option<TreeStorageNode<SceneNode>> {
        self.engine.scene_get_node(node)
    }
    pub fn scene_get_node_parent(&self, node: NodeRef) -> Option<NodeRef> {
        self.engine.scene_get_node_parent(node)
    }
    pub fn scene(&self) -> &Arc<Scene> {
        &self.engine.scene
    }
    pub fn scene_root(&self) -> Option<NodeRef> {
        *self.engine.scene_root.read().unwrap()
    }
    pub fn step_time(&self, dt: f32) {
        self.engine.step_time(dt)
    }

    pub fn scene_layer_at(&self, point: Point) -> Option<NodeRef> {
        self.engine.layer_at(point)
    }
    pub fn damage(&self) -> skia_safe::Rect {
        *self.engine.damage.read().unwrap()
    }
    pub fn clear_damage(&self) {
        *self.engine.damage.write().unwrap() = skia_safe::Rect::default();
    }
    pub fn root_layer(&self) -> Option<SceneNode> {
        let root_id = self.scene_root()?;
        let node = self.scene_get_node(root_id)?;
        Some(node.get().clone())
    }
    pub fn pointer_move(&self, point: impl Into<Point>, root_id: impl Into<Option<NodeId>>) {
        self.engine.pointer_move(point, root_id);
    }
    pub fn pointer_button_down(&self) {
        self.engine.pointer_button_down();
    }
    pub fn pointer_button_up(&self) {
        self.engine.pointer_button_up();
    }
}

impl std::fmt::Debug for LayersEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayersEngine").finish()
    }
}

static UNIQ_POINTER_HANDLER_ID: AtomicUsize = AtomicUsize::new(0);

impl Engine {
    fn new(width: f32, height: f32) -> Self {
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
        let damage = RwLock::new(skia_safe::Rect::default());
        Engine {
            scene,
            transactions: FlatStorage::new(),
            animations: FlatStorage::new(),
            timestamp: RwLock::new(Timestamp(0.0)),
            transaction_handlers: FlatStorage::new(),
            layout_tree: RwLock::new(layout_tree),
            layout_root,
            scene_root,
            damage,
            pointer_handlers: FlatStorage::new(),
            pointer_position: RwLock::new(Point::default()),
            current_hover_node: RwLock::new(None),
        }
    }
    pub fn create(width: f32, height: f32) -> Arc<Self> {
        let new_engine = Self::new(width, height);
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
            let nodes = self.scene.nodes.data();
            let mut arena = nodes.write().unwrap();
            id.0.detach(&mut arena);
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
        let mut layout_tree = self.layout_tree.write().unwrap();
        if layer.id().is_some() {
            if let Some(layout_parent) = layout_tree.parent(layout) {
                layout_tree.remove_child(layout_parent, layout).unwrap();
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
            layout_tree.add_child(parent_layout, layout).unwrap();
            let res = layout_tree.mark_dirty(parent_layout);
            if let Some(err) = res.err() {
                println!("layout err {}", err);
            }
            id
        };
        layer_id
    }
    pub fn scene_remove_layer(&self, layer: impl Into<Option<NodeRef>>) {
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
    pub fn scene_get_node(&self, node: NodeRef) -> Option<TreeStorageNode<SceneNode>> {
        self.scene.get_node(node)
    }
    pub fn scene_get_node_parent(&self, node: NodeRef) -> Option<NodeRef> {
        let node = self.scene.get_node(node)?;
        let parent = node.parent();
        parent.map(NodeRef)
    }
    pub fn now(&self) -> f32 {
        self.timestamp.read().unwrap().0
    }

    pub fn add_animation_from_transition(&self, transition: Transition) -> AnimationRef {
        let start = self.now() + transition.delay;

        self.add_animation(
            Animation {
                start,
                duration: transition.duration,
                timing: transition.timing,
            },
            true,
        )
    }

    pub fn add_animation(&self, animation: Animation, autostart: bool) -> AnimationRef {
        AnimationRef(self.animations.insert(AnimationState {
            animation,
            progress: 0.0,
            is_running: autostart,
            is_finished: false,
        }))
    }
    pub fn start_animation(&self, animation: AnimationRef, delay: Option<f32>) {
        let animations = self.animations.data();
        let mut animations = animations.write().unwrap();
        if let Some(animation_state) = animations.get_mut(&animation.0) {
            animation_state.animation.start =
                self.timestamp.read().unwrap().0 + delay.unwrap_or(0.0);
            animation_state.is_running = true;
            animation_state.is_finished = false;
            animation_state.progress = 0.0;
        }
    }

    pub fn schedule_change(
        &self,
        target_id: NodeRef,
        change: Arc<dyn SyncCommand>,
        animation_id: Option<AnimationRef>,
    ) -> TransactionRef {
        let node = self.scene.nodes.get(target_id.0);
        if node.is_some() {
            let transaction_id: usize = change.value_id();
            let node_change = AnimatedNodeChange {
                change,
                animation_id,
                node_id: target_id,
            };
            TransactionRef(
                self.transactions
                    .insert_with_id(node_change, transaction_id),
            )
        } else {
            TransactionRef(0)
        }
    }

    pub fn attach_animation(&self, transaction: TransactionRef, animation: AnimationRef) {
        let transactions = self.transactions.data();
        let mut transactions = transactions.write().unwrap();
        if let Some(transaction) = transactions.get_mut(&transaction.0) {
            transaction.animation_id = Some(animation);
        }
    }
    pub fn step_time(&self, dt: f32) {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);
    }
    #[profiling::function]
    pub fn update(&self, dt: f32) -> bool {
        let mut timestamp = self.timestamp.write().unwrap();
        let t = Timestamp(timestamp.0 + dt);

        // 1.1 Update animations to the current timestamp
        let finished_animations = update_animations(self, &t);
        *timestamp = t;
        // 1.2 Execute transactions using the updated animations
        let (mut updated_nodes, finished_transitions, _needs_redraw) = execute_transactions(self);
        let needs_draw = !updated_nodes.is_empty();

        // merge the updated nodes with the nodes that are part of the layout calculation
        {
            #[cfg(feature = "profile-with-puffin")]
            profiling::puffin::profile_scope!("needs_layout");
            let arena = self.scene.nodes.data();
            let arena = arena.read().unwrap();
            updated_nodes = arena
                .iter()
                .filter_map(|node| {
                    if node.is_removed() {
                        return None;
                    }
                    let scene_node = node.get();
                    // let layout = self.get_node_layout_style(scene_node.layout_node_id);
                    // if
                    // if layout.position != Position::Absolute {
                    scene_node.insert_flags(RenderableFlags::NEEDS_LAYOUT);
                    scene_node.id()
                    // } else {
                    // None
                    // }
                })
                .collect();
        };
        // 2.0 update the layout tree using taffy
        update_layout_tree(self);

        // 3.0 update render nodes and trigger repaint
        let mut damage = update_nodes(self, updated_nodes);
        // 4.0 trigger the callbacks for the listeners on the transitions
        trigger_callbacks(self);

        // 5.0 cleanup the animations marked as done and
        // transactions already exectured
        cleanup_animations(self, finished_animations);
        cleanup_transactions(self, finished_transitions);

        // 6.0 cleanup the nodes that are marked as removed
        let removed_damage = cleanup_nodes(self);
        damage.join(removed_damage);
        *self.damage.write().unwrap() = damage;
        needs_draw
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
        style.size = taffy::geometry::Size {
            width: size.width,
            height: size.height,
        };

        // println!("{:?} set_node_layout_size: {:?}", node, style.size);
        layout.set_style(node, style).unwrap();
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
    fn add_transaction_handler<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        event_type: TransactionEventType,
        handler: F,
    ) {
        if let Some(t) = self.transactions.get(&transaction.0) {
            if let Some(animation) = t.animation_id {
                let mut ch = self
                    .transaction_handlers
                    .get(&animation.0)
                    .unwrap_or_else(TransitionCallbacks::new);
                let handler = Arc::new(handler) as FnTransactionCallback;
                match event_type {
                    TransactionEventType::Start => ch.on_start.push(handler),
                    TransactionEventType::Finish => ch.on_finish.push(handler),
                    TransactionEventType::Update => ch.on_update.push(handler),
                };

                self.transaction_handlers.insert_with_id(ch, animation.0);
            }
        }
    }

    pub fn on_start<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        handler: F,
    ) {
        self.add_transaction_handler(transaction, TransactionEventType::Start, handler);
    }

    pub fn on_finish<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        handler: F,
    ) {
        self.add_transaction_handler(transaction, TransactionEventType::Finish, handler);
    }

    pub fn on_update<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        handler: F,
    ) {
        self.add_transaction_handler(transaction, TransactionEventType::Update, handler);
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

    pub fn remove_all_handlers(&self, layer_node: NodeRef) {
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
    // fn bubble_up_event(&self, node_id: NodeId, event_type: PointerEventType) {

    // }
    pub fn pointer_move(&self, point: impl Into<Point>, root_id: impl Into<Option<NodeId>>) {
        let p = point.into();
        *self.pointer_position.write().unwrap() = p;
        let mut root_id = root_id.into();
        if root_id.is_none() {
            let root = *self.scene_root.read().unwrap().unwrap();
            root_id = Some(root);
        }
        let root_id = root_id.unwrap();
        let (root_node, children) = self.scene.with_arena(|arena| {
            let root_node = arena.get(root_id).unwrap().get().clone();
            let children: Vec<NodeId> = root_id.reverse_children(arena).collect();
            (root_node, children)
        });

        // let root_node = arena.get(root_id).unwrap().get();
        for node_id in children {
            // if let Some(scene_node) = self.scene.get_node(node_id) {
            // let scene_node = scene_node.get();
            // if scene_node.contains(p) {
            self.pointer_move(p, node_id);
            // }
            // }
        }
        let pointer_hover_node = root_node
            .pointer_hover
            .load(std::sync::atomic::Ordering::SeqCst);

        if root_node.contains(p) {
            root_node
                .pointer_hover
                .store(true, std::sync::atomic::Ordering::SeqCst);
            self.current_hover_node.write().unwrap().replace(root_id);

            if !pointer_hover_node {
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
        } else if pointer_hover_node {
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
    }
    pub fn pointer_button_down(&self) {
        if let Some(node) = *self.current_hover_node.read().unwrap() {
            if let Some(pointer_handler) = self.pointer_handlers.get(&node.into()) {
                let pos = *self.pointer_position.read().unwrap();
                if let Some(node) = self.scene.get_node(node) {
                    let layer = node.get().layer.clone();
                    // let parent = node.parent();
                    for handler in pointer_handler.on_down.values() {
                        handler.0(layer.clone(), pos.x, pos.y);
                    }
                }
            }
        }
    }
    pub fn pointer_button_up(&self) {
        if let Some(node) = *self.current_hover_node.read().unwrap() {
            if let Some(pointer_handler) = self.pointer_handlers.get(&node.into()) {
                let pos = *self.pointer_position.read().unwrap();
                let layer = self.scene.get_node(node).unwrap().get().layer.clone();
                for handler in pointer_handler.on_up.values() {
                    handler.0(layer.clone(), pos.x, pos.y);
                }
            }
        }
    }
}

impl Deref for NodeRef {
    type Target = TreeStorageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
