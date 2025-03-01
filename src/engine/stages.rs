use std::sync::{Arc, RwLock};

use indextree::NodeId;
use rayon::{
    iter::IntoParallelRefIterator,
    prelude::{IntoParallelRefMutIterator, ParallelIterator},
};
use taffy::{prelude::Size, style_helpers::length, TaffyTree};

#[cfg(feature = "debugger")]
use layers_debug_server::send_debugger_message;

use crate::{layers::layer::render_layer::RenderLayer, prelude::Layer};

use super::{
    node::SceneNode,
    storage::{FlatStorageId, TreeStorageData},
    AnimationState, Engine, NodeRef, Timestamp, TransactionCallback, TransitionCallbacks,
};

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
    let updated_nodes = Arc::new(RwLock::new(Vec::<NodeRef>::new()));
    let transactions_finished = Arc::new(RwLock::new(Vec::<FlatStorageId>::new()));

    let needs_redraw = engine.transactions.with_data_mut(|transactions| {
        let needs_redraw = !transactions.is_empty();
        if needs_redraw {
            let animations = engine.animations.data();
            let animations = &*animations.blocking_read();
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
                            let node = node.get_mut();
                            node.insert_flags(flags);
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
#[profiling::function]
pub(crate) fn nodes_for_layout(engine: &Engine) -> Vec<NodeRef> {
    if let Some(root_node) = engine.scene_root() {
        engine.scene.with_arena_mut(|arena| {
            let id = root_node.0;
            id.descendants(arena)
                .map(|node_id| node_id.into())
                // FIXME
                // .filter_map(|node_id| {
                //     let node = arena.get_mut(node_id).unwrap(); //.get();
                //     let layer = engine.get_layer(node_id).unwrap();
                //     if node.is_removed() {
                //         return None;
                //     }
                //     let scene_node = node.get_mut();
                //     let layout = engine.get_node_layout_style(layer.layout_id);
                //     if layout.position != taffy::style::Position::Absolute {
                //         scene_node.set_need_layout(true);
                //     }
                //     // follow a replicated node
                //     // it will paint continuosly
                //     if let Some(follow) = &*scene_node._follow_node.read().unwrap() {
                //         if let Some(_follow_node) = arena.get(follow.0) {
                //             // let follow_node = _follow_node.get();
                //             // scene_node.set_need_repaint(follow_node.needs_repaint());
                //             // scene_node.set_need_repaint(true);
                //         }
                //     }
                //     Some(node_id.into())
                // })
                .collect()
        })
    } else {
        vec![]
    }
}

#[profiling::function]
pub(crate) fn update_layout_tree(engine: &Engine) {
    {
        profiling::scope!("update_nodes_size");
        engine.scene.with_arena(|arena| {
            arena.iter().for_each(|node| {
                if node.is_removed() {}
                // FIXME
                // let scene_node = node.get();
                // let size = scene_node.layer.model.size.value();
                // let layout_node_id = scene_node.layout_node_id();
                // engine.set_node_layout_size(layout_node_id, size);
            });
        });
    };
    let mut layout = engine.layout_tree.blocking_write();
    let layout_root = *engine.layout_root.blocking_read();

    // FIXME
    // if layout.dirty(layout_root).unwrap() {
    let scene_size = engine.scene.size.read().unwrap();

    {
        profiling::scope!("compute_layout");
        layout
            .compute_layout(
                layout_root,
                Size {
                    width: length(scene_size.x),
                    height: length(scene_size.y),
                },
            )
            .unwrap();
    }

    // }
}

// this function recursively update the node picture and its children
// and returns the area of pixels that are changed compared to the previeous frame
#[allow(unused_assignments, unused_mut)]
#[profiling::function]
pub(crate) fn update_node(
    engine: &Engine,
    arena: &mut TreeStorageData<SceneNode>,
    layout: &TaffyTree,
    node_id: NodeId,
    parent: Option<&RenderLayer>,
    parent_changed: bool,
) -> (bool, skia::Rect) {
    // update the layout of the node
    let children: Vec<_> = node_id.children(arena).collect();

    let mut damaged = false;
    let mut node_damage = skia::Rect::default();
    let render_layer = {
        let node = arena.get_mut(node_id);

        if node.is_none() {
            return (false, skia::Rect::default());
        }

        let node = node.unwrap().get_mut();
        let layer = engine.get_layer(node_id).unwrap();
        let node_layout = layout.layout(layer.layout_id).unwrap();

        let mut transformed_bounds;
        let mut opacity;
        (transformed_bounds, opacity) = {
            let render_layer = &node.render_layer;
            (
                render_layer.global_transformed_bounds,
                render_layer.premultiplied_opacity,
            )
        };

        let cumulative_transform = parent.map(|p| &p.transform);
        let context_opacity = parent.map(|p| p.premultiplied_opacity).unwrap_or(1.0);

        let _new_layout = node.layout_if_needed(
            node_layout,
            layer.model.clone(),
            cumulative_transform,
            context_opacity,
            // arena,
        );
        // update the picture of the node
        node_damage = node.repaint_if_needed();

        let render_layer = node.render_layer();

        let new_transformed_bounds = render_layer.global_transformed_bounds;

        let repainted = !node_damage.is_empty();

        let layout_changed = transformed_bounds.width() != new_transformed_bounds.width()
            || transformed_bounds.height() != new_transformed_bounds.height();

        let pos_changed = transformed_bounds.x() != new_transformed_bounds.x()
            || transformed_bounds.y() != new_transformed_bounds.y();

        let opacity_changed = opacity != render_layer.premultiplied_opacity;

        if (pos_changed && !transformed_bounds.is_empty())
            && render_layer.premultiplied_opacity > 0.0
            || opacity_changed
        {
            node_damage.join(node.repaint_damage);
            node_damage.join(new_transformed_bounds);

            node.repaint_damage = new_transformed_bounds;
        }
        damaged = layout_changed || repainted || parent_changed;

        let render_layer = node.render_layer();

        render_layer.clone()
    };

    let (damaged, node_damage) = children
        .iter()
        .map(|child| {
            // println!("**** map ({}) ", child);
            let (child_damaged, child_damage) = update_node(
                engine,
                arena,
                layout,
                *child,
                Some(&render_layer.clone()),
                parent_changed,
            );
            // damaged = damaged || child_repainted || child_relayout;
            (child_damaged, child_damage)
        })
        .fold(
            (damaged, node_damage),
            |(damaged, node_damage), (child_damaged, child_damage)| {
                // update the bounds of the node to include the children

                // FIXME: update bounds_with_children

                // node.render_layer
                //     .global_transformed_bounds_with_children
                //     .join(r.global_transformed_bounds_with_children);

                // let (child_bounds, _) = r.local_transform.to_m33().map_rect(r.bounds_with_children);
                // let child_bounds = r.bounds_with_children;
                // println!(
                //     "({}) fold: child_bounds mapped: {:?}",
                //     node_id, child_bounds
                // );

                // render_layer.bounds_with_children.join(child_bounds);

                let node_damage = skia::Rect::join2(node_damage, child_damage);
                (damaged || child_damaged, node_damage)
            },
        );

    // if the node has some drawing in it, and has changed size or position
    // we need to repaint
    // let last_repaint_damage = node.repaint_damage.read().unwrap();
    // if !last_repaint_damage.is_empty() && (layout_changed || pos_changed || parent_changed) {
    //     transformed_bounds.join(new_transformed_bounds);
    //     node_damage.join(transformed_bounds);
    // }
    if damaged {
        // if !node_damage.is_empty() {
        if let Some(node) = arena.get_mut(node_id) {
            let node = node.get_mut();
            node.increase_frame();
        }
    }
    // let render_layer = node.render_layer.read().unwrap();
    (damaged, node_damage)
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
                let layer = engine.get_layer(command.node_id).unwrap();
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
                let mut values_transactions = engine.values_transactions.blocking_write();
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
        let root = engine.scene_root().unwrap();
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
        engine.scene_remove_layer(id);
    }
    damage
}

#[cfg(feature = "debugger")]
pub fn send_debugger(scene: Arc<crate::engine::scene::Scene>, scene_root: NodeRef) {
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
