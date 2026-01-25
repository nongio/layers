# Code Review Instructions

When reviewing Layers code changes, focus on issues that **genuinely matter**—bugs, safety problems, logic errors, and architectural violations. Do not comment on style, formatting, or trivial matters.

## Review Focus Areas

### Rust Safety & Correctness

**Priority issues:**
- Unsafe code without proper justification or safety comments
- Potential panics (unwrap, expect, indexing without bounds check)
- Lifetime issues or potential use-after-free
- Deadlocks or lock ordering problems
- Race conditions in async code or event loop
- Resource leaks (file descriptors, GPU buffers, Wayland objects)

### Rendering & Graphics Pipeline

**Critical issues:**
- Damage tracking correctness: Check that `RenderableFlags` are set properly when node properties change
- Incorrect damage propagation: Verify `propagate_damage_to_ancestors` is called when needed
- Cache invalidation bugs: Picture/image cache must be cleared when content changes
- Transform composition errors: Local/global transform calculations in `RenderLayer`
- Skia resource leaks: Pictures, Surfaces, Images must be properly managed
- Blend mode or opacity calculations that don't premultiply correctly

**Reference:**
- Damage tracking: `src/engine/stages/mod.rs` and `docs/damage-tracking.md`
- Drawing: `src/drawing/scene.rs` and `src/drawing/layer.rs`
- Caching: `src/engine/node/mod.rs` (DrawCache), `src/drawing/scene.rs` (image cache)

### Engine Architecture

**Critical issues:**
- Breaking the update pipeline stages: animations → transactions → layout → update_nodes → damage
- Modifying scene tree outside of transactions
- Direct mutation of `RenderLayer` instead of using `ModelLayer` + transactions
- Incorrect Taffy layout updates or missing `compute_layout` calls
- Animation timing bugs in `AnimationState` or `Transition`

**Reference:**
- Pipeline: `src/engine/mod.rs::Engine::update` and `docs/engine-update-stages.md`
- Transactions: `src/engine/stages/mod.rs::execute_transactions`

### Pointer Input & Hit Testing

**Critical issues:**
- Hit-testing logic errors in `contains_point` (using wrong bounds or transforms)
- Event bubbling mistakes: hover state, enter/leave events
- Memory leaks from handler registration (must be paired with removal)
- Using stale `global_transformed_bounds` for hit-testing

**Reference:**
- `src/engine/mod.rs` (pointer_move, pointer_button_down/up)
- `src/engine/node/contains_point.rs`
- `docs/pointers.md`

### Performance

**Flag only if severe:**
- O(n²) or worse algorithms in hot paths (frame loop, layout, hit-testing)
- Unnecessary full-tree traversals instead of dirty-node iteration
- Allocations in per-frame hot loops
- Rendering entire scene instead of using damage regions

### Testing

**Required for:**
- Changes to damage tracking logic → update or add tests in `tests/damage.rs`
- New animation features → benchmark in `benches/` if performance-critical
- Pointer input changes → manual testing recommended

### Documentation

**Critical:** Keep `docs/` folder in sync with code changes

**Update required when:**
- Modifying damage tracking pipeline → update `docs/damage-tracking.md` or `docs/damage.md`
- Changing engine update stages → update `docs/engine-update-stages.md`
- Altering pointer event flow → update `docs/pointers.md`
- Adding/changing layer followers behavior → update `docs/layer-followers.md`
- Modifying inspector protocol → update `docs/layers_inspector.md`
- Portal system changes → update `docs/portals.md`

**Documentation files:**
- `docs/damage-tracking.md` - Damage tracking system
- `docs/engine-update-stages.md` - Engine update pipeline
- `docs/pointers.md` - Pointer input handling
- `docs/layer-followers.md` - Layer followers
- `docs/layers_inspector.md` - Layers inspector
- `docs/portals.md` - Portals

## What NOT to Review

- Code style, formatting (handled by rustfmt)
- Clippy warnings (handled by CI)
- Naming preferences (unless genuinely confusing)
- Minor optimizations without profiling data
- Documentation style (unless factually wrong)
