pub mod animations;
pub mod backend;
pub mod command;
pub mod node;
pub mod rendering;
pub mod scene;
pub mod storage;

use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::{Arc, RwLock};

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
    animation_id: Option<FlatStorageId>,
    node_id: TreeStorageId,
}

#[derive(Clone)]
pub struct AnimationState(Animation, f64, bool);

pub struct Engine {
    pub scene: Arc<Scene>,
    transactions: FlatStorage<AnimatedNodeChange>,
    animations: FlatStorage<AnimationState>,
    pub timestamp: RwLock<Timestamp>,
}

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
    pub fn create_animation_from_transition(
        &self,
        transition: Transition<Easing>,
    ) -> FlatStorageId {
        
        let start = self.timestamp.read().unwrap().0 + transition.delay;

        self.add_animation(Animation {
            start,
            duration: transition.duration,
            timing: transition.timing,
        })
    }

    pub fn add_animation(&self, animation: Animation) -> FlatStorageId {
        self.animations
            .insert(AnimationState(animation, 0.0, false))
    }

    pub fn add_change_with_animation(
        &self,
        target_id: TreeStorageId,
        change: Arc<dyn CommandWithTransition>,
        animation_id: Option<FlatStorageId>,
    ) -> FlatStorageId {
        let aid = change
            .transition()
            .map(|t| self.create_animation_from_transition(t))
            .or(animation_id);

        let node = self.scene.nodes.get(target_id);
        if node.is_some() {
            let transaction_id: usize = change.value_id();
            let node_change = AnimatedNodeChange {
                change,
                animation_id: aid,
                node_id: target_id,
            };
            self.transactions
                .insert_with_id(node_change, transaction_id)
        } else {
            0
        }
    }

    pub fn add_change(
        &self,
        target_id: TreeStorageId,
        change: Arc<dyn CommandWithTransition>,
    ) -> usize {
        self.add_change_with_animation(target_id, change, None)
    }

    pub fn update(&self, dt: f64) -> bool {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);

        // supporting arrays for cleanup at a later stage
        let done_animations = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));
        let done_commands = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));

        // Update animations to the current timestamp
        {
            let animations = self.animations.data();

            animations.write().unwrap().par_iter_mut().for_each_with(
                done_animations.clone(),
                |done, (id, AnimationState(animation, value, finished))| {
                    (*value, *finished) = animation.value(timestamp.0);

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
            done_commands.clone(),
            |done_commands, (id, command)| {
                let (progress, done) = command
                    .animation_id
                    .map(|id| {
                        self.animations
                            .get(&id)
                            .map(|AnimationState(_, value, done)| (value, done))
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                let flags = command.change.execute(progress);
                if let Some(node) = self.scene.get_node(command.node_id) {
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

        // cleanup the animations marked as done and
        // transactions already exectured
        let animations = self.animations.data();
        let mut animations = animations.write().unwrap();

        let animations_done_to_remove = done_animations.read().unwrap();
        for animation_id in animations_done_to_remove.iter() {
            animations.remove(animation_id);
        }

        let commands_done_to_remove = done_commands.read().unwrap();
        for command_id in commands_done_to_remove.iter() {
            transactions.remove(command_id);
        }

        needs_redraw
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
        }
    }
}

/// A trait for objects that generates changes messages for an Engine
pub trait ChangeProducer {
    fn set_engine(&self, engine: Arc<Engine>, id: TreeStorageId);
}
