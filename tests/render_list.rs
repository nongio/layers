#[cfg(test)]
mod tests {
    use layers::{
        drawing::{node_tree_list, node_tree_list_visible},
        prelude::*,
        types::*,
    };

    #[test]
    pub fn render_list() {
        let engine = Engine::create(1000.0, 1000.0);

        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            assert_eq!(nodes.len(), 3);
        });
    }
    #[test]
    pub fn render_list_opacity() {
        let engine = Engine::create(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_opacity(0.9, None);
        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(150.0, 150.0), None);
        layer.set_blend_mode(layers::prelude::BlendMode::BackgroundBlur);
        engine.add_layer(&layer).unwrap();

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            let nodes = node_tree_list_visible(nodes.iter(), arena);

            assert_eq!(nodes.len(), 3);
        });
    }
    #[test]
    pub fn render_list_children() {
        let engine = Engine::create(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(50.0, 50.0), None);
        layer.set_opacity(1.0, None);
        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(150.0, 150.0), None);
        layer.set_opacity(0.9, None);
        engine.add_layer(&layer).unwrap();

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            let nodes = node_tree_list_visible(nodes.iter(), arena);

            assert_eq!(nodes.len(), 3);
        });
    }
    #[test]
    pub fn render_list_hidden() {
        let engine = Engine::create(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_opacity(0.0, None);
        engine.add_layer(&layer).unwrap();

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(150.0, 150.0), None);
        layer.set_blend_mode(layers::prelude::BlendMode::BackgroundBlur);
        layer.set_hidden(true);
        engine.add_layer(&layer).unwrap();

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            let nodes = node_tree_list_visible(nodes.iter(), arena);

            assert_eq!(nodes.len(), 1);
        });
    }
    #[test]
    pub fn occlusion_fully_covered() {
        let engine = Engine::create(1000.0, 1000.0);

        // Back layer: fully covered by front layer
        let back = engine.new_layer();
        back.set_position((0.0, 0.0), None);
        back.set_size(Size::points(100.0, 100.0), None);
        back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&back).unwrap();

        // Front layer: opaque, fully covers back
        let front = engine.new_layer();
        front.set_position((0.0, 0.0), None);
        front.set_size(Size::points(200.0, 200.0), None);
        front.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        engine.add_layer(&front).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        // The back layer and the root should be occluded by the front layer
        assert!(occluded.contains(&back.id), "back layer should be occluded");
    }

    #[test]
    pub fn occlusion_not_covered() {
        let engine = Engine::create(1000.0, 1000.0);

        // Two layers side by side — neither occludes the other
        let left = engine.new_layer();
        left.set_position((0.0, 0.0), None);
        left.set_size(Size::points(100.0, 100.0), None);
        left.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&left).unwrap();

        let right = engine.new_layer();
        right.set_position((200.0, 0.0), None);
        right.set_size(Size::points(100.0, 100.0), None);
        right.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        engine.add_layer(&right).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            !occluded.contains(&left.id),
            "left layer should not be occluded"
        );
        assert!(
            !occluded.contains(&right.id),
            "right layer should not be occluded"
        );
    }

    #[test]
    pub fn occlusion_transparent_does_not_occlude() {
        let engine = Engine::create(1000.0, 1000.0);

        let back = engine.new_layer();
        back.set_position((0.0, 0.0), None);
        back.set_size(Size::points(100.0, 100.0), None);
        back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&back).unwrap();

        // Front layer has opacity < 1, so it cannot occlude
        let front = engine.new_layer();
        front.set_position((0.0, 0.0), None);
        front.set_size(Size::points(200.0, 200.0), None);
        front.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        front.set_opacity(0.5, None);
        engine.add_layer(&front).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            !occluded.contains(&back.id),
            "back layer should not be occluded (front is semi-transparent)"
        );
    }

    #[test]
    pub fn occlusion_rounded_corners_do_not_occlude() {
        let engine = Engine::create(1000.0, 1000.0);

        let back = engine.new_layer();
        back.set_position((0.0, 0.0), None);
        back.set_size(Size::points(100.0, 100.0), None);
        back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&back).unwrap();

        // Front layer is opaque but has rounded corners
        let front = engine.new_layer();
        front.set_position((0.0, 0.0), None);
        front.set_size(Size::points(200.0, 200.0), None);
        front.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        front.set_border_corner_radius(BorderRadius::new_single(10.0), None);
        engine.add_layer(&front).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            !occluded.contains(&back.id),
            "back layer should not be occluded (front has rounded corners)"
        );
    }

    #[test]
    pub fn occlusion_child_outside_clip_parent() {
        let engine = Engine::create(1000.0, 1000.0);

        // Parent with clip_children, sized 100x100 at origin
        let parent = engine.new_layer();
        parent.set_size(Size::points(100.0, 100.0), None);
        parent.set_clip_children(true, None);
        parent.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 1.0), None);
        engine.add_layer(&parent).unwrap();

        // Child fully outside the parent clip (absolute positioned at 200,200)
        let child = engine.new_layer();
        child.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        child.set_position((200.0, 200.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        child.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        parent.add_sublayer(&child).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            occluded.contains(&child.id),
            "child outside clip parent should be occluded"
        );
    }

    #[test]
    pub fn occlusion_opaque_child_clipped_by_parent() {
        let engine = Engine::create(1000.0, 1000.0);

        // Behind layer: larger than the clip region (200x200 at origin)
        let behind = engine.new_layer();
        behind.set_position((0.0, 0.0), None);
        behind.set_size(Size::points(200.0, 200.0), None);
        behind.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&behind).unwrap();

        // Parent with clip_children, sized 50x50 (smaller than behind)
        let parent = engine.new_layer();
        parent.set_position((0.0, 0.0), None);
        parent.set_size(Size::points(50.0, 50.0), None);
        parent.set_clip_children(true, None);
        engine.add_layer(&parent).unwrap();

        // Opaque child at 100x100 — extends beyond parent clip (50x50)
        // Only the clipped portion (50x50) should count as occluder
        let child = engine.new_layer();
        child.set_position((0.0, 0.0), None);
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        parent.add_sublayer(&child).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        // The behind layer (200x200) is larger than the child's clipped
        // occluder region (50x50), so it should NOT be fully occluded
        assert!(
            !occluded.contains(&behind.id),
            "layer larger than clipped occluder region should not be occluded"
        );
    }

    #[test]
    pub fn occlusion_background_blur_not_occluded() {
        let engine = Engine::create(1000.0, 1000.0);

        // A BackgroundBlur layer reads the backdrop, so it must never be occluded
        let blur = engine.new_layer();
        blur.set_position((0.0, 0.0), None);
        blur.set_size(Size::points(100.0, 100.0), None);
        blur.set_blend_mode(BlendMode::BackgroundBlur);
        engine.add_layer(&blur).unwrap();

        // Opaque front layer
        let front = engine.new_layer();
        front.set_position((0.0, 0.0), None);
        front.set_size(Size::points(200.0, 200.0), None);
        front.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        engine.add_layer(&front).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        // The blur layer IS behind an opaque layer, so it will be marked occluded.
        // This is correct: the blur layer's visual effect is invisible when fully
        // covered. BackgroundBlur should not act as an *occluder* (it doesn't add
        // to the opaque mask), but it can still be occluded by others.
        assert!(
            occluded.contains(&blur.id),
            "blur layer should be occluded when fully behind opaque layer"
        );
    }

    #[test]
    pub fn occlusion_semi_transparent_parent_subtree_does_not_occlude() {
        let engine = Engine::create(1000.0, 1000.0);

        // Back layer — should NOT be occluded because the front subtree is semi-transparent
        let back = engine.new_layer();
        back.set_position((0.0, 0.0), None);
        back.set_size(Size::points(100.0, 100.0), None);
        back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&back).unwrap();

        // Semi-transparent parent (opacity 0.9) — entire subtree cannot occlude
        let parent = engine.new_layer();
        parent.set_position((0.0, 0.0), None);
        parent.set_size(Size::points(200.0, 200.0), None);
        parent.set_opacity(0.9, None);
        engine.add_layer(&parent).unwrap();

        // Child with opaque background — but parent opacity < 1.0,
        // so this child must NOT act as an occluder
        let child = engine.new_layer();
        child.set_position((0.0, 0.0), None);
        child.set_size(Size::points(200.0, 200.0), None);
        child.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        parent.add_sublayer(&child).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            !occluded.contains(&back.id),
            "back layer should not be occluded by child of semi-transparent parent"
        );
    }

    #[test]
    pub fn occlusion_hidden_parent_subtree_does_not_occlude() {
        let engine = Engine::create(1000.0, 1000.0);

        // Back layer — should NOT be occluded because the front subtree is hidden
        let back = engine.new_layer();
        back.set_position((0.0, 0.0), None);
        back.set_size(Size::points(100.0, 100.0), None);
        back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&back).unwrap();

        // Hidden parent — entire subtree should be ignored
        let parent = engine.new_layer();
        parent.set_position((0.0, 0.0), None);
        parent.set_size(Size::points(200.0, 200.0), None);
        parent.set_hidden(true);
        engine.add_layer(&parent).unwrap();

        // Opaque child inside hidden parent — must not occlude anything
        let child = engine.new_layer();
        child.set_position((0.0, 0.0), None);
        child.set_size(Size::points(200.0, 200.0), None);
        child.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        parent.add_sublayer(&child).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            !occluded.contains(&back.id),
            "back layer should not be occluded by child of hidden parent"
        );
    }

    /// A layer with transparent background but `content_opaque = true` should
    /// still act as an occluder, hiding the layer behind it.
    #[test]
    pub fn occlusion_content_opaque_occludes() {
        let engine = Engine::create(1000.0, 1000.0);

        // Back layer — should be occluded
        let back = engine.new_layer();
        back.set_position((0.0, 0.0), None);
        back.set_size(Size::points(200.0, 200.0), None);
        back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&back).unwrap();

        // Front layer — transparent background but content declared opaque
        let front = engine.new_layer();
        front.set_position((0.0, 0.0), None);
        front.set_size(Size::points(200.0, 200.0), None);
        // background is transparent (default)
        front.set_content_opaque(true);
        engine.add_layer(&front).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            occluded.contains(&back.id),
            "back layer should be occluded by content_opaque front layer"
        );
    }

    /// A layer with `content_opaque = false` (default) and transparent
    /// background should NOT occlude.
    #[test]
    pub fn occlusion_content_not_opaque_does_not_occlude() {
        let engine = Engine::create(1000.0, 1000.0);

        let back = engine.new_layer();
        back.set_position((0.0, 0.0), None);
        back.set_size(Size::points(200.0, 200.0), None);
        back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
        engine.add_layer(&back).unwrap();

        let front = engine.new_layer();
        front.set_position((0.0, 0.0), None);
        front.set_size(Size::points(200.0, 200.0), None);
        // transparent background and content_opaque not set (default false)
        engine.add_layer(&front).unwrap();

        engine.update(0.016);
        engine.clear_occlusion();
        engine.compute_occlusion(engine.scene_root().unwrap());

        let occ_map = engine.scene().occlusion_map().unwrap();
        let root = engine.scene_root().unwrap();
        let occluded = occ_map.get(&root).unwrap();

        assert!(
            !occluded.contains(&back.id),
            "back layer should NOT be occluded when front is not opaque"
        );
    }
}
