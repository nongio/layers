# Pointer Input

This guide documents how the Layers engine performs pointer hit-testing, tracks hover state, and dispatches pointer callbacks registered through the public `Layer` API.

## Core data structures

- `PointerHandlerFunction` (`src/layers/layer/model.rs`) wraps an `Arc<dyn Fn(&Layer, f32, f32)>`. Handlers receive the owning `Layer` plus the pointer coordinates expressed in scene space.
- `PointerEventType` (`src/engine/mod.rs`) enumerates the five supported phases: `Move`, `In`, `Out`, `Down`, and `Up`.
- `PointerCallback` stores the handlers for a node, partitioned by event type and keyed by the handler id that is returned to the caller. The engine keeps one `PointerCallback` per scene node in `Engine::pointer_handlers` (`FlatStorage<PointerCallback>`).
- `UNIQ_POINTER_HANDLER_ID` is an atomic counter that guarantees stable, unique ids so handlers can be removed or replaced later on.

`Layer::add_on_pointer_move/in/out/press/release` (`src/layers/layer/mod.rs`) forward to `Engine::add_pointer_handler`, which places the handler into the node’s `PointerCallback`. Removing happens either through the corresponding `remove_on_*` helpers or by calling `Layer::remove_all_pointer_handlers`.

## Pointer participation

Each `ModelLayer` exposes an atomic `pointer_events` flag (`Layer::set_pointer_events`). Hit-testing only considers nodes whose flag is `true` **and** whose render state is not hidden. Turning the flag off is the supported way to ensure a layer does not receive pointer callbacks without touching visibility.

## Hover tracking and hit-testing

`Engine::pointer_move` is the main entry point used by windowing backends:

1. The method updates the shared `pointer_position` so subsequent button events can reuse the last coordinates.
2. Callers may optionally pass a subtree root; otherwise the engine falls back to the current scene root.
3. The method enumerates every descendant of that root using `TreeStorageId::descendants`. The iterator is reversed so the check starts at the visual front-most node (deepest descendant encountered last in tree order).
4. For each node the engine skips hidden nodes and those with `pointer_events == false`. Remaining nodes call `SceneNode::contains_point`, which uses the node’s `RenderLayer.global_transformed_bounds` to perform global-space hit tests that include transforms.
5. The first node that reports a hit becomes the new hover target. The engine compares it against `current_hover_node` to decide which synthetic events (`In` and `Out`) must fire. When the pointer leaves all hit-testable nodes, the old hover target is cleared and only `Out` is emitted.
6. The method calls `bubble_up_event` to dispatch `Move`, `In`, and `Out` events for the relevant nodes and returns `true` if any node was hit.

Because hit-testing walks the scene bottom-up, leaf nodes that overlap their ancestors correctly win hover focus.

## Event bubbling order

`bubble_up_event` gathers the target node and all of its ancestors (skipping nodes that have been removed from the arena). Events are delivered in reverse order, which means ancestors run before the target node. This mirrors DOM-style event capturing: high-level containers can intercept or respond to pointer state changes before the leaf node’s own handlers execute, and the target still receives the event during the same pass.

Handlers are invoked synchronously. The engine clones the shared pointer coordinates and passes the owning `Layer` handle to each registered callback. If a layer registers multiple handlers for the same event type, they are executed in the iteration order of the underlying `HashMap::values` for that event bucket.

## Button events

`Engine::pointer_button_down` and `pointer_button_up` consult `current_hover_node`, emitting `Down`/`Up` events only for the node that currently owns the hover state. The same bubbling logic applies, so ancestors receive press and release notifications even when the pointer is over a deeply nested child.

Backends typically call `pointer_move` first, then trigger `pointer_button_down`/`pointer_button_up` based on platform input events to keep hover and pressed state synchronized.

## Pointer state helpers

- `Engine::current_hover()` exposes the node reference that owns the hover lock, which is useful for debugging overlays.
- `Engine::get_pointer_position()` returns the last coordinates seen by the engine.

The automated tests under `tests/pointer_handlers.rs` exercise the main behaviors: hit-testing through parent/child hierarchies, handler removal, and the `In`/`Out` transitions.
