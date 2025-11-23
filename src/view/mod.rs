//! Helper struct to build complex layer hierarchies that change over time
//!
//! It is a wrapper around a state S, that can change overtime,
//! and a render function that generates a LayerTree from a given state.
//!
//! The view keeps track of the last state and only updates the layertree if the state
//! has changed. A View is mounted on a root layer and appends and removes layers based on
//! LayerTree rendered.
//!
//! The view keeps a cache of the layers that are rendered
//! by the engine to optimise their creation.
mod build_layer_tree;
mod layer_tree;

use std::{
    collections::{hash_map::DefaultHasher, HashMap, VecDeque},
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use crate::layers::layer::Layer;
use crate::prelude::*;

pub use build_layer_tree::*;
pub use layer_tree::*;

/// A View\<S\> is a struct to support the creation of complex hierarchies of layers
/// that can be rendered by the engine.

pub type PreRenderHook<S> = Arc<dyn Fn(&S, &View<S>) + Send + Sync + 'static>;
pub type PostRenderHook<S> = Arc<dyn Fn(&S, &View<S>, &Layer) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct View<S: Hash + Clone> {
    key: String,
    viewlayer_node_map: Arc<RwLock<HashMap<String, VecDeque<NodeRef>>>>,
    render_function: Arc<dyn ViewRenderFunction<S>>,
    pre_render_hooks: Arc<RwLock<Vec<PreRenderHook<S>>>>,
    post_render_hooks: Arc<RwLock<Vec<PostRenderHook<S>>>>,
    last_state: Arc<RwLock<Option<u64>>>,
    state: Arc<RwLock<S>>,
    pub layer: Arc<RwLock<Option<Layer>>>,
}

impl<S: Hash + Clone> View<S> {
    pub fn new(
        key: impl Into<String>,
        initial_state: S,
        render_function: impl ViewRenderFunction<S>,
    ) -> Self {
        let key = key.into();
        let render_function: Arc<dyn ViewRenderFunction<S>> = Arc::new(render_function);
        Self {
            key,
            layer: Arc::new(RwLock::new(None)),
            render_function,
            pre_render_hooks: Arc::new(RwLock::new(Vec::new())),
            post_render_hooks: Arc::new(RwLock::new(Vec::new())),
            last_state: Arc::new(RwLock::new(None)),
            viewlayer_node_map: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(initial_state)),
        }
    }
    /// Assigns a layer to the view, this will render the view into the layer
    pub fn set_layer(&self, layer: Layer) {
        self.layer.write().unwrap().replace(layer.clone());
        {
            self.render(&layer);
        }
    }
    /// Render the view into the layer
    #[profiling::function]
    pub fn render(&self, layer: &Layer) {
        let state = self.state.read().unwrap();

        // Call pre-render hooks
        {
            let hooks = self.pre_render_hooks.read().unwrap();
            for hook in hooks.iter() {
                hook(&state, self);
            }
        }

        let view = (self.render_function)(&state, self);
        // view.set_path(format!("{}.{}", self.path.clone(), self.key.clone()));
        {
            let mut viewlayer_node_map = self.viewlayer_node_map.write().unwrap();
            layer.build_layer_tree_internal(&view, &mut viewlayer_node_map);
        }
        // Call post-render hooks
        {
            let hooks = self.post_render_hooks.read().unwrap();
            for hook in hooks.iter() {
                hook(&state, self, layer);
            }
        }
    }
    /// Get the state of the view
    pub fn get_state(&self) -> S {
        self.state.read().unwrap().clone()
    }
    /// Set the state of the view without rendering the layer
    pub fn set_state(&self, state: S) {
        *self.state.write().unwrap() = state;
    }
    /// Update the state of the view and render the layer if the state has changed
    #[profiling::function]
    pub fn update_state(&self, state: &S) -> bool {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        let state_hash = hasher.finish();

        let mut last_state = self.last_state.write().unwrap();
        if last_state.is_none() || last_state.as_ref().unwrap() != &state_hash {
            {
                let mut state_mut = self.state.write().unwrap();
                *state_mut = state.clone();
            }
            *last_state = Some(state_hash);
            if let Some(layer) = &*self.layer.read().unwrap() {
                self.render(layer);
                return true;
            }
        }
        false
    }

    pub fn contains_point(&self, point: skia::Point) -> bool {
        if let Some(layer) = &*self.layer.read().unwrap() {
            return layer.engine.scene.with_arena(|arena| {
                let node = arena.get(layer.id.into()).unwrap();
                let node = node.get();
                node.contains_point(&point)
            });
        }
        false
    }

    #[cfg(feature = "layer_state")]
    pub fn get_internal_state<T: Clone + 'static>(&self, name: impl AsRef<str>) -> Option<T> {
        self.layer
            .read()
            .unwrap()
            .clone()
            .and_then(|l| l.with_state(|state| state.get::<T>(name)))
    }

    #[cfg(feature = "layer_state")]
    pub fn set_internal_state<T: Clone + Send + Sync + 'static>(
        &self,
        name: impl AsRef<str>,
        val: &T,
    ) {
        let layer_guard = self.layer.write().unwrap();
        if let Some(layer) = layer_guard.clone() {
            layer.with_mut_state(|state| {
                state.insert::<T>(name, val.clone());
            });
            drop(layer_guard);
            self.render(&layer);
        }
    }
    pub fn layer_by_key(&self, id: &str) -> Option<Layer> {
        let viewlayer_node_map = self.viewlayer_node_map.read().unwrap();
        viewlayer_node_map
            .get(id)
            .and_then(|v| v.front())
            .and_then(|node| {
                // FIXME
                self.layer
                    .read()
                    .unwrap()
                    .clone()
                    .and_then(|layer| layer.engine.get_layer(node))
                // if let Some(root) = &*self.layer.read().unwrap() {
                // if let Some(node) = root.engine.scene_get_node(node) {
                // let scene_node = node.get();
                // return Some(scene_node.layer.clone());
                // }
                // }
                // None
            })
            .or_else(|| None)
    }
    pub fn hover_layer(&self, id: &str, location: &Point) -> bool {
        if let Some(layer) = self.layer_by_key(id) {
            let rect = layer.render_bounds_transformed();
            if rect.x() < location.x
                && rect.x() + rect.width() > location.x
                && rect.y() < location.y
                && rect.y() + rect.height() > location.y
            {
                return true;
            }
        }
        false
    }

    /// Add a pre-render hook that will be called before each render
    /// Hook receives: (state, view)
    pub fn add_pre_render_hook<F>(&self, hook: F)
    where
        F: Fn(&S, &View<S>) + Send + Sync + 'static,
    {
        let mut hooks = self.pre_render_hooks.write().unwrap();
        hooks.push(Arc::new(hook));
    }

    /// Add a post-render hook that will be called after each render
    /// Hook receives: (state, view, layer)
    pub fn add_post_render_hook<F>(&self, hook: F)
    where
        F: Fn(&S, &View<S>, &Layer) + Send + Sync + 'static,
    {
        let mut hooks = self.post_render_hooks.write().unwrap();
        hooks.push(Arc::new(hook));
    }

    /// Clear all pre-render hooks
    pub fn clear_pre_render_hooks(&self) {
        self.pre_render_hooks.write().unwrap().clear();
    }

    /// Clear all post-render hooks
    pub fn clear_post_render_hooks(&self) {
        self.post_render_hooks.write().unwrap().clear();
    }
}

pub trait ViewRenderFunction<S: Hash + Clone>:
    Fn(&S, &View<S>) -> LayerTree + Sync + Send + 'static
{
}
impl<F, S> ViewRenderFunction<S> for F
where
    F: Fn(&S, &View<S>) -> LayerTree + Sync + Send + 'static,
    S: Hash + Clone,
{
}

impl<S: Hash + Clone> std::fmt::Debug for View<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            // .field("layer", &self.layer)
            .field("last_state", &self.last_state)
            .finish()
    }
}

impl<S: Hash + Clone> RenderLayerTree for View<S> {
    fn get_key(&self) -> String {
        self.key.clone()
    }
    fn mount_layer(&self, layer: Layer) {
        self.set_layer(layer);
    }
    fn render_layertree(&self) -> LayerTree {
        let state = self.state.read().unwrap();
        (self.render_function)(&state, self)
    }
}
