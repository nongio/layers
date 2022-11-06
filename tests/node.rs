use hello::engine::node::RenderableFlags;
use hello::engine::scene::Scene;
use hello::layers::layer::ModelLayer;
use hello::types::Point;
use std::sync::Arc;

#[test]
pub fn node_flags() {
    let mut flags: RenderableFlags = RenderableFlags::empty();
    assert!(!flags.contains(RenderableFlags::NEEDS_PAINT));

    flags.set(RenderableFlags::NEEDS_PAINT, true);
    assert!(flags.contains(RenderableFlags::NEEDS_PAINT));
    assert!(!flags.contains(RenderableFlags::NEEDS_LAYOUT));

    let mut flags: RenderableFlags = RenderableFlags::empty();
    let new_flags = RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT;
    flags.insert(new_flags);
    assert!(flags.contains(RenderableFlags::NEEDS_PAINT));
    assert!(flags.contains(RenderableFlags::NEEDS_LAYOUT));
}
