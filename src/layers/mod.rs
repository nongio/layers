//! Internal models representing the Layers and their animatable properties.

// use std::sync::Arc;

use std::collections::VecDeque;

use derive_builder::Builder;

use self::layer::Layer;
use crate::drawing::layer::draw_layer;
use crate::engine::NodeRef;
use crate::prelude::*;
use indextree::NodeId;
use skia_safe::Picture;

pub mod layer;

impl Drawable for Layer {
    fn bounds(&self) -> Rectangle {
        self.model.bounds()
    }
    fn draw(&self, canvas: &mut skia_safe::Canvas) {
        draw_layer(canvas, &RenderLayer::from(&*self.model))
    }
    fn scale(&self) -> (f32, f32) {
        self.model.scale()
    }
    fn scaled_bounds(&self) -> Rectangle {
        self.model.scaled_bounds()
    }
    fn transform(&self) -> Matrix {
        self.model.transform()
    }
    fn anchor_point(&self) -> (f32, f32) {
        self.model.anchor_point()
    }
    fn opacity(&self) -> f32 {
        self.model.opacity()
    }

    fn blend_mode(&self) -> BlendMode {
        self.model.blend_mode()
    }
    fn border_corner_radius(&self) -> BorderRadius {
        self.model.border_corner_radius()
    }
}

// #[repr(C)]
#[derive(Clone, Debug, Builder, Default)]
#[builder(public, default)]
pub struct ViewLayer {
    pub background_color: (PaintColor, Option<Transition>),
    pub border_color: (Color, Option<Transition>),
    pub border_width: (f32, Option<Transition>),
    pub border_style: BorderStyle,
    pub border_corner_radius: (BorderRadius, Option<Transition>),
    pub size: (Point, Option<Transition>),
    pub position: (Point, Option<Transition>),
    #[builder(default = "(Point{x:1.0, y:1.0}, None)")]
    pub scale: (Point, Option<Transition>),
    pub shadow_offset: (Point, Option<Transition>),
    pub shadow_radius: (f32, Option<Transition>),
    pub shadow_color: (Color, Option<Transition>),
    pub shadow_spread: (f32, Option<Transition>),
    pub content: (Option<Picture>, Option<Transition>),
    pub blend_mode: BlendMode,
    pub layout_style: taffy::Style,
    #[builder(default = "(1.0, None)")]
    pub opacity: (f32, Option<Transition>),
    pub children: Vec<ViewLayer>,
}

pub trait BuildLayerTree {
    fn build_layer_tree(&self, tree: &ViewLayer);
}

impl BuildLayerTree for Layer {
    fn build_layer_tree(&self, tree: &ViewLayer) {
        let layer = self.clone();
        layer.set_position(tree.position.0, tree.position.1);
        layer.set_scale(tree.scale.0, tree.scale.1);

        layer.set_background_color(tree.background_color.0.clone(), tree.background_color.1);
        layer.set_border_color(tree.border_color.0, tree.border_color.1);
        layer.set_border_width(tree.border_width.0, tree.border_width.1);
        layer.set_border_corner_radius(tree.border_corner_radius.0, tree.border_corner_radius.1);
        layer.set_size(tree.size.0, tree.size.1);
        layer.set_shadow_offset(tree.shadow_offset.0, tree.shadow_offset.1);
        layer.set_shadow_radius(tree.shadow_radius.0, tree.shadow_radius.1);
        layer.set_shadow_color(tree.shadow_color.0, tree.shadow_color.1);
        layer.set_shadow_spread(tree.shadow_spread.0, tree.shadow_spread.1);
        layer.set_layout_style(tree.layout_style.clone());
        layer.set_opacity(tree.opacity.0, tree.opacity.1);
        layer.set_blend_mode(tree.blend_mode.clone());
        layer.set_content(tree.content.0.clone(), tree.content.1);

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

            let mut child_layers: VecDeque<NodeId> = child_layers.collect();
            drop(arena);
            for child in tree.children.iter() {
                // check if there is already a layer for this child otherwise create one

                let layer = child_layers.pop_front().unwrap_or_else(|| {
                    let layer = Layer::with_engine(engine.clone());
                    let id = engine.scene_add_layer(layer, Some(id));
                    id.0
                });
                let arena = engine.scene.nodes.data();
                let arena = arena.read().unwrap();
                let layer = arena.get(layer).unwrap();
                let layer = layer.get().clone();
                drop(arena);
                layer.model.build_layer_tree(child);
            }

            while let Some(child) = child_layers.pop_front() {
                engine.scene_remove_layer(Some(NodeRef(child)));
            }
        }
    }
}
