use std::sync::{Arc, RwLock};

use indextree::NodeId;
use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use taffy::{prelude::Size, style_helpers::points, Taffy};

use crate::layers::layer::render_layer::RenderLayer;

use super::{
    node::{try_get_node, DrawCacheManagement, RenderableFlags, SceneNode},
    storage::{FlatStorageId, TreeStorageData},
    AnimationState, Engine, NodeRef, Timestamp,
};

#[profiling::function]
pub(crate) fn update_animations(engine: &Engine, timestamp: &Timestamp) -> Vec<FlatStorageId> {
    let finished_animations = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));

    let animations = engine.animations.data();
    let mut animations = animations.write().unwrap();
    if animations.len() > 0 {
        animations.par_iter_mut().for_each_with(
            finished_animations.clone(),
            |done_animations,
             (
                id,
                AnimationState {
                    animation,
                    progress,
                    is_running,
                    is_finished,
                },
            )| {
                if !*is_running {
                    return;
                }
                let (animation_progress, time_progress) = animation.value_at(timestamp.0);
                *progress = animation_progress;
                if time_progress >= 1.0 {
                    *is_running = false;
                    *is_finished = true;
                    done_animations.clone().write().unwrap().push(*id);
                }
            },
        );
    }

    let vec = finished_animations.read().unwrap();
    vec.clone()
}

#[profiling::function]
pub(crate) fn execute_transactions(engine: &Engine) -> (Vec<NodeRef>, Vec<FlatStorageId>, bool) {
    let updated_nodes = Arc::new(RwLock::new(Vec::<NodeRef>::new()));
    let transactions_finished = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));
    let transactions = engine.transactions.data();
    let transactions = transactions.write().unwrap();

    let needs_redraw = transactions.len() > 0;
    if needs_redraw {
        let animations = &engine.animations;
        let transaction_handlers = &engine.transaction_handlers;
        let scene = engine.scene.clone();
        transactions.par_iter().for_each_with(
            transactions_finished.clone(),
            |transactions_finished, (id, command)| {
                let (progress, done) = command
                    .animation_id
                    .as_ref()
                    .map(|id| {
                        animations
                            .get(&id.0)
                            .map(
                                |AnimationState {
                                     progress,
                                     is_finished,
                                     ..
                                 }| (progress, is_finished),
                            )
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                if let Some(ch) = transaction_handlers.get(id) {
                    let callbacks = &ch.on_update;
                    callbacks.iter().for_each(|callback| {
                        callback(progress);
                    });
                }
                let flags = command.change.execute(progress);

                let node_id = command.node_id;
                if let Some(node) = scene.get_node(node_id.0) {
                    {
                        if let Some(node) = try_get_node(node) {
                            updated_nodes.write().unwrap().push(node_id);
                            if done {
                                node.remove_flags(RenderableFlags::ANIMATING);
                            }
                            node.insert_flags(flags);
                        }
                    }
                }
                if done {
                    transactions_finished.write().unwrap().push(*id);
                }
            },
        );
    }
    let transactions_finished = transactions_finished.read().unwrap();
    let updated_nodes = updated_nodes.read().unwrap();
    (
        updated_nodes.clone(),
        transactions_finished.clone(),
        needs_redraw,
    )
}

#[profiling::function]
pub(crate) fn update_layout_tree(engine: &Engine) {
    {
        let arena = engine.scene.nodes.data();
        let arena = arena.read().unwrap();
        arena.iter().for_each(|node| {
            if node.is_removed() {
                return;
            }
            let scene_node = node.get();
            let size = scene_node.layer.model.size.value();
            let layout_node_id = scene_node.layout_node_id;
            engine.set_node_layout_size(layout_node_id, size);
        });
    };
    let mut layout = engine.layout_tree.write().unwrap();
    let layout_root = *engine.layout_root.read().unwrap();

    // if layout.dirty(layout_root).unwrap() {
    let scene_size = engine.scene.size.read().unwrap();

    layout
        .compute_layout(
            layout_root,
            Size {
                width: points(scene_size.x),
                height: points(scene_size.y),
            },
        )
        .unwrap();

    // }
}

pub(crate) fn update_node(
    arena: &TreeStorageData<SceneNode>,
    layout: &Taffy,
    node_id: NodeId,
    parent: Option<&RenderLayer>,
    damage: &mut skia_safe::Rect,
    parent_changed: bool,
) -> RenderLayer {
    let node = arena.get(node_id).unwrap().get();

    let node_layout = layout.layout(node.layout_node_id).unwrap();

    let mut bounds = {
        let render_layer = node.render_layer.read().unwrap();
        render_layer.transformed_bounds
    };

    let matrix = parent.map(|p| &p.transform);
    let _new_layout = node.layout_if_needed(node_layout, matrix);

    let new_bounds = {
        let render_layer = node.render_layer.read().unwrap();
        render_layer.transformed_bounds
    };
    let mut node_damage = node.repaint_if_needed();

    let layout_changed = bounds != new_bounds;

    if layout_changed || parent_changed {
        bounds.join(new_bounds);
        node_damage.join(bounds);
    }
    damage.join(node_damage);
    let parent_changed = parent_changed || !node_damage.is_empty();
    let mut render_layer = node.render_layer.write().unwrap();
    node_id.children(arena).for_each(|child| {
        let child_render_layer = update_node(
            arena,
            layout,
            child,
            Some(&render_layer),
            damage,
            parent_changed,
        );
        render_layer
            .bounds_with_children
            .join(child_render_layer.bounds_with_children);
    });

    render_layer.clone()
}
#[profiling::function]

pub(crate) fn update_nodes(engine: &Engine, _nodes_list: Vec<NodeRef>) -> skia_safe::Rect {
    // iterate in parallel over the nodes and
    // repaint if necessary
    let layout = engine.layout_tree.read().unwrap();
    let arena = engine.scene.nodes.data();
    let arena = arena.read().unwrap();

    let mut damage = *engine.damage.read().unwrap();

    let node = engine.scene_root.read().unwrap();

    if let Some(root_id) = *node {
        update_node(&arena, &layout, root_id.0, None, &mut damage, false);
    }

    damage
}
#[profiling::function]
pub(crate) fn trigger_callbacks(engine: &Engine) {
    let animations = engine.animations.data();
    let animations = animations.read().unwrap();
    animations
        .iter()
        .filter(|(_, AnimationState { is_finished, .. })| *is_finished)
        .for_each(|(id, AnimationState { .. })| {
            if let Some(handler) = engine.transaction_handlers.get(id) {
                let callbacks = &handler.on_finish;
                callbacks.iter().for_each(|callback| {
                    let callback = callback.clone();
                    callback(1.0);
                });
            }
        });
}
#[profiling::function]
pub(crate) fn cleanup_animations(engine: &Engine, finished_animations: Vec<FlatStorageId>) {
    let animations = engine.animations.data();
    let mut animations = animations.write().unwrap();
    let handlers = engine.transaction_handlers.data();
    let mut handlers = handlers.write().unwrap();

    let animations_finished_to_remove = finished_animations;
    for animation_id in animations_finished_to_remove.iter() {
        animations.remove(animation_id);
        handlers.remove(animation_id);
    }
}
#[profiling::function]
pub(crate) fn cleanup_transactions(engine: &Engine, finished_transations: Vec<FlatStorageId>) {
    let transactions = engine.transactions.data();
    let mut transactions = transactions.write().unwrap();
    for command_id in finished_transations.iter() {
        transactions.remove(command_id);
    }
}
#[profiling::function]
pub(crate) fn cleanup_nodes(engine: &Engine) -> skia_safe::Rect {
    let mut damage = skia_safe::Rect::default();
    let deleted = {
        let nodes = engine.scene.nodes.data();
        let nodes = nodes.read().unwrap();
        nodes
            .iter()
            .filter_map(|node_id| {
                if node_id.is_removed() {
                    return None;
                }
                let node = node_id.get();
                if node.is_deleted() {
                    let bounds = node.bounds_with_children();
                    damage.join(bounds);
                    Some(node.id())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    for id in deleted {
        engine.scene_remove_layer(id);
    }
    damage
}
