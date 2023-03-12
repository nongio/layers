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
pub mod storage;

use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use taffy::prelude::{Size, *};

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
    node::{ContainsPoint, DrawCacheManagement, RenderNode, RenderableFlags},
    scene::Scene,
    storage::{FlatStorage, FlatStorageId, TreeStorageId},
};

pub struct Timestamp(f64);

/// A trait for objects that can be exectuded by the engine.
pub trait Command {
    fn execute(&self, progress: f64) -> RenderableFlags;
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
/// The f64 is the current progress of the animation.
/// The bool is a flag that indicates if the animation is finished.
/// the progres can not be used to determine if the animation is finished
/// because the animation could be reversed or looped
#[derive(Clone)]
pub struct AnimationState(Animation, f64, bool);

type FnCallback = Arc<dyn 'static + Fn(f64) + Send + Sync>;

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
    pub fn update(&self, dt: f64) -> bool {
        self.engine.update(dt)
    }
    pub fn scene_add_layer(&self, layer: impl Into<Layers>) -> NodeRef {
        self.engine.scene_add_layer(layer)
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
    pub fn step_time(&self, dt: f64) {
        self.engine.step_time(dt)
    }
}

impl Default for LayersEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl Engine {
    fn new() -> Self {
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
        // self.scene_root =
        layer.set_id(id);
        id
    }

    pub fn scene_add_layer(&self, layer: impl Into<Layers>) -> NodeRef {
        let layer: Layers = layer.into();
        let mut layout = self.layout_tree.write().unwrap();

        let (id, layer_layout) = match layer {
            Layers::Layer(layer) => {
                let id = self
                    .scene
                    .add(layer.model.clone() as Arc<dyn RenderNode>, layer.layout);
                layer.set_id(id);
                (id, layer.layout)
            }
            Layers::TextLayer(text_layer) => {
                let id = self.scene.add(
                    text_layer.model.clone() as Arc<dyn RenderNode>,
                    text_layer.layout,
                );
                text_layer.set_id(id);

                (id, text_layer.layout)
            }
        };
        let mut scene_root = self.scene_root.write().unwrap();
        let layout_root = *self.layout_root.read().unwrap();
        if scene_root.is_none() {
            *scene_root = Some(id);
            layout.remove(layout_root).unwrap();
            *self.layout_root.write().unwrap() = layer_layout;
        } else {
            let scene_root = *scene_root.unwrap();
            self.scene.append_node_to(id, NodeRef(scene_root));
            layout.add_child(layout_root, layer_layout).unwrap();
        }

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
    pub fn step_time(&self, dt: f64) {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);
    }
    pub fn update(&self, dt: f64) -> bool {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);
        // println!("timestamp: {} {}", timestamp.0, dt);

        // supporting arrays for cleanup at a later stage
        let finished_animations = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));
        let finished_commands = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));

        // 1.1 Update animations to the current timestamp
        {
            let animations = self.animations.data();
            let mut animations = animations.write().unwrap();
            if animations.len() > 0 {
                animations.par_iter_mut().for_each_with(
                    finished_animations.clone(),
                    |done, (id, AnimationState(animation, value, finished))| {
                        (*value, *finished) = animation.value(timestamp.0);
                        if *finished {
                            done.clone().write().unwrap().push(*id);
                        }
                    },
                );
            }
        }
        // 1.2 Execute transactions using the updated animations

        let transactions = self.transactions.data();
        let mut transactions = transactions.write().unwrap();

        // TODO review this, we assume we should redraw the scene if there
        // are transactions to be executed
        let needs_redraw = transactions.len() > 0;

        let animations = &self.animations;
        let transaction_handlers = &self.transaction_handlers;
        let scene = self.scene.clone();
        transactions.par_iter().for_each_with(
            finished_commands.clone(),
            |done_commands, (id, command)| {
                let (progress, done) = command
                    .animation_id
                    .as_ref()
                    .map(|id| {
                        let update = animations
                            .get(&id.0)
                            .map(|AnimationState(_, value, done)| (value, done))
                            .unwrap_or((1.0, true));
                        if let Some(ch) = transaction_handlers.get(&id.0) {
                            let callbacks = &ch.on_update;
                            callbacks.iter().for_each(|callback| {
                                let callback = callback.clone();
                                callback(update.0);
                            });
                        }
                        update
                    })
                    .unwrap_or((1.0, true));

                let flags = command.change.execute(progress);

                if let Some(node) = scene.get_node(command.node_id.0) {
                    {
                        let node = node.get();
                        if flags.contains(RenderableFlags::NEEDS_LAYOUT) {
                            let bounds = node.model.bounds();
                            let size = crate::types::Size {
                                x: bounds.width,
                                y: bounds.height,
                            };
                            self.set_node_layout_size(node.layout_node, size);
                        }
                        if done {
                            // println!("done: {:?}", command.node_id.0);
                            node.remove_flags(RenderableFlags::ANIMATING);
                            // println!("flags: {:?}", flags);
                        }
                    }
                    // {
                    node.get().insert_flags(flags);
                    // }
                }
                if done {
                    done_commands.write().unwrap().push(*id);
                }
            },
        );

        let mut layout = self.layout_tree.write().unwrap();
        let layout_root = *self.layout_root.read().unwrap();
        let scene_root = *self.scene_root.read().unwrap().unwrap();
        let scene_root = self.scene.get_node(scene_root).unwrap();
        let scene_root = scene_root.get();
        let bounds = scene_root.model.bounds();
        if layout.dirty(layout_root).unwrap() {
            layout
                .compute_layout(
                    layout_root,
                    Size {
                        width: points(bounds.width as f32),
                        height: points(bounds.height as f32),
                    },
                )
                .unwrap();
        }
        // we are done writing to the layout tree, so we can
        // drop the lock
        drop(layout);

        let layout = self.layout_tree.read().unwrap();
        // iterate in parallel over the nodes and
        // 2., 3. repaint if necessary
        let arena = self.scene.nodes.data();
        let arena = arena.read().unwrap();
        arena.iter().for_each(|node| {
            let node = node.get();
            // println!("{:?}", node.layout_node);
            let l = layout.layout(node.layout_node).unwrap();

            node.layout_if_needed(l);
            node.repaint_if_needed();
        });

        // trigger animations callbacks
        {
            let animations = self.animations.data();
            let animations = animations.read().unwrap();
            animations
                .iter()
                .filter(|(_, AnimationState(_, _, finished))| *finished)
                .for_each(|(id, AnimationState(_animation, _, _))| {
                    if let Some(handler) = self.transaction_handlers.get(id) {
                        let callbacks = &handler.on_finish;
                        callbacks.iter().for_each(|callback| {
                            let callback = callback.clone();
                            callback(1.0);
                        });
                    }
                });
        }

        // cleanup the animations marked as done and
        // transactions already exectured
        let animations = self.animations.data();
        let mut animations = animations.write().unwrap();

        let animations_finished_to_remove = finished_animations.read().unwrap();
        for animation_id in animations_finished_to_remove.iter() {
            animations.remove(animation_id);
        }

        let commands_finished_to_remove = finished_commands.read().unwrap();
        let handlers = self.transaction_handlers.data();
        let mut handlers = handlers.write().unwrap();
        for command_id in commands_finished_to_remove.iter() {
            transactions.remove(command_id);
            handlers.remove(command_id);
        }

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
            width: points(size.x as f32),
            height: points(size.y as f32),
        };
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

    fn add_transaction_handler<F: Fn(f64) + Send + Sync + 'static>(
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

    pub fn on_start<F: Fn(f64) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        handler: F,
    ) {
        self.add_transaction_handler(transaction, TransactionEventType::Start, handler);
    }

    pub fn on_finish<F: Fn(f64) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        handler: F,
    ) {
        self.add_transaction_handler(transaction, TransactionEventType::Finish, handler);
    }

    pub fn on_update<F: Fn(f64) + Send + Sync + 'static>(
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
