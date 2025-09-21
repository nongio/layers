# Development Workflow for layers

This repository is a Rust workspace. Continuous integration uses Rust 1.83.0 on Ubuntu.
Follow these steps before submitting a pull request.

## Formatting and Linting
- Format the code with rustfmt and ensure the check passes:
  ```bash
  cargo fmt --all -- --check
  ```
- Lint using Clippy. All warnings must be fixed:
  ```bash
  cargo clippy --features "default" -- -D warnings
  ```

## Building and Docs
- Build the workspace to ensure it compiles:
  ```bash
  cargo check --features "default"
  ```
- Build the API documentation the same way CI does:
  ```bash
  RUSTDOCFLAGS=--cfg=docsrs cargo doc --no-deps --features "default" -p lay-rs
  ```
- Document every new method with a Rust doc comment explaining what it does.

## Criterion Benchmarks
- Benchmarks can be executed with Criterion:
  ```bash
  cargo bench --bench my_benchmark
  ```
  CI compares benchmark results against the base branch.

## System Dependencies
CI installs several development packages for Skia and Wayland support:
`libdrm-dev libudev-dev libgbm-dev libxkbcommon-dev libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev libsystemd-dev libseat-dev`.
Ensure these packages are available when running the above commands locally.

## Code Map: Rendering, Drawing, Caching, Pointer Input
- Rendering stages (pipeline)
  - Entry point: `src/engine/mod.rs::Engine::update` orchestrates a frame.
  - Animations: `src/engine/stages/mod.rs::update_animations` updates timelines.
  - Transactions: `src/engine/stages/mod.rs::execute_transactions` applies changes to nodes; sets render/layout flags.
  - Layout: `src/engine/stages/mod.rs::update_layout_tree` runs Taffy; `Engine::update_nodes` then updates render layers and repaints as needed.
  - Damage: `Engine::update_nodes` accumulates damage; `Engine::propagate_damage_to_ancestors` updates ancestor bounds; final damage stored in `Engine.damage`.

- Scene drawing (composition)
  - High-level: `src/drawing/scene.rs` provides `draw_scene` and `render_node_tree` to traverse and paint the scene.
  - Renderer implementations: `src/renderer/skia_fbo.rs` and `src/renderer/skia_image.rs` implement `DrawScene` and call into `render_node_tree` after setting the root transform and clipping to damage.
  - Transform helpers: `set_node_transform` in `src/drawing/scene.rs`.

- Layer drawing (per-node)
  - Core: `src/drawing/layer.rs::draw_layer` draws background, shadow, content (picture or dynamic callback), and border; returns content damage in layer space. Debug overlay: `draw_debug`.
  - Picture recording: `src/engine/draw_to_picture.rs::draw_layer_to_picture` wraps `draw_layer` into a Skia `Picture` with safe margins for effects.

- Draw caching
  - Picture cache (per-node): `src/engine/node/mod.rs`
    - `DrawCache` stores a recorded `Picture`; `SceneNode::repaint_if_needed` regenerates the picture when flags/size/content change.
    - `SceneNode::draw_cache` is consumed in `src/drawing/scene.rs::paint_node` to draw the cached picture efficiently.
  - Image cache (offscreen surface): `src/drawing/scene.rs`
    - For nodes with `SceneNode::is_image_cached()`, `render_node_tree` renders the subtree into an offscreen `Surface` and reuses the snapshot; helpers: `surface_for_node`, `set_surface_for_node`, `create_surface_for_node`.
  - Content caching at model level: `src/layers/layer/render_layer.rs::update_with_model_and_layout` records `content` as a `Picture` when a `draw_content` callback is present and size/handler change.
  - Model flags: `src/layers/layer/model.rs` exposes `image_cached` and `picture_cached` attributes toggled by the `Layer` API.

- Pointer interactions (hit-testing and events)
  - Handlers storage and dispatch: `src/engine/mod.rs`
    - Register/remove via `Engine::add_pointer_handler`, `remove_pointer_handler`, `remove_all_pointer_handlers`.
    - Event flow: `Engine::pointer_move` performs hit-testing (depth-first from topmost), updates hover state, and triggers `bubble_up_event` for `Move/In/Out`. `pointer_button_down`/`pointer_button_up` dispatch `Down/Up` on current hover.
    - Hit-testing: `src/engine/node/contains_point.rs::ContainsPoint` and `SceneNode::contains_point` use `RenderLayer.global_transformed_bounds`.
  - Public API on `Layer`: `src/layers/layer/mod.rs` methods `add_on_pointer_move/in/out/press/release` wire to engine pointer handlers; `pointer_events` attribute toggles participation.

- Scene utilities
  - Visibility culling helpers: `node_tree_list`/`node_tree_list_visible` in `src/drawing/scene.rs` build ordered lists and basic occlusion checks.

### Rendering Steps
Each frame, `Engine::update` advances the clock, updates animations, and applies scheduled transactions to set layout/paint flags. It runs Taffy via `update_layout_tree`, then `Engine::update_nodes` walks the scene (grouped by depth) to update each node’s `RenderLayer` from model+layout, regenerate cached pictures when repaint is needed, and accumulate per‑node damage which is propagated to ancestors and merged into `Engine.damage`. A renderer (e.g. `SkiaFboRenderer`) clips to the damage, applies the root transform, and calls `render_node_tree`, which traverses nodes, sets transforms, optionally renders image‑cached subtrees to offscreen surfaces, and paints either the cached picture or `draw_layer` content for each node with proper clipping/blend. Finally the renderer composites children and may flush; the client clears `Engine.damage` when appropriate.

## Data Models
- Engine
  - Orchestrates frames; holds `scene`, `layers` map, Taffy `layout_tree`, `animations`, `transactions`, `pointer_handlers`, `timestamp`, and frame `damage`. Key methods: `update`, `update_nodes`.
- Scene and SceneNode
  - `Scene` is a `TreeStorage<SceneNode>` wrapping an `indextree::Arena`; manages insertion, hierarchy and traversal.
  - `SceneNode` contains the drawable `RenderLayer`, rendering flags (`RenderableFlags`), `repaint_damage`, visibility/deletion state, caching flags (`image_cached`, `picture_cached`), `frame_number`, and optional `DrawCache`. Core methods: `update_render_layer_if_needed`, `repaint_if_needed`, `contains_point`.
- Layer API
  - `src/layers/layer/mod.rs::Layer` is the user-facing handle: owns a `ModelLayer` and a Taffy node id, provides setters for animatable properties, content callbacks, filters, and pointer handlers; schedules changes as transactions in the engine.
- ModelLayer (authoring state)
  - Declarative properties: position/size/anchor/scale/rotation, background/border/shadow, opacity, blend mode, `pointer_events`, content `draw_content`, filter fields, and caching toggles (`image_cached`, `picture_cached`). Defined in `src/layers/layer/model.rs`.
- RenderLayer (computed state)
  - Per-frame, ready-to-render snapshot derived from `ModelLayer` + layout + parent context: local/global bounds (rect and rrect), transforms (`M44`/`Matrix`), premultiplied opacity, clip flags, filters, and either cached `content` `Picture` or `content_draw_func`. Built by `RenderLayer::update_with_model_and_layout`.
- Caching
  - Picture cache: `SceneNode::draw_cache` stores a recorded `Picture` via `DrawCache` and is consumed in `drawing/scene.rs::paint_node`.
  - Image cache: per-node offscreen `Surface` managed in `src/drawing/scene.rs` when `SceneNode::is_image_cached()` is true.
- Animations and Transactions
  - `Animation`, `AnimationState`, `Transition` define timing; `AnimatedNodeChange` wraps a `SyncCommand` which returns `RenderableFlags`. Storage and processing in `src/engine/stages/mod.rs` and `src/engine/mod.rs`.
- Pointer Input
  - `PointerCallback` maps node ids to handlers; hover tracking and dispatch live in `Engine` (`pointer_move`, `pointer_button_down/up`, `bubble_up_event`). Hit-testing uses `SceneNode::contains_point` and `RenderLayer.global_transformed_bounds`.
