//! Internal models representing the Layers and their animatable properties.

use std::sync::Arc;

use derive_builder::Builder;

use indextree::NodeId;
use taffy::prelude::Node;

use self::layer::Layer;
use self::text::{RenderText, TextLayer};
use crate::drawing::layer::draw_layer;
use crate::drawing::text::draw_text;
use crate::engine::NodeRef;
use crate::prelude::*;

pub mod layer;
pub mod text;

#[derive(Clone)]
pub enum Layers {
    Layer(layer::Layer),
    TextLayer(text::TextLayer),
}
impl Layers {
    pub fn id(&self) -> Option<NodeRef> {
        match self {
            Layers::Layer(layer) => *layer.id.read().unwrap(),
            Layers::TextLayer(layer) => *layer.id.read().unwrap(),
        }
    }
    pub fn set_id(&self, id: NodeRef) {
        match self {
            Layers::Layer(layer) => layer.set_id(id),
            Layers::TextLayer(layer) => layer.set_id(id),
        }
    }
    pub fn layout_node(&self) -> Node {
        match self {
            Layers::Layer(layer) => layer.layout,
            Layers::TextLayer(layer) => layer.layout,
        }
    }
}
impl Drawable for Layers {
    fn bounds(&self) -> Rectangle {
        match self {
            Layers::Layer(layer) => layer.model.bounds(),
            Layers::TextLayer(layer) => layer.model.bounds(),
        }
    }
    fn draw(&self, canvas: &mut skia_safe::Canvas) {
        match self {
            Layers::Layer(layer) => draw_layer(canvas, &RenderLayer::from(&*layer.model)),
            Layers::TextLayer(layer) => draw_text(canvas, &RenderText::from(&*layer.model)),
        }
    }
    fn scale(&self) -> (f32, f32) {
        match self {
            Layers::Layer(layer) => layer.model.scale(),
            Layers::TextLayer(layer) => layer.model.scale(),
        }
    }
    fn scaled_bounds(&self) -> Rectangle {
        match self {
            Layers::Layer(layer) => layer.model.scaled_bounds(),
            Layers::TextLayer(layer) => layer.model.scaled_bounds(),
        }
    }
    fn transform(&self) -> Matrix {
        match self {
            Layers::Layer(layer) => layer.model.transform(),
            Layers::TextLayer(layer) => layer.model.transform(),
        }
    }
}
impl From<Layer> for Layers {
    fn from(layer: Layer) -> Self {
        Layers::Layer(layer)
    }
}
impl From<TextLayer> for Layers {
    fn from(layer: TextLayer) -> Self {
        Layers::TextLayer(layer)
    }
}

// #[repr(C)]
#[derive(Clone, Debug, Builder, Default)]
#[builder(public, default)]
pub struct ViewLayer {
    pub background_color: (PaintColor, Option<Transition<Easing>>),
    pub border_color: (Color, Option<Transition<Easing>>),
    pub border_width: (f32, Option<Transition<Easing>>),
    pub border_style: BorderStyle,
    pub border_corner_radius: (BorderRadius, Option<Transition<Easing>>),
    pub size: (Point, Option<Transition<Easing>>),
    pub position: (Point, Option<Transition<Easing>>),
    #[builder(default = "(Point{x:1.0, y:1.0}, None)")]
    pub scale: (Point, Option<Transition<Easing>>),
    pub shadow_offset: (Point, Option<Transition<Easing>>),
    pub shadow_radius: (f32, Option<Transition<Easing>>),
    pub shadow_color: (Color, Option<Transition<Easing>>),
    pub shadow_spread: (f32, Option<Transition<Easing>>),
    pub content: Option<Image>,
    pub blend_mode: BlendMode,
    pub layout_style: Style,
}

#[allow(dead_code)]
#[derive(Debug, Builder, Default)]
#[builder(public, default)]
pub struct ViewLayerTree {
    pub root: Arc<ViewLayer>,
    pub children: Vec<Arc<ViewLayerTree>>,
}

pub trait BuildLayerTree {
    fn build_layer_tree(&self, tree: &ViewLayerTree);
}
impl BuildLayerTree for Layers {
    fn build_layer_tree(&self, tree: &ViewLayerTree) {
        match self {
            Layers::Layer(layer) => layer.build_layer_tree(tree),
            Layers::TextLayer(_layer) => (),
        }
    }
}
impl BuildLayerTree for Layer {
    fn build_layer_tree(&self, tree: &ViewLayerTree) {
        let layer = self.clone();
        layer.set_position(tree.root.position.0, tree.root.position.1);
        layer.set_scale(tree.root.scale.0, tree.root.scale.1);

        layer.set_background_color(
            tree.root.background_color.0.clone(),
            tree.root.background_color.1,
        );
        layer.set_border_color(tree.root.border_color.0, tree.root.border_color.1);
        layer.set_border_width(tree.root.border_width.0, tree.root.border_width.1);
        layer.set_border_corner_radius(
            tree.root.border_corner_radius.0,
            tree.root.border_corner_radius.1,
        );
        layer.set_size(tree.root.size.0, tree.root.size.1);
        layer.set_shadow_offset(tree.root.shadow_offset.0, tree.root.shadow_offset.1);
        layer.set_shadow_radius(tree.root.shadow_radius.0, tree.root.shadow_radius.1);
        layer.set_shadow_color(tree.root.shadow_color.0, tree.root.shadow_color.1);
        layer.set_shadow_spread(tree.root.shadow_spread.0, tree.root.shadow_spread.1);
        // layer.set_blend_mode(tree.root.blend_mode);
        // layer.set_content(tree.root.content.clone());
        let id = layer.id();
        let engine = layer.engine;
        if let Some(id) = id {
            // let id = id.0;
            let arena = engine.scene.nodes.data();
            let arena = arena.read().unwrap();
            let child_layers = id.0.children(&arena);

            // TODO remove extra layers
            // if tree.children.len() < child_layers.count() {
            //     let child_layers = id.0.children(&arena);
            //     for child in child_layers {
            //         //         engine.scene_remove_layer(NodeRef(child));
            //     }
            // }

            // add missing layers

            let mut child_layers: Vec<NodeId> = child_layers.collect();
            drop(arena);
            for child in tree.children.iter() {
                let layer = child_layers.pop().unwrap_or_else(|| {
                    let layer = Layer::with_engine(engine.clone());
                    let id = engine.scene_add_layer(layer, Some(id));
                    id.0
                });
                let arena = engine.scene.nodes.data();
                let arena = arena.read().unwrap();
                let layer = arena.get(layer).unwrap();
                let layer = layer.get();
                layer.model.build_layer_tree(child);
            }
            while !child_layers.is_empty() {
                let child = child_layers.pop().unwrap();
                engine.scene_remove_layer(NodeRef(child));
            }
        }
    }
}
