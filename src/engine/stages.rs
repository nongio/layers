use std::sync::{Arc, RwLock};

use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use taffy::{prelude::Size, style_helpers::points};

use super::{
    node::{DrawCacheManagement, RenderableFlags},
    storage::FlatStorageId,
    AnimationState, Engine, NodeRef, Timestamp,
};

pub(crate) fn update_animations(engine: &Engine, timestamp: Timestamp) -> Vec<FlatStorageId> {
    let finished_animations = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));

    let animations = engine.animations.data();
    let mut animations = animations.write().unwrap();
    if animations.len() > 0 {
        animations.par_iter_mut().for_each_with(
            finished_animations.clone(),
            |done_animations, (id, AnimationState(animation, value, finished))| {
                (*value, *finished) = animation.value(timestamp.0);
                if *finished {
                    done_animations.clone().write().unwrap().push(*id);
                }
            },
        );
    }

    let vec = finished_animations.read().unwrap();
    vec.clone()
}

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
                            .map(|AnimationState(_, value, done)| (value, done))
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                if let Some(ch) = transaction_handlers.get(id) {
                    let callbacks = &ch.on_update;
                    callbacks.iter().for_each(|callback| {
                        let callback = callback.clone();
                        callback(progress);
                    });
                }
                let flags = command.change.execute(progress);
                let node_id = command.node_id;
                if let Some(node) = scene.get_node(node_id.0) {
                    {
                        let node = node.get();
                        if flags.contains(RenderableFlags::NEEDS_LAYOUT) {
                            let bounds = node.model.bounds();
                            let size = crate::types::Size {
                                x: bounds.width,
                                y: bounds.height,
                            };
                            engine.set_node_layout_size(node.layout_node, size);
                        }

                        updated_nodes.write().unwrap().push(node_id);
                        if done {
                            node.remove_flags(RenderableFlags::ANIMATING);
                        }
                    }
                    node.get().insert_flags(flags);
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

pub(crate) fn update_layout_tree(engine: &Engine) {
    let mut layout = engine.layout_tree.write().unwrap();
    let layout_root = *engine.layout_root.read().unwrap();

    if layout.dirty(layout_root).unwrap() {
        let scene_root = *engine.scene_root.read().unwrap().unwrap();
        let scene_root = engine.scene.get_node(scene_root).unwrap();

        let scene_root = scene_root.get();
        let bounds = scene_root.model.bounds();
        layout
            .compute_layout(
                layout_root,
                Size {
                    width: points(bounds.width),
                    height: points(bounds.height),
                },
            )
            .unwrap();
    }
}

pub(crate) fn update_nodes(engine: &Engine, nodes_list: Vec<NodeRef>) {
    // iterate in parallel over the nodes and
    // repaint if necessary
    let layout = engine.layout_tree.read().unwrap();
    let arena = engine.scene.nodes.data();
    let arena = arena.read().unwrap();
    nodes_list.par_iter().for_each(|node_id| {
        let node = arena.get(node_id.0).unwrap().get();
        let l = layout.layout(node.layout_node).unwrap();
        node.layout_if_needed(l);
        node.repaint_if_needed();
    });
}

pub(crate) fn trigger_callbacks(engine: &Engine) {
    let animations = engine.animations.data();
    let animations = animations.read().unwrap();
    animations
        .iter()
        .filter(|(_, AnimationState(_, _, finished))| *finished)
        .for_each(|(id, AnimationState(_animation, _, _))| {
            if let Some(handler) = engine.transaction_handlers.get(id) {
                let callbacks = &handler.on_finish;
                callbacks.iter().for_each(|callback| {
                    let callback = callback.clone();
                    callback(1.0);
                });
            }
        });
}

pub(crate) fn cleanup_animations(engine: &Engine, finished_animations: Vec<FlatStorageId>) {
    let animations = engine.animations.data();
    let mut animations = animations.write().unwrap();

    let animations_finished_to_remove = finished_animations;
    for animation_id in animations_finished_to_remove.iter() {
        animations.remove(animation_id);
    }
}

pub(crate) fn cleanup_transactions(engine: &Engine, finished_transations: Vec<FlatStorageId>) {
    let handlers = engine.transaction_handlers.data();
    let mut handlers = handlers.write().unwrap();
    let transactions = engine.transactions.data();
    let mut transactions = transactions.write().unwrap();
    for command_id in finished_transations.iter() {
        transactions.remove(command_id);
        handlers.remove(command_id);
    }
}