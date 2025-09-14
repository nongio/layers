use indextree::NodeId;
use taffy::TaffyTree;

use crate::{
    engine::{node::do_repaint, storage::TreeStorageData, *},
    layers::layer::render_layer::RenderLayer,
};

// This function updates a single node's RenderLayer, repaints if needed, and calculates damage.
// It compares the node's state before and after updates to determine what screen areas need
// to be redrawn, taking into account position, size, opacity, and content changes.
// Returns the total damage rectangle that encompasses all changed areas.#[allow(unused_assignments, unused_mut)]
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

    // First, read the current state immutably from both arenas
    let (prev_transformed_bounds, prev_opacity, prev_needs_paint) =
        engine.scene.with_arena(|node_arena| {
            let node = node_arena.get(node_id);

            if node.is_none() {
                return (skia::Rect::default(), 0.0, false);
            }

            let scene_node = node.unwrap().get();
            // Store previous state for comparison
            let prev_transformed_bounds = scene_node
                .render_layer
                .global_transformed_bounds_with_children;
            let prev_opacity = scene_node.render_layer.premultiplied_opacity;
            let needs_paint = scene_node.needs_repaint();

            (prev_transformed_bounds, prev_opacity, needs_paint)
        });
    // calculate children bounds
    let local_children_bounds = engine.scene.with_arena(|node_arena| {
        let mut local_children_bounds = skia::Rect::default();
        node_id.children(node_arena).for_each(|child_id| {
            if let Some(child_node) = node_arena.get(child_id) {
                let child_scene_node = child_node.get();
                let child_rl = &child_scene_node.render_layer;
                // Accumulate child's local union
                local_children_bounds.join(child_rl.local_transformed_bounds_with_children);
            }
        });
        local_children_bounds
    });
    // Get cumulative transform and opacity from parent
    let cumulative_transform = parent.map(|p| &p.transform);
    let context_opacity = parent.map(|p| p.premultiplied_opacity).unwrap_or(1.0);

    // Update the render layer (only node arena is mutable here)
    let (changed_render_layer, _is_image_cached, is_debug) =
        engine.scene.with_arena_mut(|node_arena| {
            let node = node_arena.get_mut(node_id);
            if let Some(node) = node {
                let scene_node = node.get_mut();
                // LAYOUT STEP: merge with children bounds
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
                let is_cached = scene_node.is_image_cached();
                (changed, is_cached, scene_node._debug_info.is_some())
            } else {
                (false, false, false)
            }
        });

    // Read updated state before deciding to repaint
    let (new_transformed_bounds, new_opacity, _current_needs_paint) =
        engine.scene.with_arena(|arena| {
            let node = arena.get(node_id).unwrap();
            let scene_node = node.get();
            (
                scene_node
                    .render_layer
                    .global_transformed_bounds_with_children,
                scene_node.render_layer.premultiplied_opacity,
                scene_node.needs_repaint(),
            )
        });

    // Check what changed using previous vs current state (without forcing repaint)
    let layout_changed = prev_transformed_bounds.width() != new_transformed_bounds.width()
        || prev_transformed_bounds.height() != new_transformed_bounds.height();
    let position_changed = prev_transformed_bounds.x() != new_transformed_bounds.x()
        || prev_transformed_bounds.y() != new_transformed_bounds.y();
    let opacity_changed = prev_opacity != new_opacity;

    // Early exit: nothing changed and no repaint requested by flags/parents
    if !parent_changed
        && !prev_needs_paint
        && !layout_changed
        && !position_changed
        && !opacity_changed
        && !changed_render_layer
    {
        return skia::Rect::default();
    }

    // Do the actual repaint if needed (both arenas need to be accessed)
    let mut some_renderable = None;
    let content_damage = engine.scene.with_arena(|node_arena| {
        let opt_renderable = engine.scene.renderables.get(&node_id.into());
        let node = node_arena.get(node_id);
        if let (Some(node), Some(scene_node_renderable)) = (node, opt_renderable) {
            let scene_node = node.get();
            let mut repaint_damage = skia::Rect::default();
            if scene_node.needs_repaint()
                || parent_changed
                || layout_changed
                || position_changed
                || opacity_changed
            {
                let renderable = do_repaint(&scene_node_renderable, scene_node);
                repaint_damage = renderable.repaint_damage;
                some_renderable = Some(renderable);
            }
            repaint_damage
        } else {
            skia_safe::Rect::default()
        }
    });

    if let Some(new_renderable) = some_renderable {
        engine
            .scene
            .renderables
            .insert_with_id(new_renderable, node_id.into());
    }
    engine.scene.with_arena_mut(|node_arena| {
        let node = node_arena.get_mut(node_id);
        if let Some(node) = node {
            let scene_node = node.get_mut();
            scene_node.set_needs_repaint(false);
            scene_node.set_needs_layout(false);
        }
    });

    // Map content damage into global space using the current transform
    let (mapped_content_damage, _) = engine.scene.with_arena(|arena| {
        let node = arena.get(node_id).unwrap();
        let render_layer = &node.get().render_layer;
        render_layer.transform_33.map_rect(content_damage)
    });

    // Calculate total damage for this node
    let mut total_damage = mapped_content_damage;

    if position_changed || is_debug {
        // Include both old and new bounds when position changes
        total_damage.join(prev_transformed_bounds);
        total_damage.join(new_transformed_bounds);
    }

    if opacity_changed {
        // When opacity changes, we need to damage the areas that become visible or invisible
        if prev_opacity <= 0.0 && new_opacity > 0.0 {
            // Layer becomes visible - damage the new bounds
            total_damage.join(new_transformed_bounds);
        } else if prev_opacity > 0.0 && new_opacity <= 0.0 {
            // Layer becomes invisible - damage the previous bounds
            total_damage.join(prev_transformed_bounds);
        } else if prev_opacity > 0.0 && new_opacity > 0.0 {
            // Layer remains visible but opacity changes - damage current bounds
            total_damage.join(new_transformed_bounds);
        }
    }

    // Update frame if anything changed
    let content_repainted = !content_damage.is_empty();
    let damaged = layout_changed
        || content_repainted
        || position_changed
        || opacity_changed
        || parent_changed;

    if damaged {
        engine.scene.with_arena_mut(|arena| {
            if let Some(scene_node) = arena.get_mut(node_id) {
                let node = scene_node.get_mut();
                node.increase_frame();
            }
        });
    }

    // Log render_layer.key at cursor position
    // if !total_damage.is_empty() {
    //     engine.scene.with_arena(|arena| {
    //         if let Some(node) = arena.get(node_id) {
    //             let scene_node = node.get();

    //             println!(
    //                 "Damage: {:?} | damaged area: {},{},{},{}",
    //                 scene_node.render_layer.key,
    //                 total_damage.x(),
    //                 total_damage.y(),
    //                 total_damage.width(),
    //                 total_damage.height()
    //             );
    //         }
    //     });
    // }

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
    fn update_node_single_position_change_damages_union() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
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
