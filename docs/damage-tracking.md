# Damage Tracking in Layers

Damage tracking is the process the engine uses to figure out the minimum portion of the scene that must be redrawn after state changes. This document walks through the end-to-end flow, the core data structures, and the rules that govern how damage rectangles are accumulated, propagated, and consumed by renderers.

## Quick reference

| Concept             | Location                                                   | Purpose                                                                             |
| ------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Frame orchestration | `Engine::update` in `src/engine/mod.rs`                    | Advances time, runs transactions, and drives the frame pipeline                     |
| Node update pass    | `Engine::update_nodes` in `src/engine/mod.rs`              | Visits scene nodes depth-first to refresh render layers and collect damage          |
| Per-node damage     | `update_node_single` in `src/engine/stages/update_node.rs` | Compares previous and current state and returns a damage rect in global coordinates |
| Repaint             | `do_repaint` in `src/engine/node/mod.rs`                   | Records draw commands into a Skia `Picture`, returning layer-space repaint damage   |
| Propagation         | `Engine::propagate_damage_to_ancestors`                    | Grows ancestor damage when descendants change                                       |
| Final draw          | `render_node_tree` in `src/drawing/scene.rs`               | Clips to the engine damage and issues draw calls                                    |

## Frame pipeline overview

Each call to `Engine::update(dt)` executes a deterministic sequence of stages:

1. **Update animations** – `update_animations` steps active timelines, scheduling property transitions as needed.
2. **Execute transactions** – `execute_transactions` mutates layer models and sets rendering/layout flags on the affected nodes.
3. **Layout** – `update_layout_tree` runs Taffy to resolve sizes and positions; nodes that changed layout receive the `NEEDS_LAYOUT` flag.
4. **Node updates** – `Engine::update_nodes` walks the scene graph from parents to children. For each node it invokes `update_node_single`, accumulates returned damage, and optionally propagates damage up the tree.
5. **Renderer draw** – The active renderer takes the merged damage (`Engine.damage`) and re-renders only the impacted area.

The damage rectangle is reset by `Engine::clear_damage()` after the client processes it.

## Data captured per node

Each `SceneNode` stores both declarative state (`ModelLayer`) and the derived `RenderLayer`. The latter caches everything the renderer needs for fast draws: local/global transforms, pre-multiplied opacity, rounded bounds, clip flags, cached pictures, and content callbacks. During `update_node_single` the engine reads the previous `RenderLayer` state and compares it with the freshly-updated one to detect what actually changed.

Key comparisons include:

- **Geometry** – Differences in width/height (layout) or x/y (position) at both the node and children-unioned levels, using `RenderLayer::global_transformed_bounds` and `global_transformed_bounds_with_children`.
- **Visibility** – `RenderLayer::has_visible_drawables()` encapsulates whether the node can draw anything (background, border, shadow, content, or filters) after opacity is applied.
- **Opacity** – Changes to premultiplied opacity determine whether the node faded in/out or adjusted alpha while visible.
- **Content repaint** – `do_repaint` records content callbacks into Skia pictures and returns layer-space rectangles that represent fresh pixels. These rectangles are mapped into global coordinates via the node transform.

## How `update_node_single` computes damage

1. **Snapshot previous state** – Pulls the last `RenderLayer` bounds, children bounds, opacity, and repaint flag.
2. **Collect child bounds** – Aggregates the child nodes’ `local_transformed_bounds_with_children` so parents can detect subtree size changes.
3. **Refresh the render layer** – Updates the node’s `RenderLayer` with new layout and model data. If anything changes (or debug overlays are active), the node is marked as needing repaint.
4. **Repaint if needed** – When `needs_repaint` or any parent/geometry/opacity flag dictates, `do_repaint` is called. The returned damage stays in layer space until it is transformed to global coordinates.
5. **Reset flags** – `NEEDS_PAINT` and `NEEDS_LAYOUT` are cleared after the update is complete.
6. **Accumulate geometry damage** – If geometry changed and the node (or debug overlay) is visible, the union of old and new bounds is added to the damage rect. Children bounds changes are treated similarly.
7. **Opacity and visibility** – Fades in/out add the appropriate previous/new bounds; toggling visibility marks the region where pixels appear or disappear.
8. **Mark animated frame** – Nodes that produced damage bump their frame counter (`SceneNode::increase_frame`), aiding debug tools.
9. **Return totals** – The resulting `skia::Rect` is in global coordinates and includes both mapped content damage and geometry/opacity unions.

Because parents are processed before children in `update_nodes`, the parent transform and opacity used during a child update are always up to date.

## Damage propagation

`Engine::update_nodes` collects per-node damage, then merges it into the engine-wide rect. When subtree changes occur, `Engine::propagate_damage_to_ancestors` extends ancestor damage to include descendant bounds so higher-level render targets know to redraw. The combination of per-node accumulation plus ancestor propagation ensures damage always encloses the pixels that can actually change.

## Interaction with caching

Layers support two complementary caches:

- **Picture cache (`SceneNodeRenderable::draw_cache`)** – Stores the result of `do_repaint` to avoid re-recording Skia commands when nothing changed. When a cache is reused, no new content damage is produced.
- **Image cache (`render_node_tree`)** – For nodes flagged with `SceneNode::is_image_cached`, the engine can render the entire subtree into an off-screen surface once, then reuse the resulting snapshot. Damage is still computed off the logical bounds so the renderer knows when to refresh the cached surface.

Damage tracking works hand-in-hand with caching: repaint occurs only when model/layout flags say pixels might differ, and otherwise cached pictures are drawn without inflating the damage rectangle.

## Testing the damage pipeline

The repository includes a dedicated integration suite in `tests/damage.rs` that exercises common scenarios:

- `damage_content` and `damage_content_nested` confirm that content callbacks map to global coordinates correctly, even through parent transforms.
- `damage_rect`, `damage_move_layer`, and `damage_parent_offset` validate geometry-driven damage.
- `damage_opacity` and `damage_render_layer_transparent` cover opacity transitions and invisible layers.
- `damage_render_layer_shadow`, `damage_render_layer_border`, and `damage_render_layer_backblur` ensure visual effects expand the damage as expected.

Run the suite with:

```bash
cargo test --test damage
```

Unit tests in `src/engine/stages/update_node.rs` provide targeted checks for `update_node_single` (geometry union, opacity transitions, and content mapping).

## Debugging tips

- Set the `LAYERS_DEBUG_DAMAGE=1` environment variable to enable verbose logging overlays (if the debug flag is wired through your build), allowing you to visualize damage rectangles.
- Inspect `Engine::damage()` after each `update` call to ensure the rect matches your mental model; call `Engine::clear_damage()` once you have consumed it.
- Temporarily enable `_debug_info` on a node (via debugger tools) to force it to mark damage even without visible drawables—helpful when tracking invisible elements.

## Best practices

- **Minimize layout churn** – Frequent changes deep in the tree can bubble large damage up through ancestors. Batch property updates and prefer child-local transforms when possible.
- **Leverage caching** – Enable picture or image caching for nodes with expensive paint routines; the damage system already skips repainting when the cache remains valid.
- **Validate with tests** – Add focused unit or integration tests whenever you modify damage logic to avoid regressions.
- **Profile strategically** – Combine `cargo bench --bench my_benchmark` with the damage tests to detect performance regressions in damage-heavy scenes.

Understanding these rules will help you diagnose why a frame redraw happens (or does not) and will guide you when altering the engine’s rendering pipeline.
