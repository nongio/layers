# Damage Tracking

This document explains how Layers computes frame damage so renderers can use this information to only redraw the pixels that changed. The engine uses previous render-layer state, compares it with the freshly updated data inside `Engine::update_nodes`, and merges any differences into `Engine.damage`.

## Key inputs

- **Geometry**: `RenderLayer::global_transformed_bounds` and `global_transformed_bounds_with_children` capture a node’s own bounds and the union with descendants. Differences between the previous and current values are unioned into damage.
- **Visibility**: `RenderLayer::has_visible_drawables()` reports whether the node can draw anything (background, border, shadow, content, or filters) after premultiplying opacity. Invisible nodes suppress geometry damage unless debug overlays are active.
- **Opacity**: Changes to premultiplied opacity determine whether a node faded in, faded out, or adjusted alpha while visible.
- **Content repaint**: `do_repaint` records Skia draw commands into `SceneNodeRenderable::draw_cache` and returns a layer-space damage rectangle. This rectangle is mapped into global coordinates via the node’s `transform_33` matrix.

## Decision rules inside `update_node_single`

- **Early exit**: If parents were stable, repaint flags are clear, geometry/opacity/visibility are unchanged, and the render layer did not mutate, the function returns an empty rectangle.
- **Content damage**: Non-empty rectangles returned by `do_repaint` are transformed to global coordinates and seeded into the total damage.
- **Geometry unions**: When size or position changes, the union of old and new bounds is joined to damage. Child-aggregated bounds behave the same way so container nodes repaint around resized descendants.
- **Visibility flips**: When a node becomes hidden, previous bounds are damaged so pixels can be cleared. When it becomes visible, new bounds are damaged so pixels can be drawn. Partial opacity transitions damage the current bounds while the node remains visible.
- **Debug overlays**: Nodes with `_debug_info` set force geometry damage even if they have no drawables, which helps tooling visualize updates.
- **Parent-first traversal**: Because parents execute before children, child damage is accumulated slightly later in the loop. This is safe because each child returns a global-space rectangle that goes straight into `Engine.damage`, while parents already captured their own geometry/layout deltas using the refreshed layout data.

## Propagation and accumulation

The per-node rectangles returned by `update_node_single` are merged into `Engine.damage`. When descendant updates alter ancestor bounds, `propagate_damage_to_ancestors` inflates ancestor damage to the descendant’s transformed bounds. This guarantees that render targets encompassing multiple nodes receive all necessary redraw regions.

## Caching interplay

Damage tracking works alongside two caching layers:

- **Picture cache** (`SceneNodeRenderable::draw_cache`): Stores Skia `Picture`s produced by `do_repaint`. As long as the cache is valid, `update_node_single` skips repainting and produces empty content damage.
- **Image cache** (`SceneNode::is_image_cached` + `render_node_tree`): Allows entire subtrees to render once into an off-screen surface. Damage logic still relies on logical bounds to decide when the cached surface must be refreshed.

## Testing & debugging

- **Integration suite**: `tests/damage.rs` covers content callbacks, nested transforms, geometry changes, opacity transitions, and visual effects (shadow, border, blur). Run the suite with `cargo test --test damage`.
- **Unit tests**: `src/engine/stages/update_node.rs` contains unit tests that verify geometry unions, opacity transitions, and global mapping for `update_node_single`.
- **Inspection tips**: After each `update`, call `Engine::damage()` to verify the rectangles match expectations. Remember to call `Engine::clear_damage()` after consuming the result.

## Best practices

- Batch related property changes to minimize layout churn and reduce large damage unions.
- Enable picture or image caching for nodes with expensive paint routines so repaint damage only occurs when content truly changes.
- Add unit or integration tests when modifying damage logic to guard against regressions.
- Combine benchmarking (`cargo bench --bench my_benchmark`) with damage-focused tests when profiling performance-critical scenes.
