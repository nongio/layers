use indextree::{Arena, NodeId};
use taffy::TaffyTree;

use crate::{
    engine::{node::do_repaint, storage::TreeStorageData, *},
    layers::layer::render_layer::RenderLayer,
};

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
            id.children(arena)
                .for_each(|child_id| stack.push(child_id));
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
) -> skia::Rect {
    let layer = engine.get_layer(&NodeRef(node_id)).unwrap();
    let node_layout = layout_tree.layout(layer.layout_id).unwrap();

    // First, read the previous state for comparisons
    let (prev_transformed_bounds, prev_global_bounds, prev_opacity, prev_visible, prev_needs_paint) =
        engine.scene.with_arena(|arena| {
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
                )
            } else {
                (
                    skia_safe::Rect::default(),
                    skia_safe::Rect::default(),
                    0.0,
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

    // Update the render layer using the latest model/layout state
    let (changed_render_layer, is_debug) = engine.scene.with_arena_mut(|arena| {
        arena
            .get_mut(node_id)
            .map(|node| {
                let scene_node = node.get_mut();
                let changed = scene_node.update_render_layer_if_needed(
                    node_layout,
                    layer.model.clone(),
                    cumulative_transform,
                    context_opacity,
                    local_children_bounds,
                ) || scene_node._debug_info.is_some();

                if changed {
                    scene_node.set_needs_repaint(true);
                }

                (changed, scene_node._debug_info.is_some())
            })
            .unwrap_or((false, false))
    });

    // Capture the new state after the update
    let (new_transformed_bounds, new_global_bounds, new_opacity, new_visible, current_needs_paint) =
        engine.scene.with_arena(|arena| {
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
        return skia_safe::Rect::default();
    }

    // Trigger repaint if required and capture content damage
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

    if geometry_changed_self && (has_visible_drawables || is_debug) {
        total_damage.join(prev_global_bounds);
        total_damage.join(new_global_bounds);
    }

    if geometry_changed_children && (has_visible_drawables || is_debug) {
        total_damage.join(prev_transformed_bounds);
        total_damage.join(new_transformed_bounds);
    }

    if prev_effective_visible && !new_effective_visible {
        total_damage.join(prev_transformed_bounds);
    }

    if opacity_changed && has_visible_drawables {
        if prev_opacity <= 0.0 && new_opacity > 0.0 {
            total_damage.join(new_transformed_bounds);
        } else if prev_opacity > 0.0 && new_opacity <= 0.0 {
            total_damage.join(prev_transformed_bounds);
        } else {
            total_damage.join(new_transformed_bounds);
        }
    }

    let content_repainted = !content_damage.is_empty();
    let damaged = content_repainted
        || (geometry_changed_self && (has_visible_drawables || is_debug))
        || (geometry_changed_children && (has_visible_drawables || is_debug))
        || opacity_changed
        || parent_changed
        || visibility_changed
        || changed_render_layer;

    if damaged {
        engine.scene.with_arena_mut(|arena| {
            if let Some(node) = arena.get_mut(node_id) {
                node.get_mut().increase_frame();
            }
        });
    }

    total_damage
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
        let damage = update_node_single(&engine, &layout, node_id, None, false);

        // Expect union of old (100,100,100x100) and new (200,100,100x100) bounds => (100,100,200x100)
        assert_eq!(
            damage,
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
        let damage = update_node_single(&engine, &layout, node_id, None, false);
        assert_eq!(damage, skia_safe::Rect::from_xywh(100.0, 0.0, 100.0, 100.0));

        // Fade in: 0.0 -> 0.1 damages new bounds
        engine.clear_damage();
        drop(layout);
        layer.set_opacity(0.1, None);
        apply_changes_and_update_layout(&engine);
        let layout = engine.layout_tree.read().unwrap();
        let damage = update_node_single(&engine, &layout, node_id, None, false);
        assert_eq!(damage, skia_safe::Rect::from_xywh(100.0, 0.0, 100.0, 100.0));

        // Opacity change while visible: 0.1 -> 0.5 damages current bounds
        engine.clear_damage();
        drop(layout);
        layer.set_opacity(0.5, None);
        apply_changes_and_update_layout(&engine);
        let layout = engine.layout_tree.read().unwrap();
        let damage = update_node_single(&engine, &layout, node_id, None, false);
        assert_eq!(damage, skia_safe::Rect::from_xywh(100.0, 0.0, 100.0, 100.0));
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
        let damage = update_node_single(&engine, &layout, node_id, None, false);

        // Expect content damage mapped by the node transform => translated by (100,100)
        assert_eq!(damage, skia_safe::Rect::from_xywh(100.0, 100.0, 10.0, 10.0));
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
        let damage = update_node_single(&engine, &layout, node_id, None, false);

        // Expect no damage when nothing changed
        assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
    }
}
