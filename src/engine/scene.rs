use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::{
    num::NonZeroUsize,
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::types::Rectangle;

use super::{
    animations::{Animation, Easing, Transition},
    node::{DrawCacheManagement, Renderable, SceneNode},
    rendering::Drawable,
    storage::{FlatStorage, FlatStorageId, TreeStorage, TreeStorageId},
    AnimatedNodeChange, AnimationState, ChangeInvoker, CommandWithTransition, Engine, Timestamp,
};

pub struct Scene {
    pub nodes: TreeStorage<SceneNode>,
    pub root: TreeStorageId,
    transactions: FlatStorage<AnimatedNodeChange>,
    animations: FlatStorage<AnimationState>,
    pub timestamp: RwLock<Timestamp>,
}

#[derive(Clone)]
pub struct SceneRef(pub Arc<Scene>);

impl Engine for Scene {
    fn add_change(
        &self,
        target_id: TreeStorageId,
        change: Arc<dyn CommandWithTransition>,
    ) -> usize {
        self.add_change(target_id, change)
    }
}

impl From<SceneRef> for Arc<dyn Engine> {
    fn from(scene: SceneRef) -> Self {
        scene.0 as Arc<dyn Engine>
    }
}

impl SceneRef {
    pub fn new(scene: Scene) -> Self {
        Self(Arc::new(scene))
    }

    /// Creates a new node from a renderable model and adds it to the scene
    /// R can be converted into Arc<dyn Renderable>
    ///
    pub fn add_renderable<R: Into<Arc<dyn Renderable>>>(&self, renderable: R) -> TreeStorageId {
        let s: Arc<dyn Engine> = self.clone().into();
        let renderable: Arc<dyn Renderable> = renderable.into();
        let mut node = SceneNode::with_renderable(renderable.clone());
        let id = self.insert_node(&node);
        node.id = Some(id);
        node.scene = Some(self.clone());
        renderable.set_engine(s, id);
        id
    }
}
// implements deref for SceneRef

impl Deref for SceneRef {
    type Target = Arc<Scene>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Scene {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn create() -> SceneRef {
        SceneRef::new(Self::new())
    }
    pub fn root(&self) -> TreeStorageId {
        self.root
    }

    /// Add a new node to the scene by default append it to root
    pub fn insert_node(&self, node: &SceneNode) -> TreeStorageId {
        let id = self.nodes.insert(node.clone());

        let nodes = self.nodes.data();
        let mut nodes = nodes.write().unwrap();
        self.root.append(id, &mut nodes);
        id
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

        let node = self.nodes.get(target_id);
        if let Some(node) = node {
            let node = node.get();
            let node_change = AnimatedNodeChange {
                change,
                animation_id: aid,
                node: node.clone(),
            };
            let transation_id: NonZeroUsize = target_id.into();
            self.transactions
                .insert_with_id(node_change, transation_id.into())
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

        // let elapsed = self.time.elapsed();
        // self.time = Instant::now();
        // let _fps =
        // 1.0 / (elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0);

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
                command.node.flags.write().unwrap().insert(flags);

                if done {
                    done_commands.write().unwrap().push(*id);
                }
            },
        );

        // iterate in parallel over the nodes and
        // repaint if necessary
        let arena = self.nodes.data();
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

#[derive(Debug, Clone)]
struct Passthrough {}

impl Passthrough {
    pub fn new() -> Self {
        Self {}
    }
}

use skia_safe::{Canvas, Matrix};

impl Drawable for Passthrough {
    fn draw(&self, _: &mut Canvas) {}
    fn bounds(&self) -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
    fn transform(&self) -> Matrix {
        Matrix::new_identity()
    }
}
impl ChangeInvoker for Passthrough {
    fn set_engine(&self, _: Arc<dyn super::Engine>, _id: TreeStorageId) {}
}
impl Renderable for Passthrough {}

impl Default for Scene {
    fn default() -> Self {
        let model: Arc<dyn Renderable> = Arc::new(Passthrough::new());
        let root = SceneNode::with_renderable(model);
        let nodes = TreeStorage::new();
        let root_id = nodes.insert(root);

        Scene {
            nodes,
            root: root_id,
            animations: FlatStorage::new(),
            transactions: FlatStorage::new(),
            timestamp: RwLock::new(Timestamp(0.0)),
            // time: Instant::now(),
        }
    }
}
