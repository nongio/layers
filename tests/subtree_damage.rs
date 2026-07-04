#[cfg(test)]
mod tests {
    use layers::{
        prelude::*,
        types::{Color, Size},
    };

    /// Two sibling subtrees, each a container with one colored child.
    /// Returns (engine, root_a, child_a, root_b).
    fn two_subtrees() -> (std::sync::Arc<Engine>, Layer, Layer, Layer) {
        let engine = Engine::create(1000.0, 1000.0);

        let root_a = engine.new_layer();
        root_a.set_position((0.0, 0.0), None);
        root_a.set_size(Size::points(400.0, 400.0), None);
        engine.add_layer(&root_a).unwrap();

        let child_a = engine.new_layer();
        child_a.set_size(Size::points(100.0, 100.0), None);
        child_a.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.append_layer(&child_a.id, Some(root_a.id)).unwrap();

        let root_b = engine.new_layer();
        root_b.set_position((500.0, 500.0), None);
        root_b.set_size(Size::points(400.0, 400.0), None);
        engine.add_layer(&root_b).unwrap();

        let child_b = engine.new_layer();
        child_b.set_size(Size::points(100.0, 100.0), None);
        child_b.set_background_color(Color::new_rgba(0.0, 1.0, 0.0, 1.0), None);
        engine.append_layer(&child_b.id, Some(root_b.id)).unwrap();

        engine.update(0.016);
        (engine, root_a, child_a, root_b)
    }

    #[test]
    pub fn subtree_damage_attributes_to_the_damaged_subtree() {
        let (engine, root_a, child_a, root_b) = two_subtrees();

        // Initial construction damages both subtrees.
        assert!(engine.subtree_damage(root_a.id).is_some());
        assert!(engine.subtree_damage(root_b.id).is_some());

        engine.clear_damage();

        // Damage only subtree A.
        child_a.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        engine.update(0.016);

        assert!(
            engine.subtree_damage(root_a.id).is_some(),
            "damaged subtree must report damage"
        );
        assert!(
            engine.subtree_damage(root_b.id).is_none(),
            "untouched subtree must not report damage"
        );
    }

    #[test]
    pub fn subtree_damage_cleared_by_clear_damage() {
        let (engine, root_a, child_a, root_b) = two_subtrees();
        engine.clear_damage();

        child_a.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        engine.update(0.016);
        assert!(engine.subtree_damage(root_a.id).is_some());

        engine.clear_damage();
        assert!(engine.subtree_damage(root_a.id).is_none());
        assert!(engine.subtree_damage(root_b.id).is_none());

        // A quiet update produces no new damage.
        engine.update(0.016);
        assert!(engine.subtree_damage(root_a.id).is_none());
    }

    #[test]
    pub fn removed_node_damage_is_conservative() {
        let (engine, root_a, child_a, root_b) = two_subtrees();
        engine.clear_damage();

        // Removing a node can't be attributed to a subtree after the fact —
        // it must conservatively damage every subtree query.
        child_a.remove();
        engine.update(0.016);

        assert!(
            engine.subtree_damage(root_a.id).is_some(),
            "removal damages its own subtree"
        );
        assert!(
            engine.subtree_damage(root_b.id).is_some(),
            "removed-node damage is conservative: all subtrees report it"
        );

        // ...until the next clear.
        engine.clear_damage();
        assert!(engine.subtree_damage(root_a.id).is_none());
        assert!(engine.subtree_damage(root_b.id).is_none());
    }
}
