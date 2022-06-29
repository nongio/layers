pub mod animations;

use std::default::{Default};
use std::sync::{RwLock, Arc};
use std::time::Instant;
use indexmap::{IndexMap};
use rayon::iter::{ParallelIterator, IntoParallelRefMutIterator, IntoParallelRefIterator};
use skia_safe::{Picture};


use crate::easing::{Interpolable, interpolate};
use crate::layer::*;
use crate::skcache::render_layer_cache;

use self::animations::*;

pub struct Timestamp(f64);

#[derive(Debug)]
pub struct PropChange<T:Interpolable + Sync> {
    change: ValueChange<T>,
    animation_id: Option<usize>,
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
    Layer(ModelLayer, RenderLayer, SkiaCache),
}

pub struct Storage<V> {
    pub map: Arc<RwLock<IndexMap<usize,V>>>,
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
        let id = self.model_storage.insert(entity.clone());

        id
    }
    pub fn add_animation(&mut self, animation: Animation) -> usize {
        let id = self.animations_storage
            .insert(AnimationState(animation, 0.0, false));
        id
    }
    pub fn add_animation_from_transition(&mut self, transition: Transition<Easing>) -> usize {
        let start = self.timestamp.read().unwrap().0 + transition.delay;
        let id = self.add_animation(
            Animation { 
                start, 
                duration: transition.duration, 
                timing: transition.timing 
            },
        );
        id
    }
    pub fn add_change_with_animation(&mut self, change: ValueChanges, animation_id: Option<usize>) -> usize {

        let id:usize;
        let prop_change = match change {
            ValueChanges::Point(change) => {
                id = change.target.id;
                let aid = change.transition.map(|t| self.add_animation_from_transition(t)).or(animation_id);
                PropChanges::ChangePoint(
                    PropChange {
                        change,
                        animation_id: aid,
                    }
                )
            },
            ValueChanges::F64(change) => {
                let aid = change.transition.map(|t| self.add_animation_from_transition(t)).or(animation_id);
                id = change.target.id;
                PropChanges::ChangeF64(
                    PropChange {
                        change,
                        animation_id: aid,
                    }
                )
            },
            ValueChanges::BorderCornerRadius(change) => {
                let aid = change.transition.map(|t| self.add_animation_from_transition(t)).or(animation_id);
                id = change.target.id;
                PropChanges::ChangeBorderRadius(
                    PropChange {
                        change,
                        animation_id: aid,
                    }
                )
            },
            ValueChanges::PaintColor(change) => {
                let aid = change.transition.map(|t| self.add_animation_from_transition(t)).or(animation_id);
                id = change.target.id;
                PropChanges::ChangePaintColor(
                    PropChange {
                        change,
                        animation_id: aid,
                    }
                )
            },
        };
        let id = self.commands_storage.insert_with_id(prop_change, id);
              
        id
    }
    pub fn add_change(&mut self, change: ValueChanges) -> usize {
        
        self.add_change_with_animation(change, None)
        
    }
    pub fn add_changes(&mut self, changes: Vec<ValueChanges>, transition: Option<Transition<Easing>>) -> Vec<usize> {
        let animation_id = transition.map(|t| self.add_animation_from_transition(t));
        let mut ids = Vec::new();
        for vc in changes {
            let id = self.add_change_with_animation(vc, animation_id);
            ids.push(id);
        }
        ids
    }
    pub fn update(&mut self, dt: f64) {

        let mut timestamp = self.timestamp.write().unwrap();
        *timestamp = Timestamp(timestamp.0 + dt);

        let elapsed = self.time.elapsed();
        self.time = Instant::now();
        let fps = 1.0 / (elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
        self.fps = (self.fps + fps) / 2.0; // <-- smooth values to make them readable

        let done_animations = Arc::new(RwLock::new(Vec::<usize>::new()));
        let done_commands = Arc::new(RwLock::new(Vec::<usize>::new()));
        
        // Update animations
        {
        let animations = self.animations_storage.map.clone();

        animations.write().unwrap().par_iter_mut()
            .for_each_with(done_animations.clone(), |done, (id, AnimationState(animation, value, finished))| {
                (*value, *finished) = animation.value(timestamp.0);
                if *finished {
                    done.clone().write().unwrap().push(*id);
                }
            });
        }
        // Execute commands
        let cmd = self.commands_storage.map.clone();
            
        cmd.write().unwrap().par_iter().for_each_with(done_commands.clone(),|a, (id, command)| {
            match command {
                PropChanges::ChangePoint(command) => {
                    let (f, done) = command.animation_id.map(|id| 
                        self.animations_storage.map.read().unwrap().get(&id)
                            .map(|AnimationState(_, value, done)| {
                                (*value, *done)
                            }).unwrap_or((1.0, true))
                    ).unwrap_or((1.0, true));
                    
                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from, command.change.to, f);
                    if done {
                        a.write().unwrap().push(*id);
                    }            
                },
                PropChanges::ChangeF64(command) => {
                    let (f, done) = command.animation_id.map(|id| 
                        self.animations_storage.map.read().unwrap().get(&id)
                            .map(|AnimationState(_, value, done)| {
                                (*value, *done)
                            }).unwrap_or((1.0, true))
                    ).unwrap_or((1.0, true));
                    
                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from, command.change.to, f);
                    if done {
                        a.write().unwrap().push(*id);
                    }            
                },
                PropChanges::ChangeBorderRadius(command) => {
                    let (f, done) = command.animation_id.map(|id| 
                        self.animations_storage.map.read().unwrap().get(&id)
                            .map(|AnimationState(_, value, done)| {
                                (*value, *done)
                            }).unwrap_or((1.0, true))
                    ).unwrap_or((1.0, true));
                    
                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from, command.change.to, f);
                    if done {
                        a.write().unwrap().push(*id);
                    }            
                },
                PropChanges::ChangePaintColor(command) => {
                    let (f, done) = command.animation_id.map(|id| 
                        self.animations_storage.map.read().unwrap().get(&id)
                            .map(|AnimationState(_, value, done)| {
                                (*value, *done)
                            }).unwrap_or((1.0, true))
                    ).unwrap_or((1.0, true));
                    
                    *command.change.target.value.write().unwrap() =
                        interpolate(command.change.from.clone(), command.change.to.clone(), f);
                    if done {
                        a.write().unwrap().push(*id);
                    }            
                },
            }                
        });

        self.model_storage.map.clone().write().unwrap().par_iter_mut()
            .for_each(|(_, entity)| {
                match entity {
                    Entities::Layer(layer, render, cache) => {
                        *render = layer.render_layer();
                        cache.picture = render_layer_cache(render.clone());
                    },
                }
            });

        // cleanup
        for animation_id in done_animations.clone().read().unwrap().iter() {
            self.animations_storage.map.write().unwrap().remove(animation_id);
        }

        for command_id in done_commands.clone().read().unwrap().iter() {
            let cmd = self.commands_storage.map.clone();
            let mut indexmap = cmd.write().unwrap();
            indexmap.remove(command_id);
        }

    }
}


pub fn setup_ecs() -> State {
    let mut state = State::new();

    // for 10 times
    for _ in 0..500 {
        let model = ModelLayer::new();

        let change = model.position().to(Point {x: 300.0, y: 300.0}, Some(Transition {
            delay: 0.0,
            duration: 1.0,
            timing: Easing::default(),
        }));
        state.add_entity(Entities::Layer(model.clone(), model.render_layer(), SkiaCache{
            picture: None,
        }));

        state.add_change(change);
    }

    

    return state;
}