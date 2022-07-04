pub mod animations;
pub mod entities;
pub mod storage;

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::easing::{interpolate, Interpolable};
use crate::layer::*;

use self::animations::*;
use self::entities::*;
use self::storage::*;

pub struct Timestamp(f64);

#[derive(Clone, Debug)]
pub struct PropChange<T: Interpolable + Sync> {
    change: ValueChange<T>,
    animation_id: Option<usize>,
    model_id: usize,
    needs_repaint: bool,
    target_needs_repaint: Arc<AtomicBool>,
}

#[derive(Clone, Debug)]
pub enum PropChanges {
    ChangePoint(PropChange<Point>),
    ChangeF64(PropChange<f64>),
    ChangeBorderRadius(PropChange<BorderRadius>),
    ChangePaintColor(PropChange<PaintColor>),
}

#[derive(Clone)]
pub struct AnimationState(Animation, f64, bool);

pub struct State {
    pub model_storage: Storage<Entities>,
    commands_storage: Storage<PropChanges>,
    animations_storage: Storage<AnimationState>,

    pub root: Entities,

    timestamp: RwLock<Timestamp>,
    time: Instant,
    pub fps: f64,
}

impl State {
    pub fn new() -> Self {
        let mut state = State {
            model_storage: Storage::new(),
            animations_storage: Storage::new(),
            commands_storage: Storage::new(),
            root: Entities::new_root(),
            timestamp: RwLock::new(Timestamp(0.0)),
            time: Instant::now(),
            fps: 0.0,
        };

        state.model_storage.insert_with_id(Entities::new_root(), 0);
        state
    }
    pub fn get_entities(&self) -> Arc<RwLock<IndexMap<usize, Entities>>> {
        self.model_storage.map.clone()
    }
    pub fn add_entity(&mut self, entity: Entities) {
        match entity {
            Entities::Root { .. } => (),
            Entities::Layer { ref parent, .. } => {
                if parent.read().unwrap().is_none() {
                    self.root.add_child(&mut entity.clone());
                }
                self.model_storage
                    .insert_with_id(entity.clone(), entity.id());
            }
        }
    }
    pub fn add_animation(&mut self, animation: Animation) -> usize {
        let id = self
            .animations_storage
            .insert(AnimationState(animation, 0.0, false));
        id
    }
    pub fn add_animation_from_transition(&mut self, transition: Transition<Easing>) -> usize {
        let start = self.timestamp.read().unwrap().0 + transition.delay;
        let id = self.add_animation(Animation {
            start,
            duration: transition.duration,
            timing: transition.timing,
        });
        id
    }

    pub fn add_animation_for_change<T: Interpolable + Sync>(
        &mut self,
        change: ValueChange<T>,
        default_animation: Option<usize>,
    ) -> Option<usize> {
        change
            .transition
            .map(|t| self.add_animation_from_transition(t))
            .or(default_animation)
    }

    pub fn add_change_with_animation(
        &mut self,
        change: ModelChanges,
        animation_id: Option<usize>,
    ) -> usize {
        let result: Option<(usize, PropChanges)> = match change {
            ModelChanges::Point(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = self.add_animation_for_change(change.clone(), animation_id);
                let entity = self.model_storage.get(mid).unwrap().clone();
                match entity {
                    Entities::Layer { needs_paint: a, .. } => Some((
                        id,
                        PropChanges::ChangePoint(PropChange {
                            change,
                            animation_id: aid,
                            model_id: mid,
                            needs_repaint,
                            target_needs_repaint: a.clone(),
                        }),
                    )),
                    _ => None,
                }
            }
            ModelChanges::F64(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = self.add_animation_for_change(change.clone(), animation_id);
                let entity = self.model_storage.get(mid).unwrap().clone();
                match entity {
                    Entities::Layer { needs_paint: a, .. } => Some((
                        id,
                        PropChanges::ChangeF64(PropChange {
                            change,
                            animation_id: aid,
                            model_id: mid,
                            needs_repaint,
                            target_needs_repaint: a.clone(),
                        }),
                    )),
                    _ => None,
                }
            }
            ModelChanges::BorderCornerRadius(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = self.add_animation_for_change(change.clone(), animation_id);
                let entity = self.model_storage.get(mid).unwrap().clone();
                match entity {
                    Entities::Layer { needs_paint: a, .. } => Some((
                        id,
                        PropChanges::ChangeBorderRadius(PropChange {
                            change,
                            animation_id: aid,
                            model_id: mid,
                            needs_repaint,
                            target_needs_repaint: a.clone(),
                        }),
                    )),
                    _ => None,
                }
            }
            ModelChanges::PaintColor(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = self.add_animation_for_change(change.clone(), animation_id);
                let entity = self.model_storage.get(mid).unwrap().clone();
                match entity {
                    Entities::Layer { needs_paint: a, .. } => Some((
                        id,
                        PropChanges::ChangePaintColor(PropChange {
                            change,
                            animation_id: aid,
                            model_id: mid,
                            needs_repaint,
                            target_needs_repaint: a.clone(),
                        }),
                    )),
                    _ => None,
                }
            }
        };
        match result {
            Some((id, change)) => {
                self.commands_storage.insert_with_id(change, id);
                id
            }
            None => 0,
        }
    }
    pub fn add_change(&mut self, change: ModelChanges) -> usize {
        self.add_change_with_animation(change, None)
    }
    pub fn add_changes(
        &mut self,
        changes: Vec<ModelChanges>,
        transition: Option<Transition<Easing>>,
    ) -> Vec<usize> {
        let animation_id = transition.map(|t| self.add_animation_from_transition(t));
        let mut ids = Vec::new();
        for vc in changes {
            let id = self.add_change_with_animation(vc, animation_id);
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
            let animations = self.animations_storage.map.clone();

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
        let cmd = self.commands_storage.map.clone();
        let needs_redraw = cmd.read().unwrap().len() > 0;

        cmd.write()
            .unwrap()
            .par_iter()
            .for_each_with(done_commands.clone(), |a, (id, command)| match command {
                PropChanges::ChangePoint(command) => {
                    let (f, done) = command
                        .animation_id
                        .map(|id| {
                            self.animations_storage
                                .map
                                .read()
                                .unwrap()
                                .get(&id)
                                .map(|AnimationState(_, value, done)| (*value, *done))
                                .unwrap_or((1.0, true))
                        })
                        .unwrap_or((1.0, true));

                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from, command.change.to, f);
                    if command.needs_repaint {
                        command.target_needs_repaint.store(true, Ordering::Relaxed);
                    }

                    if done {
                        a.write().unwrap().push(*id);
                    }
                }
                PropChanges::ChangeF64(command) => {
                    let (f, done) = command
                        .animation_id
                        .map(|id| {
                            self.animations_storage
                                .map
                                .read()
                                .unwrap()
                                .get(&id)
                                .map(|AnimationState(_, value, done)| (*value, *done))
                                .unwrap_or((1.0, true))
                        })
                        .unwrap_or((1.0, true));

                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from, command.change.to, f);

                    if command.needs_repaint {
                        command.target_needs_repaint.store(true, Ordering::Relaxed);
                    }

                    if done {
                        a.write().unwrap().push(*id);
                    }
                }
                PropChanges::ChangeBorderRadius(command) => {
                    let (f, done) = command
                        .animation_id
                        .map(|id| {
                            self.animations_storage
                                .map
                                .read()
                                .unwrap()
                                .get(&id)
                                .map(|AnimationState(_, value, done)| (*value, *done))
                                .unwrap_or((1.0, true))
                        })
                        .unwrap_or((1.0, true));

                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from, command.change.to, f);

                    if command.needs_repaint {
                        command.target_needs_repaint.store(true, Ordering::Relaxed);
                    }

                    if done {
                        a.write().unwrap().push(*id);
                    }
                }
                PropChanges::ChangePaintColor(command) => {
                    let (f, done) = command
                        .animation_id
                        .map(|id| {
                            self.animations_storage
                                .map
                                .read()
                                .unwrap()
                                .get(&id)
                                .map(|AnimationState(_, value, done)| (*value, *done))
                                .unwrap_or((1.0, true))
                        })
                        .unwrap_or((1.0, true));

                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from.clone(), command.change.to.clone(), f);

                    if command.needs_repaint {
                        command.target_needs_repaint.store(true, Ordering::Relaxed);
                    }

                    if done {
                        a.write().unwrap().push(*id);
                    }
                }
            });

        // repaint
        self.model_storage
            .map
            .clone()
            .write()
            .unwrap()
            .par_iter_mut()
            .for_each(|(_, entity)| {
                entity.repaint_if_needed();
            });

        // cleanup
        for animation_id in done_animations.clone().read().unwrap().iter() {
            self.animations_storage
                .map
                .write()
                .unwrap()
                .remove(animation_id);
        }

        for command_id in done_commands.clone().read().unwrap().iter() {
            let cmd = self.commands_storage.map.clone();
            let mut indexmap = cmd.write().unwrap();
            indexmap.remove(command_id);
        }
        needs_redraw
    }
}

pub fn setup_ecs() -> State {
    let mut state = State::new();

    let model = ModelLayer::new();
    let mut entity = Entities::new_layer(model.clone());
    state.add_entity(entity.clone());
    state.add_change(model.size_to(Point { x: 100.0, y: 100.0 }, None));
    state.add_change(model.position_to(Point { x: 100.0, y: 100.0 }, None));
    state.add_change(model.background_color_to(
        PaintColor::Solid {
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        },
        None,
    ));
    let child_entity = Entities::new_layer(ModelLayer::new());

    entity.add_child(&mut child_entity.clone());
    state.add_entity(child_entity);

    let entity = Entities::new_layer(ModelLayer::new());

    state.add_entity(entity.clone());
    return state;
}
