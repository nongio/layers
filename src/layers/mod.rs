//! Internal models representing the Layers and their animatable properties.

use core::fmt;
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use derive_builder::Builder;

use self::layer::{
    model::{ContentDrawFunction, PointerHandlerFunction},
    Layer,
};
use crate::prelude::*;
use crate::{engine::NodeRef, types::Size};
use indextree::NodeId;

pub mod layer;

// #[repr(C)]
#[derive(Clone, Builder, Default)]
#[builder(public, default)]
pub struct ViewLayer {
    #[builder(setter(into, strip_option))]
    pub key: String,
    #[builder(setter(into, strip_option), default)]
    pub background_color: Option<(PaintColor, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub border_color: Option<(Color, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub border_width: Option<(f32, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub border_style: Option<BorderStyle>,
    #[builder(setter(into, strip_option), default)]
    pub border_corner_radius: Option<(BorderRadius, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub size: Option<(Size, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub position: Option<(Point, Option<Transition>)>,
    #[builder(setter(into, strip_option))]
    pub scale: Option<(Point, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_offset: Option<(Point, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_radius: Option<(f32, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_color: Option<(Color, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_spread: Option<(f32, Option<Transition>)>,
    #[builder(setter(custom))]
    pub content: Option<ContentDrawFunction>,
    #[builder(setter(into, strip_option), default)]
    pub blend_mode: Option<BlendMode>,
    #[builder(setter(into, strip_option), default)]
    pub layout_style: Option<taffy::Style>,
    #[builder(setter(into, strip_option))]
    pub opacity: Option<(f32, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub children: Option<Vec<ViewLayer>>,
    #[builder(setter(into, strip_option), default)]
    pub image_cache: Option<bool>,
    #[builder(setter(custom))]
    pub on_pointer_move: Option<PointerHandlerFunction>,
}

impl fmt::Debug for ViewLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ViewLayer")
            .field("key", &self.key)
            .field("background_color", &self.background_color)
            .field("border_color", &self.border_color)
            .field("border_width", &self.border_width)
            .field("border_style", &self.border_style)
            .field("border_corner_radius", &self.border_corner_radius)
            .field("size", &self.size)
            .field("position", &self.position)
            .field("scale", &self.scale)
            .field("shadow_offset", &self.shadow_offset)
            .field("shadow_radius", &self.shadow_radius)
            .field("shadow_color", &self.shadow_color)
            .field("shadow_spread", &self.shadow_spread)
            // .field("content", &self.content)
            .field("blend_mode", &self.blend_mode)
            .field("layout_style", &self.layout_style)
            .field("opacity", &self.opacity)
            .field("children", &self.children)
            .finish()
    }
}

fn cache_remove_viewlayer(view: &ViewLayer, cache_viewlayer: &mut HashMap<ViewLayer, NodeRef>) {
    cache_viewlayer.remove(view);
    if let Some(children) = view.children.clone() {
        for child in children.iter() {
            cache_remove_viewlayer(child, cache_viewlayer);
        }
    }
}

impl Hash for ViewLayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}
impl PartialEq for ViewLayer {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for ViewLayer {}

#[allow(clippy::from_over_into)]
impl Into<Vec<ViewLayer>> for ViewLayer {
    fn into(self) -> Vec<ViewLayer> {
        vec![self]
    }
}

impl ViewLayerBuilder {
    pub fn content<F: Into<ContentDrawFunction>>(
        &mut self,
        content_handler: Option<F>,
    ) -> &mut Self {
        if let Some(content_handler) = content_handler {
            let content = Some(content_handler.into());
            self.content = Some(content);
        }
        self
    }
    pub fn on_pointer_move<F: Into<PointerHandlerFunction>>(
        &mut self,
        on_pointer_move: F,
    ) -> &mut Self {
        let on_pointer_move = Some(on_pointer_move.into());
        self.on_pointer_move = Some(on_pointer_move);
        self
    }
}
pub trait BuildLayerTree {
    fn build_layer_tree(&self, tree: &ViewLayer, cache_viewlayer: &mut HashMap<ViewLayer, NodeRef>);
}
pub fn unique_layer_key() -> String {
    format!("layer_{}", rand::random::<u64>())
}

impl BuildLayerTree for Layer {
    fn build_layer_tree(
        &self,
        viewlayer_tree: &ViewLayer,
        cache_viewlayer: &mut HashMap<ViewLayer, NodeRef>,
    ) {
        let scene_layer = self.clone();
        let key = if viewlayer_tree.key.is_empty() {
            // generate a random key
            unique_layer_key()
        } else {
            viewlayer_tree.key.clone()
        };

        scene_layer.set_key(key);

        if let Some((position, transition)) = viewlayer_tree.position {
            scene_layer.set_position(position, transition);
        }
        if let Some((scale, transition)) = viewlayer_tree.scale {
            scene_layer.set_scale(scale, transition);
        }
        if let Some((background_color, transition)) = viewlayer_tree.background_color.clone() {
            scene_layer.set_background_color(background_color, transition);
        }
        if let Some((border_color, transition)) = viewlayer_tree.border_color {
            scene_layer.set_border_color(border_color, transition);
        }
        if let Some((border_width, transition)) = viewlayer_tree.border_width {
            scene_layer.set_border_width(border_width, transition);
        }
        if let Some((border_corner_radius, transition)) = viewlayer_tree.border_corner_radius {
            scene_layer.set_border_corner_radius(border_corner_radius, transition);
        }
        if let Some((size, transition)) = viewlayer_tree.size {
            scene_layer.set_size(size, transition);
        }
        if let Some((shadow_offset, transition)) = viewlayer_tree.shadow_offset {
            scene_layer.set_shadow_offset(shadow_offset, transition);
        }
        if let Some((shadow_radius, transition)) = viewlayer_tree.shadow_radius {
            scene_layer.set_shadow_radius(shadow_radius, transition);
        }
        if let Some((shadow_color, transition)) = viewlayer_tree.shadow_color {
            scene_layer.set_shadow_color(shadow_color, transition);
        }
        if let Some((shadow_spread, transition)) = viewlayer_tree.shadow_spread {
            scene_layer.set_shadow_spread(shadow_spread, transition);
        }
        if let Some(layout_style) = viewlayer_tree.layout_style.clone() {
            scene_layer.set_layout_style(layout_style);
        }
        if let Some((opacity, transition)) = viewlayer_tree.opacity {
            scene_layer.set_opacity(opacity, transition);
        }
        if let Some(blend_mode) = viewlayer_tree.blend_mode {
            scene_layer.set_blend_mode(blend_mode);
        }

        if let Some(content) = viewlayer_tree.content.clone() {
            scene_layer.set_draw_content(Some(content));
        }

        if let Some(image_cache) = viewlayer_tree.image_cache {
            scene_layer.set_image_cache(image_cache);
        }
        if let Some(on_pointer_move) = viewlayer_tree.on_pointer_move.clone() {
            scene_layer.remove_all_handlers();
            scene_layer.add_on_pointer_move(on_pointer_move);
        }
        let id = scene_layer.id();
        let engine = scene_layer.engine;
        if let Some(id) = id {
            let mut old_scene_layers: HashSet<NodeId> = {
                let arena = engine.scene.nodes.data();
                let arena = arena.read().unwrap();
                id.0.children(&arena).collect()
            };

            // // add missing layers

            let mut layer_view_map: HashMap<NodeRef, ViewLayer> = HashMap::new();
            {
                // this seems like a copy...
                for (view_layer, node_id) in cache_viewlayer.iter() {
                    layer_view_map.insert(*node_id, view_layer.clone());
                }
            }

            if let Some(children) = viewlayer_tree.children.as_ref() {
                for child in children.iter() {
                    let mut child = child.clone();
                    child.key = if child.key.is_empty() {
                        unique_layer_key()
                    } else {
                        child.key.clone()
                    };
                    // check if there is already a layer for this child otherwise create one
                    let scene_layer_id = { cache_viewlayer.get(&child).cloned() };

                    let scene_layer_id = scene_layer_id.unwrap_or_else(|| {
                        let layer = Layer::with_engine(engine.clone());
                        let id = engine.scene_add_layer(layer, Some(id));
                        cache_viewlayer.insert(child.clone(), id);
                        id
                    });

                    let scene_node = engine.scene.get_node(scene_layer_id).unwrap();
                    let scene_layer = scene_node.get().clone();
                    // re-add the layer to the parent in case it is not in the right order
                    engine.scene_add_layer(scene_layer.layer.clone(), Some(id));
                    scene_layer.layer.build_layer_tree(&child, cache_viewlayer);

                    old_scene_layers.remove(&scene_layer_id);
                }
            }

            // remove remaining extra layers
            for scene_layer_id in old_scene_layers {
                let scene_layer_ref = NodeRef(scene_layer_id);

                let scene_layer = {
                    let arena = engine.scene.nodes.data();
                    let arena = arena.read().unwrap();
                    let scene_node = arena.get(scene_layer_id).unwrap();
                    scene_node.get().clone()
                };
                // let transition = scene_layer.layer.set_size(
                //     Size {
                //         width: taffy::Dimension::Points(0.0),
                //         height: taffy::Dimension::Points(0.0),
                //     },
                //     Some(Transition {
                //         duration: 0.5,
                //         ..Default::default()
                //     }),
                // );

                {
                    if let Some(view) = layer_view_map.get(&scene_layer_ref) {
                        cache_remove_viewlayer(view, cache_viewlayer);
                    }
                }
                // let scene_layer_clone = scene_layer.clone();
                // scene_layer.layer.on_finish(transition, move |_| {
                scene_layer.delete();
                // });
            }
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct View<S: Hash + Clone> {
    viewlayer_node_map: Arc<RwLock<HashMap<ViewLayer, NodeRef>>>,
    render_function: Arc<dyn Fn(&S, &View<S>) -> ViewLayer + Sync + Send>,
    last_state: Arc<RwLock<Option<u64>>>,
    pub state: Arc<RwLock<S>>,
    pub layer: Layer,
}

impl<S: Hash + Clone> std::fmt::Debug for View<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            // .field("layer", &self.layer)
            .field("last_state", &self.last_state)
            .finish()
    }
}
// impl View for a function that accept an argument
#[allow(clippy::type_complexity)]
impl<S: Hash + Clone> View<S> {
    pub fn new(
        layer: Layer,
        initial_state: S,
        render_function: Box<dyn Fn(&S, &View<S>) -> ViewLayer + Sync + Send>,
    ) -> Self {
        let view = Self {
            layer,
            render_function: Arc::from(render_function),
            last_state: Arc::new(RwLock::new(None)),
            viewlayer_node_map: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(initial_state)),
        };
        {
            let state = view.state.read().unwrap();
            view.render(&state);
        }
        view
    }

    pub fn render(&self, state: &S) {
        let view = (self.render_function)(state, self);
        let mut viewlayer_node_map = self.viewlayer_node_map.write().unwrap();
        self.layer.build_layer_tree(&view, &mut viewlayer_node_map);
    }
    pub fn get_state(&self) -> S {
        self.state.read().unwrap().clone()
    }
    pub fn set_state(&self, state: S) {
        *self.state.write().unwrap() = state;
    }
    pub fn update_state(&self, state: S) -> bool {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        let state_hash = hasher.finish();

        let mut last_state = self.last_state.write().unwrap();
        if last_state.is_none() || last_state.as_ref().unwrap() != &state_hash {
            let mut state_mut = self.state.write().unwrap();
            *state_mut = state;
            *last_state = Some(state_hash);
            self.render(&state_mut);
            return true;
        }
        false
    }

    pub fn get_layer_by_id(&self, id: &str) -> Option<Layer> {
        let view_layer_map = self.viewlayer_node_map.read().unwrap();
        let keys = view_layer_map
            .keys()
            .map(|vl| vl.key.clone())
            .collect::<Vec<String>>();
        println!("view_layer_map: {:?}", keys);
        let view_layer = view_layer_map
            .keys()
            .find(|view_layer| view_layer.key == id)?;

        if let Some(node_ref) = view_layer_map.get(view_layer) {
            let scene_node = self.layer.engine.scene.get_node(node_ref.0);
            if let Some(scene_node) = scene_node {
                let node = scene_node.get();
                return Some(node.layer.clone());
            }
        }
        None
    }
}
