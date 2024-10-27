use std::collections::{HashMap, HashSet, VecDeque};

use crate::engine::{storage::TreeStorageId, NodeRef};
use crate::layers::layer::Layer;
use crate::prelude::*;
use indextree::NodeId;

/// Helper function to remove a viewlayer from the cache
fn cache_remove_viewlayer(
    view_key: &String,
    layer_id: Option<&NodeRef>,
    cache_viewlayer: &mut HashMap<String, VecDeque<NodeRef>>,
) {
    let nodes = if let Some(nodes) = cache_viewlayer.get_mut(view_key) {
        if let Some(layer_id) = layer_id {
            nodes.retain(|node| !node_same_index(node.0, layer_id.0));
        } else {
            nodes.pop_front();
        }
        Some(nodes.clone())
    } else {
        None
    };
    if let Some(nodes) = nodes {
        cache_viewlayer.insert(view_key.clone(), nodes.clone());
    }
}
fn node_same_index(node1: impl Into<TreeStorageId>, node2: impl Into<TreeStorageId>) -> bool {
    let node1: TreeStorageId = node1.into();
    let node2: TreeStorageId = node2.into();
    let index1: usize = node1.into();
    let index2: usize = node2.into();
    index1 == index2
}
fn cache_remove_id(id: &NodeRef, cache_viewlayer: &mut HashMap<String, VecDeque<NodeRef>>) {
    let mut keys: Vec<String> = Vec::new();
    for (key, nodes) in cache_viewlayer.iter() {
        if nodes.iter().any(|node| node_same_index(node.0, id.0)) {
            keys.push(key.clone());
        }
    }
    for key in keys {
        cache_remove_viewlayer(&key, Some(id), cache_viewlayer);
    }
}

/// A trait for structs that accept a layertree. It is implemented for Layer
/// it generates and updates the properties of a hierarchy of layers
/// described by the layertree
pub trait BuildLayerTree {
    fn build_layer_tree_internal(
        &self,
        viewlayer_tree: &LayerTree,
        cache_viewlayer: &mut HashMap<String, VecDeque<NodeRef>>,
    );
    fn build_layer_tree(&self, viewlayer_tree: &LayerTree) {
        self.build_layer_tree_internal(viewlayer_tree, &mut HashMap::new());
    }
}

impl BuildLayerTree for Layer {
    #[profiling::function]
    fn build_layer_tree_internal(
        &self,
        viewlayer_tree: &LayerTree,
        cache_viewlayer: &mut HashMap<String, VecDeque<NodeRef>>,
    ) {
        let scene_layer = self.clone();

        if !viewlayer_tree.key.is_empty() {
            scene_layer.set_key(viewlayer_tree.key.clone());
        }

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
            scene_layer.set_draw_content(content);
        }

        if let Some(image_cache) = viewlayer_tree.image_cache {
            scene_layer.set_image_cache(image_cache);
        }
        if let Some(pointer_events) = viewlayer_tree.pointer_events {
            scene_layer.set_pointer_events(pointer_events);
        }

        // Handlers
        scene_layer.remove_all_pointer_handlers();

        if let Some(on_pointer_move) = viewlayer_tree.on_pointer_move.clone() {
            scene_layer.add_on_pointer_move(on_pointer_move);
        }
        if let Some(on_pointer_in) = viewlayer_tree.on_pointer_in.clone() {
            scene_layer.add_on_pointer_in(on_pointer_in);
        }
        if let Some(on_pointer_out) = viewlayer_tree.on_pointer_out.clone() {
            scene_layer.add_on_pointer_out(on_pointer_out);
        }
        if let Some(on_pointer_press) = viewlayer_tree.on_pointer_press.clone() {
            scene_layer.add_on_pointer_press(on_pointer_press);
        }
        if let Some(on_pointer_release) = viewlayer_tree.on_pointer_release.clone() {
            scene_layer.add_on_pointer_release(on_pointer_release);
        }

        // Children
        let layer_id = scene_layer.id();
        let engine = scene_layer.engine;
        if let Some(layer_id) = layer_id {
            let mut current_scene_layers_children: HashSet<NodeId> = {
                let arena = engine.scene.nodes.data();
                let arena = arena.read().unwrap();
                layer_id.0.children(&arena).collect()
            };

            let mut layer_view_map: HashMap<NodeRef, String> = HashMap::new();
            {
                for (view_key, nodes) in cache_viewlayer.iter() {
                    for node_id in nodes.iter() {
                        layer_view_map.insert(*node_id, view_key.clone());
                    }
                }
            }
            // Add missing layers
            if let Some(children) = viewlayer_tree.children.as_ref() {
                for child in children.iter() {
                    let child_key = child.key().clone();
                    // check if there is already a layer for this child otherwise create one
                    let mut nodes = cache_viewlayer.get(&child_key).cloned().unwrap_or_default();
                    let child_layer_id = nodes.pop_front();

                    // get or create the child layer
                    let (child_layer_id, child_scene_layer) = child_layer_id
                        .and_then(|child_layer_id| {
                            // try to use existing layer
                            let child_scene_node = engine.scene.get_node(child_layer_id).unwrap();
                            if child_scene_node.is_removed() {
                                return None;
                            }
                            let child_scene_layer = child_scene_node.get().clone();

                            if child_scene_layer.is_deleted() {
                                return None;
                            }
                            // we should not need to add the layer back to the parent
                            engine.scene_add_layer(child_scene_layer.layer.clone(), Some(layer_id));
                            Some((child_layer_id, child_scene_layer))
                        })
                        .unwrap_or_else(|| {
                            // the child layer does not exist, or is removed
                            let layer = Layer::with_engine(engine.clone());
                            let id = engine.scene_add_layer(layer, Some(layer_id));
                            let node = engine.scene.get_node(id).unwrap();

                            (id, node.get().clone())
                        });

                    layer_view_map.retain(|n, _| !node_same_index(n.0, child_layer_id.0));
                    cache_remove_id(&child_layer_id, cache_viewlayer);
                    drop(nodes);

                    // re-add the layer to the parent in case it is not in the right order
                    child.mount_layer(child_scene_layer.layer.clone());
                    let child = child.render_layertree();
                    child_scene_layer
                        .layer
                        .build_layer_tree_internal(&child, cache_viewlayer);
                    {
                        // add child to cache
                        let mut nodes =
                            cache_viewlayer.get(&child_key).cloned().unwrap_or_default();
                        nodes.push_back(child_layer_id);
                        cache_viewlayer.insert(child.key(), nodes);
                    }

                    current_scene_layers_children
                        .retain(|id| !node_same_index(*id, child_layer_id.0));
                }
            }

            // Remove remaining extra layers
            for scene_layer_id in current_scene_layers_children {
                let scene_layer_ref = NodeRef(scene_layer_id);

                let scene_layer = {
                    let arena = engine.scene.nodes.data();
                    let arena = arena.read().unwrap();
                    let scene_node = arena.get(scene_layer_id).unwrap();
                    scene_node.get().clone()
                };
                // let transition = scene_layer.layer.set_size(
                //     Size {
                //         width: taffy::Dimension::Length(0.0),
                //         height: taffy::Dimension::Length(0.0),
                //     },
                //     Some(Transition {
                //         duration: 0.5,
                //         ..Default::default()
                //     }),
                // );

                {
                    if let Some(view_key) = layer_view_map.get(&scene_layer_ref) {
                        cache_remove_viewlayer(view_key, None, cache_viewlayer);
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
