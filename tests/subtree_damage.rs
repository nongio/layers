#[cfg(test)]
mod tests {
    use layers::{
        prelude::*,
        types::{Color, Size},
    };

    /// Two sibling subtrees, each a container with one colored child.
    /// The roots are absolutely positioned (like a compositor's per-plane
    /// containers) so changes in one subtree cannot relayout the other.
    /// Returns (engine, root_a, child_a, root_b).
    fn two_subtrees() -> (std::sync::Arc<Engine>, Layer, Layer, Layer) {
        let engine = Engine::create(1000.0, 1000.0);
        let absolute = layers::taffy::style::Style {
            position: layers::taffy::style::Position::Absolute,
            ..Default::default()
        };

        let root_a = engine.new_layer();
        root_a.set_layout_style(absolute.clone());
        root_a.set_position((0.0, 0.0), None);
        root_a.set_size(Size::points(400.0, 400.0), None);
        engine.add_layer(&root_a).unwrap();

        let child_a = engine.new_layer();
        child_a.set_size(Size::points(100.0, 100.0), None);
        child_a.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.append_layer(&child_a.id, Some(root_a.id)).unwrap();

        let root_b = engine.new_layer();
        root_b.set_layout_style(absolute);
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
    pub fn removed_node_damage_attributes_to_surviving_ancestor() {
        let (engine, root_a, child_a, root_b) = two_subtrees();
        engine.clear_damage();

        // A removed node's damage lands on its nearest surviving ancestor,
        // so only the subtree the removal happened in reports it.
        child_a.remove();
        engine.update(0.016);

        assert!(
            engine.subtree_damage(root_a.id).is_some(),
            "removal damages its own subtree"
        );
        assert!(
            engine.subtree_damage(root_b.id).is_none(),
            "removal must not damage unrelated subtrees"
        );

        // ...and clears like any other damage.
        engine.clear_damage();
        assert!(engine.subtree_damage(root_a.id).is_none());
        assert!(engine.subtree_damage(root_b.id).is_none());
    }
}
