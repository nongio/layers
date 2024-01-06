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

use taffy::prelude::*;

use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::{
    layers::layer::{Layer, ModelLayer},
    types::Point,
};

use self::{
    animation::{Animation, Transition},
    command::NoopChange,
    node::{ContainsPoint, RenderableFlags},
    scene::Scene,
    stages::{
        cleanup_animations, cleanup_transactions, execute_transactions, trigger_callbacks,
        update_animations, update_layout_tree, update_nodes,
    },
    storage::{FlatStorage, FlatStorageId, TreeStorageId},
};
#[derive(Clone)]
pub struct Timestamp(f32);

/// A trait for objects that can be exectuded by the engine.
pub trait Command {
    fn execute(&self, progress: f32) -> RenderableFlags;
    fn value_id(&self) -> usize;
}

pub trait SyncCommand: Command + Sync + Send {}
/// A trait for objects that contain a transition.
pub trait WithTransition {
    // fn transition(&self) -> Option<Transition<Easing>>;
}

/// A group trait for commands that may contain an animation.
pub trait CommandWithAnimation: SyncCommand {
    fn animation(&self) -> Option<Animation>;
}

#[derive(Clone)]
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

type FnCallback = Arc<dyn 'static + Fn(f32) + Send + Sync>;

pub enum TransactionEventType {
    Start,
    Update,
    Finish,
}
#[derive(Clone)]
pub struct TransitionCallbacks {
    pub on_start: Vec<FnCallback>,
    pub on_finish: Vec<FnCallback>,
    pub on_update: Vec<FnCallback>,
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

pub struct Engine {
    pub scene: Arc<Scene>,
    scene_root: RwLock<Option<NodeRef>>,
    transactions: FlatStorage<AnimatedNodeChange>,
    animations: FlatStorage<AnimationState>,
    pub timestamp: RwLock<Timestamp>,
    transaction_handlers: FlatStorage<TransitionCallbacks>,
    pub layout_tree: RwLock<Taffy>,
    layout_root: RwLock<taffy::node::Node>,
}
#[derive(Clone, Copy)]
pub struct TransactionRef(pub FlatStorageId);

#[derive(Clone, Copy)]
pub struct AnimationRef(FlatStorageId);

/// An identifier for a node in the three storage
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, std::cmp::Ord)]
pub struct NodeRef(pub TreeStorageId);

#[derive(Clone)]
pub struct HandlerRef(FlatStorageId);

impl From<NodeRef> for TreeStorageId {
    fn from(node_ref: NodeRef) -> Self {
        node_ref.0
    }
}

pub struct LayersEngine {
    pub(crate) engine: Arc<Engine>,
}

impl LayersEngine {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            engine: Engine::create(width, height),
        }
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
            layout_node_id: layout,
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
}

impl std::fmt::Debug for LayersEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayersEngine").finish()
    }
}
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
        Engine {
            scene,
            transactions: FlatStorage::new(),
            animations: FlatStorage::new(),
            timestamp: RwLock::new(Timestamp(0.0)),
            transaction_handlers: FlatStorage::new(),
            layout_tree: RwLock::new(layout_tree),
            layout_root,
            scene_root,
        }
    }
    pub fn create(width: f32, height: f32) -> Arc<Self> {
        let new_engine = Self::new(width, height);
        Arc::new(new_engine)
    }
    pub fn scene_set_root(&self, layer: impl Into<Layer>) -> NodeRef {
        let layer: Layer = layer.into();
        let layout = layer.layout_node_id;

        let id = self.scene.add(layer.clone(), layout);
        layer.set_id(id);

        let mut scene_root = self.scene_root.write().unwrap();
        let layout_root = *self.layout_root.read().unwrap();
        let mut layout_tree = self.layout_tree.write().unwrap();

        if scene_root.is_none() {
            *scene_root = Some(id);
            layout_tree.remove(layout_root).unwrap();
            *self.layout_root.write().unwrap() = layout;
        } else {
            let scene_root = *scene_root.unwrap();
            self.scene.append_node_to(id, NodeRef(scene_root));
            layout_tree.add_child(layout_root, layout).unwrap();
        }
        let change = Arc::new(NoopChange::new(id.0.into()));
        self.schedule_change(id, change, None);
        id
    }

    pub fn scene_add_layer(&self, layer: impl Into<Layer>, parent: Option<NodeRef>) -> NodeRef {
        let layer: Layer = layer.into();
        let mut layout_tree = self.layout_tree.write().unwrap();

        let layer_layout = layer.layout_node_id;
        let id = self.scene.append(parent, layer.clone(), layer_layout);
        layer.set_id(id);

        if let Some(parent) = parent {
            let parent_layout = self.scene.get_node(parent).unwrap().get().layout_node_id;
            self.scene.append_node_to(id, parent);
            layout_tree.add_child(parent_layout, layer_layout).unwrap();

            let change: Arc<NoopChange> = Arc::new(NoopChange::new(parent.0.into()));
            self.schedule_change(parent, change, None);
        } else {
            let mut scene_root = self.scene_root.write().unwrap();
            let layout_root = *self.layout_root.read().unwrap();
            if scene_root.is_none() {
                *scene_root = Some(id);
                layout_tree.remove(layout_root).unwrap();
                *self.layout_root.write().unwrap() = layer_layout;
            } else {
                let scene_root = *scene_root.unwrap();
                let root_id = NodeRef(scene_root);
                self.scene.append_node_to(id, NodeRef(scene_root));
                layout_tree.add_child(layout_root, layer_layout).unwrap();
                let change: Arc<NoopChange> = Arc::new(NoopChange::new(root_id.0.into()));
                self.schedule_change(root_id, change, None);
            }
        }
        let change: Arc<NoopChange> = Arc::new(NoopChange::new(id.0.into()));
        self.schedule_change(id, change, None);
        id
    }
    pub fn scene_remove_layer(&self, layer: impl Into<Option<NodeRef>>) {
        let layer_id: Option<NodeRef> = layer.into();
        if let Some(layer_id) = layer_id {
            {
                if let Some(node) = self.scene.get_node(layer_id) {
                    let node = node.get();
                    self.scene.remove(layer_id);
                    let mut layout_tree = self.layout_tree.write().unwrap();
                    layout_tree.remove(node.layout_node_id).unwrap();
                }
            }
        }
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
    pub fn update(&self, dt: f32) -> bool {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);

        // 1.1 Update animations to the current timestamp
        let finished_animations = update_animations(self, timestamp.clone());
        // 1.2 Execute transactions using the updated animations
        let (mut updated_nodes, finished_transations, needs_redraw) = execute_transactions(self);

        // merge the updated nodes with the nodes that are part of the layout calculation
        let arena = self.scene.nodes.data();
        let arena = arena.read().unwrap();

        let nodes = arena.iter().filter_map(|node| {
            if node.is_removed() {
                return None;
            }
            let scene_node = node.get();
            // let layout = self.get_node_layout_style(scene_node.layout_node_id);
            // if layout.position != Position::Absolute {
            scene_node.insert_flags(RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT);
            scene_node.id()
            // } else {
            // None
            // }
        });
        updated_nodes.extend(nodes); // nodes are deduplicated in the update_nodes function
                                     // 2.0 update the layout tree using taffy
        update_layout_tree(self);

        // 3.0 update render nodes and trigger repaint
        update_nodes(self, updated_nodes);

        // 4.0 update render nodes and trigger repaint
        trigger_callbacks(self);

        // 5.0 cleanup the animations marked as done and
        // transactions already exectured
        cleanup_animations(self, finished_animations);

        cleanup_transactions(self, finished_transations);

        needs_redraw
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

        // println!("set_node_layout_size: {:?}", style.size);
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
                let handler = Arc::new(handler) as FnCallback;
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
}

impl Deref for NodeRef {
    type Target = TreeStorageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
