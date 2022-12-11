use layers::engine::node::RenderableFlags;

#[test]
pub fn node_flags() {
    // empty flags do not check for need paint
    let mut flags: RenderableFlags = RenderableFlags::empty();
    assert!(!flags.contains(RenderableFlags::NEEDS_PAINT));

    // set needs paint and check, needs_layout should be false
    flags.set(RenderableFlags::NEEDS_PAINT, true);
    assert!(flags.contains(RenderableFlags::NEEDS_PAINT));
    assert!(!flags.contains(RenderableFlags::NEEDS_LAYOUT));

    // set both needs paint and needs layout and check
    let mut flags: RenderableFlags = RenderableFlags::empty();
    let new_flags = RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT;
    flags.insert(new_flags);
    assert!(flags.contains(RenderableFlags::NEEDS_PAINT));
    assert!(flags.contains(RenderableFlags::NEEDS_LAYOUT));
}
