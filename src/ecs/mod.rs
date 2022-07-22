pub mod animations;
pub mod entities;
pub mod storage;

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::easing::{interpolate, Interpolable};
use crate::layers::layer::ModelLayer;
use crate::layers::*;

use self::animations::*;
use self::entities::*;
use self::storage::*;

pub struct Timestamp(f64);

pub trait ExecutableChange {
    fn execute(&self, progress: f64) -> bool;
    // fn animation_id(&self) -> Option<usize>;
    fn id(&self) -> usize;
}
pub trait AnimatedChange: ExecutableChange + ChangeWithTransition + Send + Sync {}

#[derive(Clone)]

pub struct Transaction {
    pub change: Arc<dyn AnimatedChange>,
    animation_id: Option<usize>,
    entity: Entities,
}

impl<T: Interpolable + Sync + Clone + Sized + 'static> ExecutableChange for ModelChange<T> {
    fn execute(&self, progress: f64) -> bool {
        let ModelChange {
            value_change,
            need_repaint,
            ..
        } = &self;
        *value_change.target.value.write().unwrap() =
            interpolate(value_change.from.clone(), value_change.to.clone(), progress);

        *need_repaint
    }

    fn id(&self) -> usize {
        let ModelChange { id: mid, .. } = self;
        *mid
    }
}

impl<T: Interpolable + Sync + Send + Clone + Sized + 'static> AnimatedChange for ModelChange<T> {}

#[derive(Clone)]
pub struct AnimationState(Animation, f64, bool);

pub struct State {
    pub entities_storage: Storage<Entities>,
    transactions_storage: Storage<Transaction>,
    animations_storage: Storage<AnimationState>,

    pub root: Arc<RwLock<Entities>>,

    timestamp: RwLock<Timestamp>,
    time: Instant,
    pub fps: f64,
}

impl State {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn get_entities(&self) -> Arc<RwLock<IndexMap<usize, Entities>>> {
        self.entities_storage.data.clone()
    }
    pub fn add_entity(&mut self, entity: Entities) {
        match entity {
            Entities::Root { .. } => (),
            Entities::Layer { ref parent, .. } => {
                if parent.read().unwrap().is_none() {
                    self.root.write().unwrap().add_child(&mut entity.clone());
                }
                self.entities_storage
                    .insert_with_id(entity.clone(), entity.id());
            }
        }
    }
    pub fn add_layer(&mut self, layer: ModelLayer) -> Entities {
        let layer_entity = Entities::Layer {
            model: Arc::new(layer),
            cache: Arc::new(RwLock::new(SkiaCache { picture: None })),
            needs_paint: Arc::new(AtomicBool::new(true)),
            parent: Arc::new(RwLock::new(None)),
            children: Arc::new(RwLock::new(vec![])),
        };

        self.add_entity(layer_entity.clone());
        layer_entity
    }
    pub fn add_animation(&mut self, animation: Animation) -> usize {
        self.animations_storage
            .insert(AnimationState(animation, 0.0, false))
    }
    pub fn add_animation_from_transition(&mut self, transition: Transition<Easing>) -> usize {
        let start = self.timestamp.read().unwrap().0 + transition.delay;

        self.add_animation(Animation {
            start,
            duration: transition.duration,
            timing: transition.timing,
        })
    }

    pub fn add_change_with_animation(
        &mut self,
        change: Arc<dyn AnimatedChange>,
        animation_id: Option<usize>,
    ) -> usize {
        let mid = change.id();
        let value_target_id = change.value_change_id();

        let aid = change
            .transition()
            .map(|t| self.add_animation_from_transition(t))
            .or(animation_id);

        let entity = self.entities_storage.get(mid);
        if let Some(entity) = entity {
            let ec = Transaction {
                change,
                animation_id: aid,
                entity,
            };

            self.transactions_storage
                .insert_with_id(ec, value_target_id);
            value_target_id
        } else {
            0
        }
    }

    pub fn add_change(&mut self, change: Arc<dyn AnimatedChange>) -> usize {
        self.add_change_with_animation(change, None)
    }
    pub fn add_changes(
        &mut self,
        changes: Vec<Arc<dyn AnimatedChange>>,
        transition: Option<Transition<Easing>>,
    ) -> Vec<usize> {
        let animation_id = transition.map(|t| self.add_animation_from_transition(t));
        let mut ids = Vec::new();
        for mc in changes {
            let id = self.add_change_with_animation(mc, animation_id);
            ids.push(id);
        }
        ids
    }
    pub fn update(&mut self, dt: f64) -> bool {
        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);

        let elapsed = self.time.elapsed();
        self.time = Instant::now();
        let fps =
            1.0 / (elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
        self.fps = (self.fps + fps) / 2.0; // <-- smooth values to make them readable

        let done_animations = Arc::new(RwLock::new(Vec::<usize>::new()));
        let done_commands = Arc::new(RwLock::new(Vec::<usize>::new()));

        // Update animations
        {
            let animations = self.animations_storage.data.clone();

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
        // Execute commands
        let cmd = self.transactions_storage.data.clone();
        let needs_redraw = cmd.read().unwrap().len() > 0;

        cmd.read()
            .unwrap()
            .par_iter()
            .for_each_with(done_commands.clone(), |a, (_, command)| {
                let (progress, done) = command
                    .animation_id
                    .map(|id| {
                        self.animations_storage
                            .data
                            .read()
                            .unwrap()
                            .get(&id)
                            .map(|AnimationState(_, value, done)| (*value, *done))
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                let repaint = command.change.execute(progress);
                command.entity.set_need_repaint(repaint);

                if done {
                    a.write().unwrap().push(command.change.id());
                }
            });

        // repaint
        self.entities_storage
            .data
            .clone()
            .read()
            .unwrap()
            .par_iter()
            .for_each(|(_, entity)| {
                entity.repaint_if_needed();
            });

        // cleanup
        for animation_id in done_animations.read().unwrap().iter() {
            self.animations_storage
                .data
                .write()
                .unwrap()
                .remove(animation_id);
        }

        for command_id in done_commands.read().unwrap().iter() {
            let cmd = self.transactions_storage.data.clone();
            let mut indexmap = cmd.write().unwrap();
            indexmap.remove(command_id);
        }
        needs_redraw
    }
}

impl Default for State {
    fn default() -> Self {
        let mut state = State {
            entities_storage: Storage::new(),
            animations_storage: Storage::new(),
            transactions_storage: Storage::new(),
            root: Arc::new(RwLock::new(Entities::new_root())),
            timestamp: RwLock::new(Timestamp(0.0)),
            time: Instant::now(),
            fps: 0.0,
        };

        state
            .entities_storage
            .insert_with_id(Entities::new_root(), 0);
        state
    }
}

pub fn setup_ecs() -> State {
    State::new()
}
