#![allow(dead_code)]

//! Contains all the logic for running the animations, execute the scheduled changes
//! to the models, and prepare the render tree
//! The scene is drawn in 4 stages:
//! - The *layout* step calculates the dimensions and position of a node and generates a transformation Matrix
//! - The *draw* step generates a displaylist
//! - The *render* step uses the displaylist to generate a texture of the node
//! - The *compose* step generates the final image using the textures

pub mod animation;
pub mod command;
mod draw_to_picture;
pub mod node;
pub mod pointer;
pub mod rendering;
pub mod scene;
mod stages;
pub mod storage;

use indextree::NodeId;
use taffy::prelude::*;

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
    layers::layer::{model::PointerHandlerFunction, Layer, ModelLayer},
    types::Point,
};

use self::{
    animation::{Animation, Transition},
    command::NoopChange,
    node::{ContainsPoint, DrawCacheManagement, RenderableFlags, SceneNode},
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
/// A trait for objects that contain a transition.
pub trait WithTransition {
    // fn transition(&self) -> Option<Transition<Easing>>;
}

/// A group trait for commands that may contain an animation.
pub trait CommandWithAnimation: SyncCommand {
    fn animation(&self) -> Option<Animation>;
}

#[derive(Clone, Debug)]
pub struct AnimatedNodeChange {
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
pub struct AnimationState {
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
pub struct TransitionCallbacks {
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
}

impl PointerCallback {
    pub fn new() -> Self {
        Self {
            on_move: HashMap::new(),
        }
    }
}
impl Default for PointerCallback {
    fn default() -> Self {
        Self::new()
    }
}
pub struct Engine {
    pub scene: Arc<Scene>,
    scene_root: RwLock<Option<NodeRef>>,
    transactions: FlatStorage<AnimatedNodeChange>,
    animations: FlatStorage<AnimationState>,
    pub timestamp: RwLock<Timestamp>,
    transaction_handlers: FlatStorage<TransitionCallbacks>,
    pub layout_tree: RwLock<Taffy>,
    layout_root: RwLock<taffy::node::Node>,
    pub damage: RwLock<skia_safe::Rect>,
    // pointer handlers
    pointer_handlers: FlatStorage<PointerCallback>,
    // pointer_down_handlers: FlatStorage<PointerDownCallback>,
    // pointer_up_handlers: FlatStorage<PointerUpCallback>,
    // pointer_hover_handlers: FlatStorage<PointerHoverCallback>,
    // pointer_leave_handlers: FlatStorage<PointerLeaveCallback>,
}
#[derive(Clone, Copy, Debug)]
pub struct TransactionRef(pub FlatStorageId);

#[derive(Clone, Copy, Debug)]
pub struct AnimationRef(FlatStorageId);

/// An identifier for a node in the three storage
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, std::cmp::Ord, Hash)]
pub struct NodeRef(pub TreeStorageId);

#[derive(Clone)]
pub struct HandlerRef(FlatStorageId);

impl From<NodeRef> for TreeStorageId {
    fn from(node_ref: NodeRef) -> Self {
        node_ref.0
    }
}

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
    pub fn pointer_move(&self, point: impl Into<Point>, root_id: NodeId) {
        self.engine.pointer_move(point, root_id);
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
        let mut layout_tree = Taffy::new();
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

        let parent = parent.or_else(|| {
            let scene_root = *self.scene_root.read().unwrap();
            scene_root
        });
        let id = if parent.is_none() {
            // if we append to a scene without a root, we set the layer as the root
            self.scene_set_root(layer)
        } else {
            let parent = parent.unwrap();
            let id = layer.id().unwrap_or_else(|| {
                let id = self.scene.add(layer.clone(), layout);
                layer.set_id(id);
                id
            });

            let parent_node = self.scene.get_node(parent).unwrap();
            let parent_node = parent_node.get();
            parent_node.set_need_layout(true);

            let parent_layout = parent_node.layout_node_id;
            self.scene.append_node_to(id, parent);

            let mut layout_tree = self.layout_tree.write().unwrap();
            layout_tree.add_child(parent_layout, layout).unwrap();
            let res = layout_tree.mark_dirty(parent_layout);
            if let Some(err) = res.err() {
                println!("layout err {}", err);
            }
            id
        };
        id
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

    pub fn get_node_layout_style(&self, node: Node) -> Style {
        let layout = self.layout_tree.read().unwrap();
        layout.style(node).unwrap().clone()
    }
    pub fn set_node_layout_style(&self, node: Node, style: Style) {
        let mut layout = self.layout_tree.write().unwrap();
        layout.set_style(node, style).unwrap();
    }

    pub fn set_node_layout_size(&self, node: Node, size: crate::types::Size) {
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
    pub fn add_pointer_handler<F: Into<PointerHandlerFunction>>(
        &self,
        layer_node: NodeRef,
        handler: F,
    ) -> usize {
        let node_id = layer_node.0.into();
        let mut pointer_callback = self
            .pointer_handlers
            .get(&node_id)
            .unwrap_or_else(PointerCallback::new);
        let handler = handler.into();
        let handler_id = UNIQ_POINTER_HANDLER_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        pointer_callback.on_move.insert(handler_id, handler);

        self.pointer_handlers
            .insert_with_id(pointer_callback, node_id);

        handler_id
    }

    pub fn remove_pointer_handler(&self, layer_node: NodeRef, handler_id: usize) {
        let node_id = layer_node.0.into();
        if let Some(mut pointer_callback) = self.pointer_handlers.get(&node_id) {
            pointer_callback.on_move.remove(&handler_id);
            self.pointer_handlers
                .insert_with_id(pointer_callback, node_id);
        }
    }

    pub fn remove_all_handlers(&self, layer_node: NodeRef) {
        let node_id = layer_node.0.into();
        if let Some(mut pointer_callback) = self.pointer_handlers.get(&node_id) {
            pointer_callback.on_move.clear();
        }
    }

    pub fn pointer_move(&self, point: impl Into<Point>, root_id: NodeId) {
        let p = point.into();
        let arena = self.scene.nodes.data();
        let arena = arena.read().unwrap();

        // let root_node = arena.get(root_id).unwrap().get();

        for node_id in root_id.reverse_children(&arena) {
            let scene_node = arena.get(node_id).unwrap().get();
            if scene_node.contains(p) {
                self.pointer_move(p, node_id);
            }
        }
        let root_node = arena.get(root_id).unwrap().get();
        if root_node.contains(p) {
            if let Some(pointer_handler) = self.pointer_handlers.get(&root_id.into()) {
                for handler in pointer_handler.on_move.values() {
                    handler.0(p.x, p.y);
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
