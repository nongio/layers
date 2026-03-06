use indextree::{Arena, NodeId};
use taffy::TaffyTree;

use crate::{
    engine::{node::do_repaint, storage::TreeStorageData, *},
    layers::layer::render_layer::RenderLayer,
};

#[derive(Debug, Clone)]
pub(crate) struct NodeUpdateResult {
    pub damage: skia::Rect,
    /// Whether this node's transform/opacity change should force children to recompute.
    pub propagate_to_children: bool,
}

fn subtree_has_visible_drawables(arena: &Arena<SceneNode>, node_id: NodeId) -> bool {
    let mut stack = vec![node_id];
    while let Some(id) = stack.pop() {
        if let Some(node) = arena.get(id) {
            let scene_node = node.get();
            if scene_node.hidden() {
                continue;
            }
            if scene_node.render_layer.has_visible_drawables() {
                return true;
            }
            id.children(arena).for_each(|child_id| stack.push(child_id));
        }
    }
    false
}

// This function updates a single node's RenderLayer, repaints if needed, and calculates damage.
// It compares the node's state before and after updates to determine what screen areas need
// to be redrawn, taking into account position, size, opacity, and content changes.
#[profiling::function]
pub(crate) fn update_node_single(
    engine: &Engine,
    layout_tree: &TaffyTree,
    node_id: NodeId,
    parent: Option<&RenderLayer>,
    parent_changed: bool,
) -> NodeUpdateResult {
    let Some(layer) = engine.get_layer(&NodeRef(node_id)) else {
        return NodeUpdateResult {
            damage: skia::Rect::default(),
            propagate_to_children: false,
        };
    };
    let node_layout = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        layout_tree.layout(layer.layout_id).cloned()
    })) {
        Ok(Ok(layout)) => layout,
        Ok(Err(_)) | Err(_) => {
            tracing::warn!(
                "update_node_single: invalid layout_id on node {:?}, marking for deletion",
                node_id
            );
            // Mark the node for deletion so cleanup_nodes removes it from the scene tree
            // instead of leaving it stuck with a broken layout forever.
            engine.mark_for_delete(NodeRef(node_id));
            return NodeUpdateResult {
                damage: skia::Rect::default(),
                propagate_to_children: false,
            };
        }
    };

    // First, read the previous state for comparisons
    let (
        prev_transformed_bounds,
        prev_global_bounds,
        prev_opacity,
        prev_visible,
        prev_needs_paint,
        prev_has_filters,
        prev_is_layout_only_passthrough,
        // render_layer.visible accounts for the hidden flag (set by update_render_layer_if_needed);
        // this lets us detect hidden-state changes separately from drawable-content changes.
        prev_render_layer_visible,
    ) = engine.scene.with_arena(|arena| {
        if let Some(node) = arena.get(node_id) {
            let scene_node = node.get();
            (
                scene_node
                    .render_layer
                    .global_transformed_bounds_with_children,
                scene_node.render_layer.global_transformed_bounds,
                scene_node.render_layer.premultiplied_opacity,
                scene_node.render_layer.has_visible_drawables(),
                scene_node.needs_repaint(),
                scene_node.render_layer.has_filters(),
                scene_node.render_layer.is_layout_only_passthrough(),
                scene_node.render_layer.visible,
            )
        } else {
            (
                skia_safe::Rect::default(),
                skia_safe::Rect::default(),
                0.0,
                false,
                false,
                false,
                false,
                false,
            )
        }
    });

    let prev_children_visible = engine.scene.with_arena(|arena| {
        node_id
            .children(arena)
            .any(|child_id| subtree_has_visible_drawables(arena, child_id))
    });

    // Aggregate children bounds in the node's local space
    let local_children_bounds = engine.scene.with_arena(|arena| {
        let mut bounds = skia::Rect::default();
        node_id.children(arena).for_each(|child_id| {
            if let Some(child) = arena.get(child_id) {
                bounds.join(
                    child
                        .get()
                        .render_layer
                        .local_transformed_bounds_with_children,
                );
            }
        });
        bounds
    });

    // Account for parent transform/opacity
    let cumulative_transform = parent.map(|p| &p.transform);
    let context_opacity = parent.map(|p| p.premultiplied_opacity).unwrap_or(1.0);

    // Ensure nodes refresh when only the parent opacity changes.
    engine.scene.with_arena_mut(|arena| {
        if let Some(node) = arena.get_mut(node_id) {
            let scene_node = node.get_mut();
            let prev_premultiplied = scene_node.render_layer.premultiplied_opacity;
            let context_premultiplied = scene_node.render_layer.opacity * context_opacity;
            if (prev_premultiplied - context_premultiplied).abs() > f32::EPSILON {
                scene_node.set_needs_repaint(true);
            }
        }
    });

    // Update the render layer using the latest model/layout state
    let (changed_render_layer, is_debug) = engine.scene.with_arena_mut(|arena| {
        arena
            .get_mut(node_id)
            .map(|node| {
                let scene_node = node.get_mut();
                let changed = scene_node.update_render_layer_if_needed(
                    &node_layout,
                    layer.model.clone(),
                    cumulative_transform,
                    context_opacity,
                    local_children_bounds,
                    parent_changed,
                ) || scene_node._debug_info.is_some();

                if changed {
                    scene_node.set_needs_repaint(true);
                }

                (changed, scene_node._debug_info.is_some())
            })
            .unwrap_or((false, false))
    });

    // Capture the new state after the update
    let (
        new_transformed_bounds,
        new_global_bounds,
        new_opacity,
        new_visible,
        current_needs_paint,
        new_has_filters,
        new_is_layout_only_passthrough,
        is_now_hidden,
    ) = engine.scene.with_arena(|arena| {
        let node = arena.get(node_id).unwrap();
        let scene_node = node.get();
        (
            scene_node
                .render_layer
                .global_transformed_bounds_with_children,
            scene_node.render_layer.global_transformed_bounds,
            scene_node.render_layer.premultiplied_opacity,
            scene_node.render_layer.has_visible_drawables(),
            scene_node.needs_repaint(),
            scene_node.render_layer.has_filters(),
            scene_node.render_layer.is_layout_only_passthrough(),
            scene_node.hidden(),
        )
    });

    let new_children_visible = engine.scene.with_arena(|arena| {
        node_id
            .children(arena)
            .any(|child_id| subtree_has_visible_drawables(arena, child_id))
    });

    let prev_effective_visible = prev_visible || prev_children_visible;
    let new_effective_visible = new_visible || new_children_visible;

    // Determine which properties changed
    let layout_changed_self = prev_global_bounds.width() != new_global_bounds.width()
        || prev_global_bounds.height() != new_global_bounds.height();
    let position_changed_self = prev_global_bounds.x() != new_global_bounds.x()
        || prev_global_bounds.y() != new_global_bounds.y();
    let layout_changed_children = prev_transformed_bounds.width() != new_transformed_bounds.width()
        || prev_transformed_bounds.height() != new_transformed_bounds.height();
    let position_changed_children = prev_transformed_bounds.x() != new_transformed_bounds.x()
        || prev_transformed_bounds.y() != new_transformed_bounds.y();

    let geometry_changed_self = layout_changed_self || position_changed_self;
    let geometry_changed_children = layout_changed_children || position_changed_children;
    let layout_changed = layout_changed_self || layout_changed_children;
    let position_changed = position_changed_self || position_changed_children;
    let opacity_changed = prev_opacity != new_opacity;
    let visibility_changed = prev_effective_visible != new_effective_visible;

    // If nothing relevant changed, bail out early
    if !parent_changed
        && !prev_needs_paint
        && !current_needs_paint
        && !layout_changed
        && !position_changed
        && !opacity_changed
        && !changed_render_layer
        && !visibility_changed
    {
        return NodeUpdateResult {
            damage: skia_safe::Rect::default(),
            propagate_to_children: parent_changed,
        };
    }
    let changed_filters = prev_has_filters != new_has_filters;

    // A node is passthrough-only when both before and after the update it has no own
    // visible drawables, no filters, no clipping, and uses Normal blend mode.
    // In that case geometry changes on the node itself don't produce damage – only
    // the children's damage matters.
    let passthrough_only = prev_is_layout_only_passthrough && new_is_layout_only_passthrough;

    let mut updated_renderable = None;
    let content_damage = engine.scene.with_arena(|arena| {
        let opt_renderable = engine.scene.renderables.get(&node_id.into());
        let node = arena.get(node_id);
        if let (Some(node), Some(renderable)) = (node, opt_renderable) {
            let scene_node = node.get();
            let mut repaint_damage = skia_safe::Rect::default();
            if scene_node.needs_repaint()
                || parent_changed
                || layout_changed
                || position_changed
                || opacity_changed
            // || changed_filters
            {
                let new_renderable = do_repaint(&renderable, scene_node);
                repaint_damage = new_renderable.repaint_damage;
                updated_renderable = Some(new_renderable);
            }
            repaint_damage
        } else {
            skia_safe::Rect::default()
        }
    });

    if let Some(renderable) = updated_renderable {
        engine
            .scene
            .renderables
            .insert_with_id(renderable, node_id.into());
    }

    // Clear repaint/layout flags now that the node has been updated
    engine.scene.with_arena_mut(|arena| {
        if let Some(node) = arena.get_mut(node_id) {
            let scene_node = node.get_mut();
            scene_node.set_needs_repaint(false);
            scene_node.set_needs_layout(false);
        }
    });

    // Map content damage to global coordinates
    let (mapped_content_damage, _) = engine.scene.with_arena(|arena| {
        let node = arena.get(node_id).unwrap();
        node.get()
            .render_layer
            .transform_33
            .map_rect(content_damage)
    });

    let mut total_damage = mapped_content_damage;
    let has_visible_drawables = prev_effective_visible || new_effective_visible;
    let self_contributes_visual_output = !passthrough_only;

    if geometry_changed_self
        && ((has_visible_drawables && self_contributes_visual_output) || is_debug)
    {
        total_damage.join(prev_global_bounds);
        total_damage.join(new_global_bounds);
    }

    if geometry_changed_children && (has_visible_drawables || is_debug) && !passthrough_only {
        total_damage.join(prev_transformed_bounds);
        total_damage.join(new_transformed_bounds);
    }

    // When a layer loses all drawable content, damage its previous bounds so the
    // compositor redraws the area it occupied.
    if prev_effective_visible && !new_effective_visible {
        total_damage.join(prev_transformed_bounds);
    }

    // Hidden-state changes: use render_layer.visible (reflects the hidden flag) to distinguish
    // a visibility change due to set_hidden() from a change due to gaining/losing drawables.
    // - visible→hidden: damage previous bounds (area must be cleared)
    // - hidden→visible: damage new bounds (area must be drawn)
    if prev_render_layer_visible && is_now_hidden {
        total_damage.join(prev_transformed_bounds);
    }
    if !prev_render_layer_visible && prev_visible && !is_now_hidden {
        // The layer had drawables before (prev_visible=true) but was suppressed by the hidden
        // flag (prev_render_layer_visible=false); it is now un-hidden, so damage its new bounds.
        total_damage.join(new_transformed_bounds);
    }

    if opacity_changed && has_visible_drawables && !passthrough_only {
        if prev_opacity <= 0.0 && new_opacity > 0.0 {
            total_damage.join(new_transformed_bounds);
        } else if prev_opacity > 0.0 && new_opacity <= 0.0 {
            total_damage.join(prev_transformed_bounds);
        } else {
            total_damage.join(new_transformed_bounds);
        }
    }
    // When only a filter is added/removed and the node has no own visible drawables
    // (pure filter container), do_repaint produces no picture so content_damage is empty.
    // We must still damage the full subtree bounds so the compositor redraws the children.
    if changed_filters && has_visible_drawables && content_damage.is_empty() {
        total_damage.join(prev_transformed_bounds);
        total_damage.join(new_transformed_bounds);
    }
    let content_repainted = !content_damage.is_empty();
    let damaged = content_repainted
        || (geometry_changed_self && (has_visible_drawables || is_debug))
        || (geometry_changed_children && (has_visible_drawables || is_debug))
        || opacity_changed
        || parent_changed
        || visibility_changed
        || changed_render_layer
        || changed_filters;

    if damaged {
        engine.scene.with_arena_mut(|arena| {
            if let Some(node) = arena.get_mut(node_id) {
                node.get_mut().increase_frame();
            }
        });
    }

    let propagate_to_children = parent_changed || geometry_changed_self || opacity_changed;
    let nid: usize = node_id.into();
    if nid == 8 && total_damage.width() == 1000.0 {
        println!("update_node_single: node_id={:?}, damage={:?}, layout_changed={}, position_changed={}, opacity_changed={}, visibility_changed={}, changed_render_layer={}, prev_has_filters={}, new_has_filters={}, propagate_to_children={}", nid, total_damage, layout_changed, position_changed, opacity_changed, visibility_changed, changed_render_layer, prev_has_filters, new_has_filters, propagate_to_children);
    }
    NodeUpdateResult {
        damage: total_damage,
        propagate_to_children,
    }
}

#[cfg(test)]
mod tests {
    use super::update_node_single;
    use crate::engine::stages::{execute_transactions, update_layout_tree};
    use crate::engine::Engine;
    use crate::types::{Color, PaintColor, Size};

    // Helper: run the minimal pipeline for a single node after scheduling changes
    fn apply_changes_and_update_layout(engine: &Engine) {
        let _ = execute_transactions(engine);
        update_layout_tree(engine);
    }

    #[test]
    // Ensures a layer position change returns the union of old and new bounds.
    fn update_node_single_position_change_damages_union() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&layer);

        // Prime initial state so previous bounds are non-empty
        engine.update(0.016);
        engine.clear_damage();

        // Move the layer
        layer.set_position((200.0, 100.0), None);
        apply_changes_and_update_layout(&engine);

        let layout = engine.layout_tree.read().unwrap();
        let node_id: indextree::NodeId = layer.id.0;
        let result = update_node_single(&engine, &layout, node_id, None, false);

        // Expect union of old (100,100,100x100) and new (200,100,100x100) bounds => (100,100,200x100)
        assert_eq!(
            result.damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 200.0, 100.0)
        );
    }

    #[test]
    fn update_node_single_opacity_transitions_damage() {
        let engine = Engine::create(1000.0, 100.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        // Give it visible content (not required by logic, but realistic)
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.add_layer(&layer);

        // Prime baseline (opacity = 1.0)
        engine.update(0.016);
        engine.clear_damage();

        // Fade out: 1.0 -> 0.0 damages previous bounds
        layer.set_opacity(0.0, None);
        apply_changes_and_update_layout(&engine);
        let layout = engine.layout_tree.read().unwrap();
        let node_id: indextree::NodeId = layer.id.0;
        let result = update_node_single(&engine, &layout, node_id, None, false);
        assert_eq!(
            result.damage,
            skia_safe::Rect::from_xywh(100.0, 0.0, 100.0, 100.0)
        );

        // Fade in: 0.0 -> 0.1 damages new bounds
        engine.clear_damage();
        drop(layout);
        layer.set_opacity(0.1, None);
        apply_changes_and_update_layout(&engine);
        let layout = engine.layout_tree.read().unwrap();
        let result = update_node_single(&engine, &layout, node_id, None, false);
        assert_eq!(
            result.damage,
            skia_safe::Rect::from_xywh(100.0, 0.0, 100.0, 100.0)
        );

        // Opacity change while visible: 0.1 -> 0.5 damages current bounds
        engine.clear_damage();
        drop(layout);
        layer.set_opacity(0.5, None);
        apply_changes_and_update_layout(&engine);
        let layout = engine.layout_tree.read().unwrap();
        let result = update_node_single(&engine, &layout, node_id, None, false);
        assert_eq!(
            result.damage,
            skia_safe::Rect::from_xywh(100.0, 0.0, 100.0, 100.0)
        );
    }

    #[test]
    fn update_node_single_content_damage_is_mapped() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&layer);

        // Prime initial state
        engine.update(0.016);
        engine.clear_damage();

        // Content draws a small rect in layer space (0,0,10x10)
        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0)
        };
        layer.set_draw_content(draw_func);

        apply_changes_and_update_layout(&engine);
        let layout = engine.layout_tree.read().unwrap();
        let node_id: indextree::NodeId = layer.id.0;
        let result = update_node_single(&engine, &layout, node_id, None, false);

        // Expect content damage mapped by the node transform => translated by (100,100)
        assert_eq!(
            result.damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 10.0, 10.0)
        );
    }

    #[test]
    fn update_node_single_no_changes_no_damage() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        // Give the layer some visible content to ensure non-empty bounds
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ffffffff"),
            },
            None,
        );
        engine.add_layer(&layer);

        // Prime initial state so previous/new states are identical and stable
        engine.update(0.016);
        engine.clear_damage();

        // No property changes are applied here
        let layout = engine.layout_tree.read().unwrap();
        let node_id: indextree::NodeId = layer.id.0;
        let result = update_node_single(&engine, &layout, node_id, None, false);

        // Expect no damage when nothing changed
        assert_eq!(
            result.damage,
            skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn child_bounds_follow_parent_transform() {
        let engine = Engine::create(1000.0, 1000.0);

        let parent = engine.new_layer();
        parent.set_position((10.0, 10.0), None);

        let child = engine.new_layer();
        child.set_position((5.0, 5.0), None);

        engine.add_layer(&parent);
        engine.append_layer(&child.id, Some(parent.id));

        // Prime initial layout/state
        engine.update(0.016);

        // Move the parent; child global bounds should shift by the same delta.
        parent.set_position((50.0, 0.0), None);
        engine.update(0.016);

        let child_bounds = child.render_bounds_transformed();

        assert_eq!(
            child_bounds,
            skia_safe::Rect::from_xywh(55.0, 5.0, 0.0, 0.0)
        );
    }
}
