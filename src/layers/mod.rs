//! Internal models representing the Layers and their animatable properties.

use core::fmt;
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};

use derive_builder::Builder;

use self::layer::{model::ContentDrawFunction, Layer};
use crate::prelude::*;
use crate::{engine::NodeRef, types::Size};
use indextree::NodeId;

pub mod layer;

// #[repr(C)]
#[derive(Clone, Builder, Default)]
#[builder(public, default)]
pub struct ViewLayer {
    #[builder(setter(custom))]
    pub id: String,
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
}

impl fmt::Debug for ViewLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ViewLayer")
            .field("id", &self.id)
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

fn unmap_view(view: &ViewLayer, view_layer_map: &mut HashMap<ViewLayer, NodeRef>) {
    view_layer_map.remove(view);
    if let Some(children) = view.children.clone() {
        for child in children.iter() {
            unmap_view(child, view_layer_map);
        }
    }
}

impl Hash for ViewLayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for ViewLayer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ViewLayer {}

impl ViewLayerBuilder {
    pub fn id(&mut self, id: impl Into<String>) -> &mut Self {
        let id = id.into();
        self.id = Some(id);
        self
    }
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
}
pub trait BuildLayerTree {
    fn build_layer_tree(&self, tree: &ViewLayer, view_layer_map: &mut HashMap<ViewLayer, NodeRef>);
}

impl BuildLayerTree for Layer {
    fn build_layer_tree(
        &self,
        view_layer_tree: &ViewLayer,
        view_layer_map: &mut HashMap<ViewLayer, NodeRef>,
    ) {
        let scene_layer = self.clone();
        if let Some((position, transition)) = view_layer_tree.position {
            scene_layer.set_position(position, transition);
        }
        if let Some((scale, transition)) = view_layer_tree.scale {
            scene_layer.set_scale(scale, transition);
        }
        if let Some((background_color, transition)) = view_layer_tree.background_color.clone() {
            scene_layer.set_background_color(background_color, transition);
        }
        if let Some((border_color, transition)) = view_layer_tree.border_color {
            scene_layer.set_border_color(border_color, transition);
        }
        if let Some((border_width, transition)) = view_layer_tree.border_width {
            scene_layer.set_border_width(border_width, transition);
        }
        if let Some((border_corner_radius, transition)) = view_layer_tree.border_corner_radius {
            scene_layer.set_border_corner_radius(border_corner_radius, transition);
        }
        if let Some((size, transition)) = view_layer_tree.size {
            scene_layer.set_size(size, transition);
        }
        if let Some((shadow_offset, transition)) = view_layer_tree.shadow_offset {
            scene_layer.set_shadow_offset(shadow_offset, transition);
        }
        if let Some((shadow_radius, transition)) = view_layer_tree.shadow_radius {
            scene_layer.set_shadow_radius(shadow_radius, transition);
        }
        if let Some((shadow_color, transition)) = view_layer_tree.shadow_color {
            scene_layer.set_shadow_color(shadow_color, transition);
        }
        if let Some((shadow_spread, transition)) = view_layer_tree.shadow_spread {
            scene_layer.set_shadow_spread(shadow_spread, transition);
        }
        if let Some(layout_style) = view_layer_tree.layout_style.clone() {
            scene_layer.set_layout_style(layout_style);
        }
        if let Some((opacity, transition)) = view_layer_tree.opacity {
            scene_layer.set_opacity(opacity, transition);
        }
        if let Some(blend_mode) = view_layer_tree.blend_mode {
            scene_layer.set_blend_mode(blend_mode);
        }

        if let Some(content) = view_layer_tree.content.clone() {
            scene_layer.set_draw_content(Some(content));
        }

        let id = scene_layer.id();
        let engine = scene_layer.engine;
        if let Some(id) = id {
            let mut existing_scene_child_layers: HashSet<NodeId> = {
                let arena = engine.scene.nodes.data();
                let arena = arena.read().unwrap();
                id.0.children(&arena).collect()
            };

            // // add missing layers

            let mut layer_view_map: HashMap<NodeRef, ViewLayer> = HashMap::new();
            {
                for (view_layer, node_id) in view_layer_map.iter() {
                    layer_view_map.insert(*node_id, view_layer.clone());
                }
            }

            if let Some(children) = view_layer_tree.children.as_ref() {
                for child in children.iter() {
                    // check if there is already a layer for this child otherwise create one
                    let scene_layer_id = { view_layer_map.get(child).cloned() };

                    let scene_layer_id = scene_layer_id.unwrap_or_else(|| {
                        let layer = Layer::with_engine(engine.clone());

                        let id = engine.scene_add_layer(layer, Some(id));
                        view_layer_map.insert(child.clone(), id);
                        id
                    });

                    let scene_node = engine.scene.get_node(scene_layer_id).unwrap();
                    let scene_layer = scene_node.get().clone();
                    scene_layer.layer.build_layer_tree(child, view_layer_map);

                    existing_scene_child_layers.remove(&scene_layer_id);
                }
            }

            // remove remaining extra layers
            for scene_layer_id in existing_scene_child_layers {
                let scene_layer_ref = NodeRef(scene_layer_id);

                let scene_layer = {
                    let arena = engine.scene.nodes.data();
                    let arena = arena.read().unwrap();
                    let scene_node = arena.get(scene_layer_id).unwrap();
                    scene_node.get().clone()
                };
                let transition = scene_layer.layer.set_size(
                    Size {
                        width: taffy::Dimension::Points(0.0),
                        height: taffy::Dimension::Points(0.0),
                    },
                    Some(Transition {
                        duration: 0.5,
                        ..Default::default()
                    }),
                );

                {
                    if let Some(view) = layer_view_map.get(&scene_layer_ref) {
                        unmap_view(view, view_layer_map);
                    }
                }
                let scene_layer_clone = scene_layer.clone();
                scene_layer.layer.on_finish(transition, move |_| {
                    scene_layer_clone.delete();
                });
            }
        }
    }
}

pub struct View<S: Hash> {
    view_layer_map: RefCell<HashMap<ViewLayer, NodeRef>>,
    render_function: Box<dyn Fn(&S) -> ViewLayer>,
    last_state: Option<u64>,
    last_view: Option<ViewLayer>,
    pub layer: Layer,
}

// impl View for a function that accept an argument
impl<S: Hash> View<S> {
    pub fn new(layer: Layer, render_function: Box<dyn Fn(&S) -> ViewLayer>) -> Self {
        Self {
            layer,
            render_function,
            last_state: None,
            last_view: None,
            view_layer_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn render(&mut self, state: &S) -> bool {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        let state_hash = hasher.finish();

        if self.last_state.is_none() || self.last_state.as_ref().unwrap() != &state_hash {
            let view = (self.render_function)(state);
            self.last_state = Some(state_hash);
            self.last_view = Some(view.clone());
            self.layer
                .build_layer_tree(&view, &mut self.view_layer_map.borrow_mut());
            return true;
        }
        false
    }
}
