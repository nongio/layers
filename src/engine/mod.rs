#![allow(dead_code)]

//! Contains all the logic for running the animations, execute the scheduled changes
//! to the models, and prepare the render tree
//! The scene is drawn in 4 stages:
//! - The *layout* step calculates the dimensions and position of a node and generates a transformation Matrix
//! - The *draw* step generates a displaylist
//! - The *render* step uses the displaylist to generate a texture of the node
//! - The *compose* step generates the final image using the textures
pub mod animations;
pub mod command;
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
    layers::Layers,
    types::Point,
};

use self::{
    animations::{Animation, Easing, Transition},
    command::NoopChange,
    node::{ContainsPoint, RenderNode, RenderableFlags},
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

/// A trait for objects that contain a transition.
pub trait WithTransition {
    fn transition(&self) -> Option<Transition<Easing>>;
}

/// A group trait for commands that contain a transition.
pub trait CommandWithTransition: Command + WithTransition + Send + Sync {}

#[derive(Clone)]
pub struct AnimatedNodeChange {
    pub change: Arc<dyn CommandWithTransition>,
    animation_id: Option<AnimationRef>,
    node_id: NodeRef,
}

/// A struct that contains the state of an animation.
/// The f32 is the current progress of the animation.
/// The bool is a flag that indicates if the animation is finished.
/// the progres can not be used to determine if the animation is finished
/// because the animation could be reversed or looped
#[derive(Clone)]
pub struct AnimationState(Animation, f32, bool);

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

pub(crate) struct Engine {
    pub scene: Arc<Scene>,
    scene_root: RwLock<Option<NodeRef>>,
    transactions: FlatStorage<AnimatedNodeChange>,
    animations: FlatStorage<AnimationState>,
    pub timestamp: RwLock<Timestamp>,
    transaction_handlers: FlatStorage<TransitionCallbacks>,
    layout_tree: RwLock<Taffy>,
    layout_root: RwLock<taffy::node::Node>,
}
#[derive(Clone, Copy)]
pub struct TransactionRef(pub FlatStorageId);
#[derive(Clone)]
pub struct AnimationRef(FlatStorageId);

/// An identifier for a node in the three storage
#[derive(Clone, Copy)]
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
    pub fn new() -> Self {
        Self {
            engine: Engine::create(),
        }
    }

    pub fn new_layer(&self) -> Layer {
        let model = Arc::new(ModelLayer::default());

        let mut lt = self.engine.layout_tree.write().unwrap();

        let layout = lt
            .new_leaf(Style {
                ..Default::default()
            })
            .unwrap();

        Layer {
            engine: self.engine.clone(),
            model,
            id: Arc::new(RwLock::new(None)),
            layout,
        }
    }
    pub fn update(&self, dt: f32) -> bool {
        self.engine.update(dt)
    }
    pub fn scene_add_layer(&self, layer: impl Into<Layers>) -> NodeRef {
        self.engine.scene_add_layer(layer, None)
    }
    pub fn scene_add_layer_to(&self, layer: impl Into<Layers>, parent: Option<NodeRef>) -> NodeRef {
        self.engine.scene_add_layer(layer, parent)
    }
    pub fn scene_set_root(&self, layer: impl Into<Layers>) -> NodeRef {
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

impl Default for LayersEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl Engine {
    fn new() -> Self {
        // rayon::ThreadPoolBuilder::new()
        //     .num_threads(2)
        //     .build_global()
        //     .unwrap();
        Default::default()
    }
    pub fn create() -> Arc<Self> {
        let new_engine = Self::new();
        Arc::new(new_engine)
    }
    pub fn scene_set_root(&self, layer: impl Into<Layers>) -> NodeRef {
        let layer: Layers = layer.into();
        let renderable = match layer.clone() {
            Layers::Layer(layer) => layer.model as Arc<dyn RenderNode>,
            Layers::TextLayer(text_layer) => text_layer.model as Arc<dyn RenderNode>,
        };
        let id = self.scene.add(renderable, layer.layout_node());
        layer.set_id(id);
        id
    }

    pub fn scene_add_layer(&self, layer: impl Into<Layers>, parent: Option<NodeRef>) -> NodeRef {
        let layer: Layers = layer.into();
        let mut layout_tree = self.layout_tree.write().unwrap();

        let (id, layer_layout) = match layer {
            Layers::Layer(layer) => {
                let id = self.scene.append(
                    parent,
                    layer.model.clone() as Arc<dyn RenderNode>,
                    layer.layout,
                );
                layer.set_id(id);
                (id, layer.layout)
            }
            Layers::TextLayer(text_layer) => {
                let id = self.scene.append(
                    parent,
                    text_layer.model.clone() as Arc<dyn RenderNode>,
                    text_layer.layout,
                );
                text_layer.set_id(id);

                (id, text_layer.layout)
            }
        };

        if let Some(parent) = parent {
            let parent_layout = self.scene.get_node(parent).unwrap().get().layout_node;
            self.scene.append_node_to(id, parent);
            layout_tree.add_child(parent_layout, layer_layout).unwrap();
        } else {
            let mut scene_root = self.scene_root.write().unwrap();
            let layout_root = *self.layout_root.read().unwrap();
            if scene_root.is_none() {
                *scene_root = Some(id);
                layout_tree.remove(layout_root).unwrap();
                *self.layout_root.write().unwrap() = layer_layout;
            } else {
                let scene_root = *scene_root.unwrap();
                self.scene.append_node_to(id, NodeRef(scene_root));
                layout_tree.add_child(layout_root, layer_layout).unwrap();
            }
        }
        let change = Arc::new(NoopChange::new(id.0.into()));
        self.schedule_change(id, change);
        id
    }

    pub fn create_animation_from_transition(&self, transition: Transition<Easing>) -> AnimationRef {
        let start = self.timestamp.read().unwrap().0 + transition.delay;

        self.add_animation(Animation {
            start,
            duration: transition.duration,
            timing: transition.timing,
        })
    }

    pub fn add_animation(&self, animation: Animation) -> AnimationRef {
        AnimationRef(
            self.animations
                .insert(AnimationState(animation, 0.0, false)),
        )
    }

    pub fn add_change_with_animation(
        &self,
        target_id: NodeRef,
        change: Arc<dyn CommandWithTransition>,
        animation_id: Option<FlatStorageId>,
    ) -> TransactionRef {
        let aid = change
            .transition()
            .map(|t| self.create_animation_from_transition(t).0)
            .or(animation_id)
            .map(AnimationRef);

        let node = self.scene.nodes.get(target_id.0);
        if node.is_some() {
            let transaction_id: usize = change.value_id();
            let node_change = AnimatedNodeChange {
                change,
                animation_id: aid,
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

    pub fn schedule_change(
        &self,
        target_id: NodeRef,
        change: Arc<dyn CommandWithTransition>,
    ) -> TransactionRef {
        self.add_change_with_animation(target_id, change, None)
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
        let (updated_nodes, finished_transations, needs_redraw) = execute_transactions(self);

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
            width: points(size.x),
            height: points(size.y),
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

impl Default for Engine {
    fn default() -> Self {
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

        let scene = Scene::create();
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
}

impl Deref for NodeRef {
    type Target = TreeStorageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
