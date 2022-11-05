use hello::engine::node::NodeFlags;
use hello::engine::scene::Scene;
use hello::layers::layer::ModelLayer;
use hello::types::Point;
use std::sync::Arc;

#[test]
pub fn node_flags() {
    let mut flags: NodeFlags = NodeFlags::empty();
    assert!(!flags.contains(NodeFlags::NEEDS_PAINT));

    flags.set(NodeFlags::NEEDS_PAINT, true);
    assert!(flags.contains(NodeFlags::NEEDS_PAINT));
    assert!(!flags.contains(NodeFlags::NEEDS_LAYOUT));

    let mut flags: NodeFlags = NodeFlags::empty();
    let new_flags = NodeFlags::NEEDS_PAINT | NodeFlags::NEEDS_LAYOUT;
    flags.insert(new_flags);
    assert!(flags.contains(NodeFlags::NEEDS_PAINT));
    assert!(flags.contains(NodeFlags::NEEDS_LAYOUT));
}
