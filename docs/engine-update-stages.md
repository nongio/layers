# Engine Update Pipeline

This guide explains how the Layers engine advances a frame from scheduled model changes to final draw calls. It focuses on the responsibilities of each stage inside `Engine::update` and dedicates a section to the damage tracking rules that determine which pixels must be redrawn.

## High-level flow

Every call to `Engine::update(dt)` executes the same ordered pipeline:

| Stage                  | Entry point                                        | Purpose                                                                  |
| ---------------------- | -------------------------------------------------- | ------------------------------------------------------------------------ |
| 1. Advance animations  | `update_animations` (`src/engine/stages/mod.rs`)   | Step active timelines, queueing property changes for affected nodes      |
| 2. Apply transactions  | `execute_transactions`                             | Update `ModelLayer` properties and adjust render/layout flags            |
| 3. Solve layout        | `update_layout_tree`                               | Run Taffy to recompute node sizes and positions                          |
| 4. Refresh nodes       | `Engine::update_nodes` (`src/engine/mod.rs`)       | Visit each scene node, update render layers, and calculate damage        |
| 5. Render              | `render_node_tree` (`src/drawing/scene.rs`)        | Replay cached pictures or draw commands, clipping to accumulated damage  |
| 6. Cleanup deleted     | `cleanup_nodes` (`src/engine/stages/mod.rs`)       | Drop `SceneNode`s marked for removal and tidy layout bookkeeping safely  |

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

### 6. Cleanup

`cleanup_nodes` runs after rendering to dispose of any scene subtrees whose root layer called `Layer::remove` (or was implicitly deleted by its parent). Each candidate node is still present in the scene arena but flagged with `is_deleted()`. For every such node the cleanup stage:

1. Records the node’s transformed bounds to extend frame damage (ensuring stale visuals are repainted).
2. Calls `Engine::scene_remove_layer` to unlink the subtree from both the scene graph and the Taffy layout tree.

`scene_remove_layer` guards its Taffy bookkeeping so that it only marks a parent layout node dirty when that parent still exists and has not been scheduled for deletion. This prevents panics from attempting to dirty or traverse already-removed layout nodes while the tree is being dismantled.

## Damage tracking

The full set of damage rules, caching interactions, and test guidance now lives in [damage-tracking.md](damage-tracking.md). Refer to that companion guide for a deep dive into how `update_node_single` compares render-layer snapshots, reports per-node rectangles, and how the engine aggregates them into `Engine::damage()`.
