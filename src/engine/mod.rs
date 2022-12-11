pub mod animations;
pub mod command;
pub mod node;
pub mod pointer;
pub mod rendering;
pub mod scene;
pub mod storage;

use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::{Arc, RwLock};

use crate::types::Point;

use self::{
    animations::{Animation, Easing, Transition},
    node::{DrawCacheManagement, RenderableFlags},
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
/// The state is the current progress of the animation.
/// The f64 is the current progress of the animation.
/// The bool is a flag that indicates if the animation is finished.
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

pub struct Engine {
    pub scene: Arc<Scene>,
    transactions: FlatStorage<AnimatedNodeChange>,
    animations: FlatStorage<AnimationState>,
    pub timestamp: RwLock<Timestamp>,
    transaction_handlers: FlatStorage<TransitionCallbacks>,
}
#[derive(Clone, Copy)]
pub struct TransactionRef(pub FlatStorageId);
#[derive(Clone)]
pub struct AnimationRef(FlatStorageId);
#[derive(Clone)]
pub struct NodeRef(pub TreeStorageId);
#[derive(Clone)]
pub struct HandlerRef(FlatStorageId);

impl Engine {
    fn new() -> Self {
        Default::default()
    }
    pub fn create() -> Arc<Self> {
        let new_engine = Self::new();
        let engine_handle = Arc::new(new_engine);

        engine_handle.scene.set_engine(engine_handle.clone());

        engine_handle
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

    pub fn add_change(
        &self,
        target_id: NodeRef,
        change: Arc<dyn CommandWithTransition>,
    ) -> TransactionRef {
        self.add_change_with_animation(target_id, change, None)
    }

    pub fn update(&self, dt: f64) -> bool {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);

        // supporting arrays for cleanup at a later stage
        let finished_animations = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));
        let finished_commands = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));

        // Update animations to the current timestamp
        {
            let animations = self.animations.data();

            animations.write().unwrap().par_iter_mut().for_each_with(
                finished_animations.clone(),
                |done, (id, AnimationState(animation, value, finished))| {
                    (*value, *finished) = animation.value(timestamp.0);
                    // TODO: add support for update callbacks
                    {}
                    if *finished {
                        done.clone().write().unwrap().push(*id);
                    }
                },
            );
        }
        // Execute transactions using the updated animations

        let transactions = self.transactions.data();
        let mut transactions = transactions.write().unwrap();

        // TODO review this, we assume we should redraw the scene if there
        // are transactions to be executed
        let needs_redraw = transactions.len() > 0;

        transactions.par_iter().for_each_with(
            finished_commands.clone(),
            |done_commands, (id, command)| {
                let (progress, done) = command
                    .animation_id
                    .as_ref()
                    .map(|id| {
                        let update = self
                            .animations
                            .get(&id.0)
                            .map(|AnimationState(_, value, done)| (value, done))
                            .unwrap_or((1.0, true));
                        if let Some(ch) = self.transaction_handlers.get(&id.0) {
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
                if let Some(node) = self.scene.get_node(command.node_id.0) {
                    node.get().insert_flags(flags);
                }

                if done {
                    done_commands.write().unwrap().push(*id);
                }
            },
        );

        // iterate in parallel over the nodes and
        // repaint if necessary
        let arena = self.scene.nodes.data();
        let arena = arena.read().unwrap();
        arena.par_iter().for_each(|node| {
            let node = node.get();
            node.layout_if_needed();
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

    pub fn layer_at(&self, point: Point) -> Option<NodeRef> {
        let arena = self.scene.nodes.data();
        let arena = arena.read().unwrap();
        let mut result = None;
        for node in arena.iter() {
            let node = node.get();
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
        let scene = Scene::create();

        Engine {
            scene,
            transactions: FlatStorage::new(),
            animations: FlatStorage::new(),
            timestamp: RwLock::new(Timestamp(0.0)),
            transaction_handlers: FlatStorage::new(),
        }
    }
}

/// A trait for objects that generates changes messages for an Engine
pub trait ChangeProducer {
    fn set_engine(&self, engine: Arc<Engine>, id: TreeStorageId);
}
