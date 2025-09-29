# Engine Update Pipeline

This guide explains how the Layers engine advances a frame from scheduled model changes to final draw calls. It focuses on the responsibilities of each stage inside `Engine::update` and dedicates a section to the damage tracking rules that determine which pixels must be redrawn.

## High-level flow

Every call to `Engine::update(dt)` executes the same ordered pipeline:

| Stage                 | Entry point                                      | Purpose                                                                 |
| --------------------- | ------------------------------------------------ | ----------------------------------------------------------------------- |
| 1. Advance animations | `update_animations` (`src/engine/stages/mod.rs`) | Step active timelines, queueing property changes for affected nodes     |
| 2. Apply transactions | `execute_transactions`                           | Update `ModelLayer` properties and adjust render/layout flags           |
| 3. Solve layout       | `update_layout_tree`                             | Run Taffy to recompute node sizes and positions                         |
| 4. Refresh nodes      | `Engine::update_nodes` (`src/engine/mod.rs`)     | Visit each scene node, update render layers, and calculate damage       |
| 5. Render             | `render_node_tree` (`src/drawing/scene.rs`)      | Replay cached pictures or draw commands, clipping to accumulated damage |

The engine exposes the merged damage rectangle through `Engine::damage()`. Clients should consume it after rendering and call `Engine::clear_damage()` to prepare for the next frame.

## Stage details

### 1. Animations

Animations are stored as `AnimationState` entries. `update_animations` increments their time, evaluates easing curves, and schedules node changes by pushing `AnimatedNodeChange` items into the transaction queue. Commands expose hooks for `on_start`, `on_update`, and `on_finish` callbacks.

### 2. Transactions

`execute_transactions` drains scheduled changes. Each command mutates its target `ModelLayer` and returns `RenderableFlags` that tell the engine whether the node needs layout and/or paint. Flags are applied to the owning `SceneNode`, making the subsequent stages aware of what needs recomputation.

### 3. Layout

`update_layout_tree` translates flagged nodes into Taffy style updates and runs flex/grid layout. Updated nodes receive the `NEEDS_LAYOUT` flag, and their computed geometry is written back to the layout arena. Layout results feed directly into render layer updates during the node refresh pass.

### 4. Node refresh

`Engine::update_nodes` walks the scene graph depth-first from parents to children (grouped by depth, parents processed before descendants). For each node it calls `update_node_single`, which:

1. Reads the previous render-layer state (bounds, opacity, visibility, repaint flag).
2. Aggregates child bounds in local space so parents can detect subtree geometry changes.
3. Updates the node’s `RenderLayer` with fresh model and layout information.
4. Repaints cached content when flagged by `NEEDS_PAINT`, parent changes, geometry adjustments, opacity transitions, or visibility toggles.
5. Clears `NEEDS_PAINT` and `NEEDS_LAYOUT` flags.
6. Produces a global-space damage rectangle that covers content repaint, geometry unions, opacity changes, and subtree adjustments.
7. Increments the node frame counter when any damage occurred (useful for debug overlays).

Returned rectangles are joined into the per-frame damage accumulator. The engine may also call `propagate_damage_to_ancestors` so higher-level render targets know to redraw when descendants change.

#### Why parents run before children

- Children rely on the latest parent transform and premultiplied opacity when repainting. Running parents first keeps those values fresh before `update_node_single` executes on the child.
- Parents still detect child-driven layout changes because they compare their own new layout data (just written by Taffy) against the previous render-layer snapshot. Aggregated child bounds are read from the stored `SceneNode`, so "before" and "after" comparisons remain valid even if the child has not updated yet.
- When a child runs later in the pass it reports damage in global coordinates. `update_nodes` merges that rectangle into the frame total, so child content changes are preserved even though the parent processed earlier.

### 5. Rendering

Renderers such as `SkiaFboRenderer` and `SkiaImageRenderer` consume the merged damage. They set up clip regions, apply root transforms, and call `render_node_tree`, which:

- Optionally renders subtrees into off-screen surfaces when image caching is enabled.
- Replays cached Skia `Picture`s or triggers `draw_layer` to paint backgrounds, shadows, borders, and dynamic content.
- Honors clip flags, blend modes, and filters from the `RenderLayer`.

The client typically swaps buffers or composites the result, then clears the engine damage before the next frame.

## Damage tracking

Damage tracking quantifies the minimal screen area that must be redrawn. It relies on comparisons between the previous and current `RenderLayer` state recorded inside `SceneNode`s.

### Key inputs

- **Geometry**: `RenderLayer::global_transformed_bounds` and `global_transformed_bounds_with_children` capture the node’s own bounds and the union with descendants. Differences trigger unions of old and new rectangles.
- **Visibility**: `RenderLayer::has_visible_drawables()` reports whether the node can draw anything (background, border, shadow, content, or filters) after pre-multiplying opacity. Invisible nodes suppress geometry damage unless debug overlays are active.
- **Opacity**: Changes to premultiplied opacity determine whether a node faded in, faded out, or adjusted alpha while visible.
- **Content repaint**: `do_repaint` records Skia draw commands into `SceneNodeRenderable::draw_cache` and returns a layer-space damage rectangle. This rect is mapped into global coordinates via the node’s `transform_33` matrix.

### Decision rules inside `update_node_single`

- **Early exit**: If parents were stable, repaint flags are clear, geometry/opacity/visibility are unchanged, and the render layer did not mutate, the function returns an empty rectangle.
- **Content damage**: Non-empty rectangles returned by `do_repaint` are transformed to global coordinates and seeded into the total damage.
- **Geometry unions**: When size or position changes, the union of old and new bounds is joined to damage. Child-aggregated bounds behave the same way so container nodes repaint around resized descendants.
- **Visibility flips**: When a node becomes hidden, previous bounds are damaged so pixels can be cleared. When it becomes visible, new bounds are damaged so pixels can be drawn. Partial opacity transitions damage the current bounds while the node remains visible.
- **Debug overlays**: Nodes with `_debug_info` set force geometry damage even if they have no drawables, which helps tooling visualize updates.
- **Parent-first traversal**: Because parents execute before children, child damage is accumulated slightly later in the loop. This is safe because each child returns a global-space rectangle that goes straight into `Engine.damage`, while parents already captured their own geometry/layout deltas using the fresh layout data.

### Propagation and accumulation

The per-node rectangles returned by `update_node_single` are joined into `Engine.damage`. When descendant updates alter ancestor bounds, `propagate_damage_to_ancestors` inflates ancestor damage to the descendant’s transformed bounds. This guarantees that render targets encompassing multiple nodes receive all necessary redraw regions.

## Caching interplay

Damage tracking works alongside two caching layers:

- **Picture cache** (`SceneNodeRenderable::draw_cache`): Stores Skia `Picture`s produced by `do_repaint`. As long as the cache is valid, `update_node_single` skips repainting and produces empty content damage.
- **Image cache** (`SceneNode::is_image_cached` + `render_node_tree`): Allows entire subtrees to render once into an off-screen surface. Damage logic still relies on logical bounds to decide when the cached surface must be refreshed.

## Testing & debugging

- **Integration suite**: `tests/damage.rs` covers content callbacks, nested transforms, geometry changes, opacity transitions, and visual effects (shadow, border, blur). Run the suite with:

  ```bash
  cargo test --test damage
  ```

- **Unit tests**: `src/engine/stages/update_node.rs` contains unit tests that verify geometry unions, opacity transitions, and global mapping for `update_node_single`.
- **Inspection tips**: After each `update`, call `Engine::damage()` to verify the rectangles match expectations. Remember to call `Engine::clear_damage()` after consuming the result.

## Best practices

- Batch related property changes to minimize layout churn and reduce large damage unions.
- Enable picture or image caching for nodes with expensive paint routines so repaint damage only occurs when content truly changes.
- Add unit or integration tests when modifying damage logic to guard against regressions.
- Combine benchmarking (`cargo bench --bench my_benchmark`) with damage-focused tests when profiling performance-critical scenes.
