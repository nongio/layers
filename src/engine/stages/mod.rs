use std::sync::{Arc, RwLock};

use rayon::{
    iter::IntoParallelRefIterator,
    prelude::{IntoParallelRefMutIterator, ParallelIterator},
};
use taffy::{prelude::Size, style_helpers::length, Dimension};

#[cfg(feature = "debugger")]
use layers_debug_server::send_debugger_message;

use crate::{engine::node::RenderableFlags, prelude::Layer};

use super::{
    storage::FlatStorageId, AnimationState, Engine, NodeRef, Timestamp, TransactionCallback,
    TransitionCallbacks,
};

mod update_node;

#[allow(unused_imports)]
pub(crate) use update_node::update_node_single;

#[profiling::function]
/// This function updates the animations in the engine, in parallel.
pub(crate) fn update_animations(
    engine: &Engine,
    timestamp: &Timestamp,
) -> (Vec<FlatStorageId>, Vec<FlatStorageId>) {
    let finished_animations = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));
    let started_animations = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));

    engine.animations.with_data_mut(|animations| {
        if !animations.is_empty() {
            animations.par_iter_mut().for_each_with(
                (finished_animations.clone(), started_animations.clone()),
                |(done_animations, started_animations),
                 (
                    id,
                    AnimationState {
                        animation,
                        progress,
                        time,
                        is_running,
                        is_finished,
                        is_started,
                    },
                )| {
                    if !*is_running {
                        return;
                    }
                    let (animation_progress, time_progress) = animation.update_at(timestamp.0);
                    if !(*is_started) && animation.start <= timestamp.0 {
                        *is_started = true;
                        started_animations.write().unwrap().push(*id);
                    }
                    *progress = animation_progress;
                    *time = time_progress.clamp(0.0, time_progress);
                    if animation.done(timestamp.0) {
                        *is_running = false;
                        *is_finished = true;
                        done_animations.write().unwrap().push(*id);
                    }
                },
            );
        }
    });

    let finished = finished_animations.read().unwrap();
    let started = started_animations.read().unwrap();
    (started.clone(), finished.clone())
}

#[profiling::function]
/// This function executes the transactions in the engine, in parallel.
pub(crate) fn execute_transactions(engine: &Engine) -> (Vec<NodeRef>, Vec<FlatStorageId>, bool) {
    let updated_nodes = Arc::new(std::sync::RwLock::new(Vec::<NodeRef>::new()));
    let transactions_finished = Arc::new(std::sync::RwLock::new(Vec::<FlatStorageId>::new()));

    let needs_redraw = engine.transactions.with_data_mut(|transactions| {
        let needs_redraw = !transactions.is_empty();
        if needs_redraw {
            let animations = engine.animations.data();
            let animations = &*animations.read().unwrap();
            let scene = engine.scene();

            // iterate in parallel over all the changes to be applied
            transactions.par_iter().for_each_with(
                (
                    animations,
                    updated_nodes.clone(),
                    scene,
                    transactions_finished.clone(),
                ),
                |(animations, updated_nodes, scene, transactions_finished), (id, command)| {
                    let animation_state = command
                        .animation_id
                        .as_ref()
                        .and_then(|id| animations.get(&id.0).cloned())
                        .unwrap_or(AnimationState {
                            animation: Default::default(),
                            progress: 1.0,
                            time: 0.0,
                            is_running: false,
                            is_finished: true,
                            is_started: false,
                        });
                    // apply the changes
                    let flags = command.change.execute(animation_state.progress);

                    let node_id = command.node_id;
                    updated_nodes.write().unwrap().push(node_id);
                    scene.with_arena_mut(|arena| {
                        if let Some(node) = arena.get_mut(node_id.0) {
                            if !node.is_removed() {
                                let scene_node = node.get_mut();
                                scene_node.insert_flags(flags);
                            }
                        }
                    });
                    if animation_state.is_finished {
                        transactions_finished.write().unwrap().push(*id);
                    }
                },
            );
        }
        needs_redraw
    });

    let transactions_finished = transactions_finished.read().unwrap();
    let updated_nodes = updated_nodes.read().unwrap();
    (
        updated_nodes.clone(),
        transactions_finished.clone(),
        needs_redraw,
    )
}
/// Collects all valid nodes that are eligible for layout computation.
/// This function traverses the scene tree from the root and filters out
/// nodes that have been removed or marked for deletion, returning only
/// the nodes that should participate in the layout pass.
#[profiling::function]
pub(crate) fn nodes_for_layout(engine: &Engine) -> Vec<NodeRef> {
    if let Some(root_node) = engine.scene_root() {
        let descendants = engine.node_descendants(&root_node);

        descendants
            .iter()
            .filter_map(|node_ref| {
                let node = engine.scene().with_arena(|arena| {
                    let node = arena.get(node_ref.0).unwrap();
                    if node.is_removed() {
                        return None;
                    }
                    let scene_node = node.get();
                    if scene_node.is_deleted() {
                        return None;
                    }

                    Some(*node_ref)
                });

                node
            })
            .collect()
    } else {
        vec![]
    }
}

#[profiling::function]
pub(crate) fn update_layout_tree(engine: &Engine) {
    // Get scene size early to avoid multiple lock acquisitions
    let scene_size = engine.scene.size.read().unwrap();
    let mut changed_nodes = Vec::new();
    let mut followers_nodes = Vec::new();

    // First, collect nodes that need size updates
    {
        profiling::scope!("collect_nodes_needing_update");
        if let Some(root) = engine.scene_root() {
            engine.scene.with_arena(|arena| {
                // Traverse all descendants from the root node to identify nodes that need layout updates
                for node_id in root.0.descendants(arena) {
                    let node = arena.get(node_id).unwrap();
                    if node.is_removed() {
                        return;
                    }
                    let node_ref = NodeRef(node_id);

                    // Update layout size for nodes that have explicit size constraints
                    engine.with_layers(|layers| {
                        if let Some(layer) = layers.get(&node_ref) {
                            let size = layer.size();
                            let layout_node_id = layer.layout_id;
                            if size.width != Dimension::Auto || size.height != Dimension::Auto {
                                engine.set_node_layout_size(layout_node_id, size);
                            }
                        }
                    });

                    // Check if this node needs layout recalculation and collect follower relationships
                    let scene_node = node.get();
                    if let Some(follow) = scene_node._follow_node {
                        followers_nodes.push((node_ref, follow));
                    }
                    let needs_layout = scene_node.needs_layout()
                        || true
                        || scene_node.render_layer.blend_mode
                            == crate::types::BlendMode::BackgroundBlur;

                    if needs_layout {
                        changed_nodes.push(node_ref);
                    }
                }
            });
        }
    }

    // profiling::scope!("update_nodes_size");
    // for node_ref in &changed_nodes {
    //     engine.scene.with_arena_mut(|arena| {
    //         if let Some(node) = arena.get_mut(node_ref.0) {
    //             let scene_node = node.get_mut();
    //             scene_node.set_needs_layout(false);
    //             scene_node.set_needs_repaint(true);
    //         }
    //     });
    // }

    for (node_ref, follow) in &followers_nodes {
        engine.scene.with_arena_mut(|arena| {
            let follow_node = arena.get(follow.0);
            let needs_repaint_follow = if let Some(follow_node) = follow_node {
                let follow_node = follow_node.get();
                follow_node.needs_repaint()
            } else {
                false
            };
            if let Some(node) = arena.get_mut(node_ref.0) {
                let scene_node = node.get_mut();
                if needs_repaint_follow {
                    scene_node.set_needs_repaint(true);
                }
            }
        });
    }

    // Now check if we need to compute layout
    let mut layout = engine.layout_tree.write().unwrap();
    let layout_root = *engine.layout_root.read().unwrap();

    // Only compute layout if nodes have changed or if the root is dirty
    let needs_layout = !changed_nodes.is_empty() || layout.dirty(layout_root).unwrap_or(false);

    if needs_layout {
        profiling::scope!("compute_layout");
        match layout.compute_layout(
            layout_root,
            Size {
                width: length(scene_size.x),
                height: length(scene_size.y),
            },
        ) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Layout computation failed: {:?}", e);
            }
        }
    }
}

#[profiling::function]
pub(crate) fn trigger_callbacks(engine: &Engine, started_animations: &[FlatStorageId]) {
    engine.transactions.with_data_cloned(|transactions| {
        let scene = engine.scene.clone();
        transactions.iter().for_each(|(transaction_id, command)| {
            let animation_state = command
                .animation_id
                .as_ref()
                .and_then(|animation_id| engine.animations.get(&animation_id.0))
                .unwrap_or(AnimationState {
                    animation: Default::default(),
                    progress: 1.0,
                    time: 0.0,
                    is_running: false,
                    is_finished: true,
                    is_started: false,
                });
            if !scene.is_node_removed(command.node_id.0) {
                let layer = engine.get_layer(&command.node_id).unwrap();
                let tcallbacks = { engine.transaction_handlers.get(transaction_id) };

                let started = command
                    .animation_id
                    .map(|a| started_animations.contains(&a.0))
                    .unwrap_or(false);
                let vcallbacks = { engine.value_handlers.get(&command.change.value_id()) };
                let (tcallback_to_remove, vcallback_to_remove) = transaction_callbacks(
                    &animation_state,
                    tcallbacks.as_ref(),
                    vcallbacks.as_ref(),
                    &layer,
                    started,
                );
                {
                    engine
                        .transaction_handlers
                        .with_data_mut(|transatction_handlers| {
                            if let Some(handler) = transatction_handlers.get_mut(transaction_id) {
                                tcallback_to_remove.iter().for_each(|tr_callback| {
                                    handler.remove(tr_callback);
                                });
                            }
                        });
                    engine.value_handlers.with_data_mut(|values_handlers| {
                        if let Some(handler) = values_handlers.get_mut(&command.change.value_id()) {
                            vcallback_to_remove.iter().for_each(|v_callback| {
                                handler.remove(v_callback);
                            });
                        }
                    });
                }
            }
        });
    });
}

#[profiling::function]
fn transaction_callbacks(
    animation_state: &AnimationState,
    tr_handler: Option<&TransitionCallbacks>,
    v_handler: Option<&TransitionCallbacks>,
    layer: &Layer,
    on_start: bool,
) -> (Vec<TransactionCallback>, Vec<TransactionCallback>) {
    let mut tr_to_remove: Vec<TransactionCallback> = Vec::new();
    let mut v_to_remove: Vec<TransactionCallback> = Vec::new();

    if animation_state.is_running {
        if on_start {
            if let Some(tr_handler) = tr_handler {
                let callbacks = &tr_handler.on_start;
                callbacks.iter().for_each(|tr_callback| {
                    let callback = &tr_callback.callback;
                    callback(layer, animation_state.time);
                    if tr_callback.once {
                        tr_to_remove.push(tr_callback.clone());
                    }
                });
            }
            if let Some(v_handler) = v_handler {
                let callbacks = &v_handler.on_start;
                callbacks.iter().for_each(|tr_callback| {
                    let callback = &tr_callback.callback;
                    callback(layer, animation_state.time);
                    if tr_callback.once {
                        v_to_remove.push(tr_callback.clone());
                    }
                });
            }
        }
        if let Some(tr_handler) = tr_handler {
            let callbacks = &tr_handler.on_update;
            callbacks.iter().for_each(|tr_callback| {
                let callback = &tr_callback.callback;
                callback(layer, animation_state.time);
                if tr_callback.once {
                    tr_to_remove.push(tr_callback.clone());
                }
            });
        }
        if let Some(v_handler) = v_handler {
            let callbacks = &v_handler.on_update;
            callbacks.iter().for_each(|tr_callback| {
                let callback = &tr_callback.callback;
                callback(layer, animation_state.time);
                if tr_callback.once {
                    v_to_remove.push(tr_callback.clone());
                }
            });
        }
    } else if animation_state.is_finished {
        if let Some(tr_handler) = tr_handler {
            let callbacks = &tr_handler.on_update;
            callbacks.iter().for_each(|tr_callback| {
                let callback = &tr_callback.callback;
                callback(layer, 1.0);
                if tr_callback.once {
                    tr_to_remove.push(tr_callback.clone());
                }
            });
        }
        if let Some(v_handler) = v_handler {
            let callbacks = &v_handler.on_update;
            callbacks.iter().for_each(|tr_callback| {
                let callback = &tr_callback.callback;
                callback(layer, animation_state.time);
                if tr_callback.once {
                    v_to_remove.push(tr_callback.clone());
                }
            });
        }
        if let Some(tr_handler) = tr_handler {
            let callbacks = &tr_handler.on_finish;
            callbacks.iter().for_each(|tr_callback| {
                let callback = &tr_callback.callback;
                if tr_callback.once {
                    tr_to_remove.push(tr_callback.clone());
                }
                callback(layer, 1.0);
            });
        }
        if let Some(v_handler) = v_handler {
            let callbacks = &v_handler.on_finish;
            callbacks.iter().for_each(|tr_callback| {
                let callback = &tr_callback.callback;
                callback(layer, animation_state.time);
                if tr_callback.once {
                    v_to_remove.push(tr_callback.clone());
                }
            });
        }
    }
    (tr_to_remove, v_to_remove)
}

#[profiling::function]
pub(crate) fn cleanup_animations(engine: &Engine, finished_animations: Vec<FlatStorageId>) {
    engine.animations.with_data_mut(|animations| {
        let animations_finished_to_remove = finished_animations;
        for animation_id in animations_finished_to_remove.iter() {
            animations.remove(animation_id);
        }
    });
}

#[profiling::function]
pub(crate) fn cleanup_transactions(engine: &Engine, finished_transations: Vec<FlatStorageId>) {
    engine.transactions.with_data_mut(|transactions| {
        for tid in finished_transations.iter() {
            if let Some(tr) = transactions.get(tid) {
                let vid = tr.change.value_id();
                transactions.remove(tid);
                let mut values_transactions = engine.values_transactions.write().unwrap();
                if let Some(existing_tid) = values_transactions.get(&vid) {
                    if (*existing_tid) == *tid {
                        values_transactions.remove(&vid);
                    }
                }
                engine.value_handlers.with_data_mut(|handlers| {
                    if let Some(handler) = handlers.get_mut(&vid) {
                        handler.cleanup_once_callbacks();
                    }
                });
            }
            engine.transaction_handlers.with_data_mut(|handlers| {
                if let Some(handler) = handlers.get_mut(tid) {
                    handler.cleanup_once_callbacks();
                }
            });
        }
    });
}

#[profiling::function]
#[allow(clippy::unnecessary_filter_map)]
pub(crate) fn cleanup_nodes(engine: &Engine) -> skia_safe::Rect {
    let mut damage = skia_safe::Rect::default();
    let deleted = {
        let root = engine.scene_root();

        if root.is_none() {
            return damage;
        }
        let root = root.unwrap();
        engine.scene.with_arena(|arena| {
            root.0
                .descendants(arena)
                .filter_map(|node_id| {
                    if node_id.is_removed(arena) {
                        return None;
                    }
                    let node = arena.get(node_id).unwrap().get();

                    if node.is_deleted() {
                        let bounds = node.transformed_bounds_with_effects();
                        damage.join(bounds);
                        Some(node_id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
    };
    for id in deleted {
        engine.scene_remove_layer(&NodeRef(id));
    }
    damage
}

#[cfg(feature = "debugger")]
pub fn send_debugger(scene: Arc<crate::engine::scene::Scene>, scene_root: NodeRef) {
    use indextree::NodeId;

    use crate::layers::layer::render_layer::RenderLayer;

    let s = scene.clone();
    s.with_arena(|arena| {
        let render_layers: std::collections::HashMap<
            usize,
            (usize, RenderLayer, Vec<usize>, NodeId),
        > = arena
            .iter()
            .filter_map(|node| {
                if node.is_removed() {
                    return None;
                }
                let scene_node = node.get();
                let node_id = arena.get_node_id(node).unwrap();
                let children = node_id
                    .children(arena)
                    .map(|child| child.into())
                    .collect::<Vec<usize>>();
                let render_layer = scene_node.render_layer.clone();
                let id: usize = node_id.into();
                Some((id, (id, render_layer, children, node_id)))
            })
            .collect();
        let root: usize = scene_root.0.into();

        let data = (root, render_layers);
        send_debugger_message(serde_json::to_string(&data).unwrap());
    });
}
