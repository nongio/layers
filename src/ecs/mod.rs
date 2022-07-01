pub mod animations;

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use skia_safe::Picture;
use std::default::Default;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::easing::{interpolate, Interpolable};
use crate::layer::*;
use crate::skcache::render_layer_cache;

use self::animations::*;

pub struct Timestamp(f64);

#[derive(Debug)]
pub struct PropChange<T: Interpolable + Sync> {
    change: ValueChange<T>,
    animation_id: Option<usize>,
    model_id: usize,
    needs_repaint: bool,
    target_needs_repaint: Arc<AtomicBool>,
}

#[derive(Debug)]
pub enum PropChanges {
    ChangePoint(PropChange<Point>),
    ChangeF64(PropChange<f64>),
    ChangeBorderRadius(PropChange<BorderRadius>),
    ChangePaintColor(PropChange<PaintColor>),
}

pub struct AnimationState(Animation, f64, bool);

#[derive(Clone)]
pub struct SkiaCache {
    pub picture: Option<Picture>,
}

#[derive(Clone)]
pub enum Entities {
    Layer(ModelLayer, RenderLayer, SkiaCache, Arc<AtomicBool>),
}

pub struct Storage<V> {
    pub map: Arc<RwLock<IndexMap<usize, V>>>,
    index: RwLock<u32>,
}
impl<V> Storage<V> {
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(IndexMap::<usize, V>::new())),
            index: RwLock::new(0),
        }
    }
    pub fn insert(&mut self, value: V) -> usize {
        let mut index = self.index.write().unwrap();
        *index = *index + 1;
        let id = *index as usize;
        self.map.write().unwrap().insert(id, value);

        id
    }
    pub fn insert_with_id(&mut self, value: V, id: usize) -> usize {
        self.map.write().unwrap().insert(id, value);
        id
    }
}
pub struct State {
    model_storage: Storage<Entities>,
    commands_storage: Storage<PropChanges>,
    animations_storage: Storage<AnimationState>,
    timestamp: RwLock<Timestamp>,
    time: Instant,
    pub fps: f64,
}

impl State {
    fn new() -> Self {
        let state = State {
            model_storage: Storage::new(),
            animations_storage: Storage::new(),
            commands_storage: Storage::new(),
            timestamp: RwLock::new(Timestamp(0.0)),
            time: Instant::now(),
            fps: 0.0,
        };
        state
    }
    pub fn get_entities(&self) -> Arc<RwLock<IndexMap<usize, Entities>>> {
        self.model_storage.map.clone()
    }
    pub fn add_entity(&mut self, entity: Entities) -> usize {
        let id = match entity {
            Entities::Layer(ref model, _, _, _) => {
                let mid = model.id;
                self.model_storage.insert_with_id(entity, mid);
                mid
            }
        };
        id
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

    pub fn add_change_with_animation(
        &mut self,
        change: ModelChanges,
        animation_id: Option<usize>,
    ) -> usize {
        let (target_id, prop_change) = match change {
            ModelChanges::Point(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = change
                    .transition
                    .map(|t| self.add_animation_from_transition(t))
                    .or(animation_id);
                let Entities::Layer(_, _, _, a) = self
                    .model_storage
                    .map
                    .read()
                    .unwrap()
                    .get(&mid)
                    .unwrap()
                    .clone();

                (
                    id,
                    PropChanges::ChangePoint(PropChange {
                        change,
                        animation_id: aid,
                        model_id: mid,
                        needs_repaint,
                        target_needs_repaint: a.clone(),
                    }),
                )
            }
            ModelChanges::F64(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = change
                    .transition
                    .map(|t| self.add_animation_from_transition(t))
                    .or(animation_id);
                let Entities::Layer(_, _, _, a) = self
                    .model_storage
                    .map
                    .read()
                    .unwrap()
                    .get(&mid)
                    .unwrap()
                    .clone();

                (
                    id,
                    PropChanges::ChangeF64(PropChange {
                        change,
                        animation_id: aid,
                        model_id: mid,
                        needs_repaint,
                        target_needs_repaint: a.clone(),
                    }),
                )
            }
            ModelChanges::BorderCornerRadius(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = change
                    .transition
                    .map(|t| self.add_animation_from_transition(t))
                    .or(animation_id);
                let Entities::Layer(_, _, _, a) = self
                    .model_storage
                    .map
                    .read()
                    .unwrap()
                    .get(&mid)
                    .unwrap()
                    .clone();

                (
                    id,
                    PropChanges::ChangeBorderRadius(PropChange {
                        change,
                        animation_id: aid,
                        model_id: mid,
                        needs_repaint,
                        target_needs_repaint: a.clone(),
                    }),
                )
            }
            ModelChanges::PaintColor(mid, change, needs_repaint) => {
                let id = change.target.id;
                let aid = change
                    .transition
                    .map(|t| self.add_animation_from_transition(t))
                    .or(animation_id);
                let Entities::Layer(_, _, _, a) = self
                    .model_storage
                    .map
                    .read()
                    .unwrap()
                    .get(&mid)
                    .unwrap()
                    .clone();
                (
                    id,
                    PropChanges::ChangePaintColor(PropChange {
                        change,
                        animation_id: aid,
                        model_id: mid,
                        needs_repaint,
                        target_needs_repaint: a.clone(),
                    }),
                )
            }
        };
        let id = self.commands_storage.insert_with_id(prop_change, target_id);

        id
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
            .for_each(|(_, entity)| match entity {
                Entities::Layer(layer, render, cache, needs_repaint) => {
                    *render = layer.render_layer();
                    let nr = &**needs_repaint;
                    let needs_repaint = nr.swap(false, Ordering::Relaxed);
                    if needs_repaint {
                        cache.picture = render_layer_cache(render.clone());
                    }
                }
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

    // for 10 times
    for _ in 0..50 {
        let model = ModelLayer::new();

        let change = model.position_to(
            Point { x: 300.0, y: 300.0 },
            Some(Transition {
                delay: 0.0,
                duration: 1.0,
                timing: Easing::default(),
            }),
        );
        state.add_entity(Entities::Layer(
            model.clone(),
            model.render_layer(),
            SkiaCache { picture: None },
            Arc::new(AtomicBool::new(true)),
        ));

        state.add_change(change);
    }

    return state;
}
