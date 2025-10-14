# Damage Notes

This document supplements `docs/damage-tracking.md` with situational details that
matter when reasoning about container damage in recent regressions.

## Effective visibility for parents

Transparent container layers no longer suppress geometry damage when they own
visible descendants. During `update_node_single` we compute an *effective*
visibility flag that ORs the parent’s own drawables with any visible content in
its sub-tree. When the parent moves or changes layout, the union of the previous
and new bounds for the entire sub-tree is joined into the damage rectangle.

This fixes scenarios where a parent had no background/border but wrapped a child
with paintable content. Moving the parent now damages the child’s previous and
new positions, which keeps the scene from leaving stale pixels behind.

## Testing

- `tests/damage.rs::damage_move_parent_with_visible_child` verifies that moving a
  transparent parent produces the expected union damage.
- Unit tests in `src/engine/stages/update_node.rs` check the effective visibility
  logic directly.

When adapting the pipeline, add similar coverage for parent/child interactions
to avoid future regressions.
