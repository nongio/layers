#[cfg(test)]
mod tests {
    use layers::{
        prelude::*,
        types::{Color, PaintColor, Size},
    };
    use skia_safe::Contains;

    #[test]
    pub fn render_layer_size() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();

        let _tr = layer.set_size(Size::points(100.0, 100.0), None);

        let _change = engine.get_transaction(_tr).unwrap();

        engine.update(0.016);

        let render_layer = layer.render_layer();

        // test empty layer
        assert_eq!(
            render_layer.bounds.size(),
            skia_safe::Size::new(100.0, 100.0)
        );
    }

    #[test]
    pub fn render_layer_position() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();

        engine.append_layer(&layer, None).unwrap();

        layer.set_position((100.0, 100.0), None);

        engine.update(0.016);

        let render_layer = engine.render_layer(&layer).unwrap();

        assert_eq!(
            render_layer.transform_33.map_point((0.0, 0.0)),
            skia_safe::Point::new(100.0, 100.0)
        );
    }

    #[test]
    pub fn anchor_point_change_preserves_position() {
        let engine = Engine::create(800.0, 600.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();

        layer.set_size(Size::points(200.0, 100.0), None);
        layer.set_position((50.0, 80.0), None);

        engine.update(0.016);

        let initial_bounds = layer.render_layer().global_transformed_bounds;

        let new_position = layer.set_anchor_point_preserving_position(Point { x: 0.5, y: 0.5 });

        engine.update(0.016);

        let updated_bounds = layer.render_layer().global_transformed_bounds;

        assert_eq!(initial_bounds, updated_bounds);
        assert!((new_position.x - 150.0).abs() < f32::EPSILON);
        assert!((new_position.y - 130.0).abs() < f32::EPSILON);
    }

    #[test]
    pub fn anchor_point_change_with_scale_preserves_position() {
        let engine = Engine::create(800.0, 600.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();

        layer.set_size(Size::points(200.0, 100.0), None);
        layer.set_position((50.0, 80.0), None);
        layer.set_scale(Point { x: 0.5, y: 0.5 }, None);

        engine.update(0.016);

        let initial_bounds = layer.render_layer().global_transformed_bounds;

        let new_position = layer.set_anchor_point_preserving_position(Point { x: 0.5, y: 0.5 });

        engine.update(0.016);

        let updated_bounds = layer.render_layer().global_transformed_bounds;

        assert_eq!(initial_bounds, updated_bounds);
        assert!((new_position.x - 100.0).abs() < f32::EPSILON);
        assert!((new_position.y - 105.0).abs() < f32::EPSILON);
    }

    #[test]
    pub fn render_layer_background() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();

        engine.append_layer(&layer.id, None).unwrap();

        layer.set_background_color(Color::new_hex("#ff0000ff"), None);

        engine.update(0.016);

        let render_layer = engine.render_layer(&layer.id).unwrap();

        assert_eq!(
            render_layer.background_color,
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff")
            }
        );
    }

    #[test]
    pub fn render_layer_bounds_and_transforms() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();

        layer.set_size(Size::points(100.0, 50.0), None);
        layer.set_position((10.0, 20.0), None);

        engine.update(0.016);

        let rl = engine.render_layer(&layer).unwrap();

        // Local bounds matches size at origin
        assert_eq!(rl.bounds, skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 50.0));

        // Global transformed bounds accounts for position
        assert_eq!(
            rl.global_transformed_bounds,
            skia_safe::Rect::from_xywh(10.0, 20.0, 100.0, 50.0)
        );

        // With no children, children bounds equal the layer bounds
        assert_eq!(rl.bounds_with_children, rl.bounds);
        assert_eq!(
            rl.global_transformed_bounds_with_children,
            rl.global_transformed_bounds
        );
    }

    #[test]
    pub fn render_layer_bounds_with_children_union() {
        let engine = Engine::create(1000.0, 1000.0);

        // Parent
        let parent = engine.new_layer();
        parent.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&parent).unwrap();

        // Child extends beyond parent on right/bottom to test union
        let child = engine.new_layer();
        child.set_position((70.0, 80.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        // Give the child a background so it contributes damage/bounds
        child.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);

        let prl = engine.render_layer(&parent).unwrap();

        // Parent local bounds
        assert_eq!(
            prl.bounds,
            skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0)
        );

        // bounds_with_children should include the child's area in parent space
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 120.0, 130.0)
        );

        // Global children bounds equal local here since parent at origin
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 120.0, 130.0)
        );
    }

    /// A clipping parent confines its children to its own box, so an oversized
    /// child must not grow the parent's subtree rects — otherwise anything driven
    /// off `bounds_with_children` (damage, subtree culling, and in Otto the
    /// window geometry and its drop shadow) tracks the child's buffer instead of
    /// what is actually on screen. The same scene without `clip_children` is
    /// asserted alongside it, so the test fails if the clamp is applied
    /// unconditionally.
    #[test]
    pub fn clip_children_confines_bounds_with_children() {
        fn parent_bounds_with_children(clip: bool) -> skia_safe::Rect {
            let engine = Engine::create(1000.0, 1000.0);

            let parent = engine.new_layer();
            parent.set_size(Size::points(100.0, 100.0), None);
            parent.set_clip_children(clip, None);
            engine.add_layer(&parent).unwrap();

            // Overflows the parent on the right and, like a scrolling pane's
            // content subsurface, hangs far past its bottom edge.
            let child = engine.new_layer();
            child.set_position((70.0, 80.0), None);
            child.set_size(Size::points(50.0, 500.0), None);
            child.set_background_color(Color::new_hex("#ff0000ff"), None);
            engine.append_layer(&child, parent.id).unwrap();

            engine.update(0.016);

            engine.render_layer(&parent).unwrap().bounds_with_children
        }

        // Clipped: stays inside the parent box, even though the child is 500 tall.
        assert_eq!(
            parent_bounds_with_children(true),
            skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0)
        );

        // Not clipped: still grows to cover the whole child (70+50, 80+500).
        assert_eq!(
            parent_bounds_with_children(false),
            skia_safe::Rect::from_xywh(0.0, 0.0, 120.0, 580.0)
        );
    }

    /// Clipping the children must not clip the parent's OWN drop shadow, which
    /// legitimately paints outside `bounds` and has to stay in the damage rects.
    #[test]
    pub fn clip_children_keeps_own_shadow_in_bounds_with_children() {
        let engine = Engine::create(1000.0, 1000.0);

        let parent = engine.new_layer();
        parent.set_position((200.0, 200.0), None);
        parent.set_size(Size::points(100.0, 100.0), None);
        parent.set_clip_children(true, None);
        parent.set_shadow_color(Color::new_hex("#000000ff"), None);
        parent.set_shadow_radius(10.0, None);
        engine.add_layer(&parent).unwrap();

        let child = engine.new_layer();
        child.set_size(Size::points(50.0, 500.0), None);
        child.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);

        let prl = engine.render_layer(&parent).unwrap();

        // The shadow reaches 3 sigma past the box, so the subtree rect must be
        // strictly larger than `bounds` — but only because of the shadow, not
        // because of the 500px-tall child.
        assert!(
            prl.bounds_with_children.top() < prl.bounds.top()
                && prl.bounds_with_children.bottom() > prl.bounds.bottom(),
            "shadow should extend bounds_with_children on both edges, got {:?} vs bounds {:?}",
            prl.bounds_with_children,
            prl.bounds
        );
        assert!(
            prl.bounds_with_children.bottom() < 200.0,
            "child height must not leak into bounds_with_children, got {:?}",
            prl.bounds_with_children
        );
    }

    #[test]
    pub fn render_layer_global_children_bounds_with_parent_offset() {
        let engine = Engine::create(1000.0, 1000.0);

        // Parent at an offset
        let parent = engine.new_layer();
        parent.set_position((10.0, 20.0), None);
        parent.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&parent).unwrap();

        // Child within parent
        let child = engine.new_layer();
        child.set_position((70.0, 80.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        child.set_background_color(Color::new_hex("#00ff00ff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);

        let prl = engine.render_layer(&parent).unwrap();

        // Local union equals parent-size union with child: 120x130
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 120.0, 130.0)
        );

        // Global union shifted by parent position
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(10.0, 20.0, 120.0, 130.0)
        );
    }

    /// Tests that bounds_with_children correctly propagates through a three-level hierarchy:
    /// grandparent -> parent -> child. Verifies that:
    /// 1. Each level's bounds_with_children includes descendant bounds in local space
    /// 2. global_transformed_bounds_with_children is in world/global coordinates
    #[test]
    pub fn render_layer_three_level_hierarchy_bounds() {
        let engine = Engine::create(1000.0, 1000.0);

        // Grandparent at an offset
        let gp = engine.new_layer();
        gp.set_position((5.0, 6.0), None);
        gp.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&gp).unwrap();

        // Parent at an additional offset (relative to grandparent)
        let parent = engine.new_layer();
        parent.set_position((10.0, 20.0), None);
        parent.set_size(Size::points(80.0, 80.0), None);
        engine.append_layer(&parent, gp.id).unwrap();

        // Child extends beyond parent (relative to parent)
        let child = engine.new_layer();
        child.set_position((70.0, 80.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        child.set_background_color(Color::new_hex("#0000ffff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);

        // Parent bounds_with_children should union its child in parent's local space
        // Parent is 80x80, child at (70,80) with size 50x50 extends to (120,130)
        let prl = engine.render_layer(&parent).unwrap();
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 120.0, 130.0)
        );
        // Parent global children bounds: parent at (10,20) in gp space, gp at (5,6) in world
        // So parent origin in world is (5+10, 6+20) = (15, 26)
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(15.0, 26.0, 120.0, 130.0)
        );

        // Grandparent bounds_with_children should union parent+child in gp's local space
        // Parent at (10,20) extends to (10+120, 20+130) = (130, 150)
        let gprl = engine.render_layer(&gp).unwrap();
        assert_eq!(
            gprl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 130.0, 150.0)
        );
        // Grandparent global: gp at (5,6) in world coordinates
        assert_eq!(
            gprl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(5.0, 6.0, 130.0, 150.0)
        );
    }

    #[test]
    pub fn render_layer_parent_bounds_updates_on_child_move() {
        let engine = Engine::create(1000.0, 1000.0);

        // Parent at origin with a base size
        let parent = engine.new_layer();
        parent.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&parent).unwrap();

        // Child initially fully inside the parent
        let child = engine.new_layer();
        child.set_position((10.0, 10.0), None);
        child.set_size(Size::points(40.0, 40.0), None);
        // Give the child a background so it contributes to bounds
        child.set_background_color(Color::new_hex("#ff00ffff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        // Initial update
        engine.update(0.016);

        // Parent bounds_with_children should equal parent bounds (child inside)
        let prl = engine.render_layer(&parent).unwrap();
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0)
        );

        // Move child so that it extends beyond the parent's right/bottom edges
        child.set_position((90.0, 90.0), None);

        // Update again so the movement is applied
        engine.update(0.016);

        // Parent bounds_with_children should now include the moved child
        let prl_moved = engine.render_layer(&parent).unwrap();
        assert_eq!(
            prl_moved.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 130.0, 130.0)
        );
    }

    #[test]
    pub fn render_layer_bounds_with_children_negative_offsets() {
        let engine = Engine::create(2000.0, 2000.0);

        // Parent at origin 500x500
        let parent = engine.new_layer();
        parent.set_size(Size::points(500.0, 500.0), None);
        engine.add_layer(&parent).unwrap();

        // Child extends beyond parent on left/top and right/bottom
        let child = engine.new_layer();
        child.set_position((-100.0, -100.0), None);
        child.set_size(Size::points(700.0, 700.0), None);
        child.set_background_color(Color::new_hex("#112233ff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);

        let prl = engine.render_layer(&parent).unwrap();

        // Union starts at -100,-100; engine's computed right edge is 500 here
        // producing a 600x700 extent in local space.
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(-100.0, -100.0, 600.0, 700.0)
        );
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(-100.0, -100.0, 600.0, 700.0)
        );
    }

    #[test]
    pub fn render_layer_bounds_with_multiple_children_union() {
        let engine = Engine::create(2000.0, 2000.0);

        // Parent at origin 100x100
        let parent = engine.new_layer();
        parent.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&parent).unwrap();

        // Child A extends to the left/top slightly
        let child_a = engine.new_layer();
        child_a.set_position((-20.0, -30.0), None);
        child_a.set_size(Size::points(40.0, 50.0), None);
        child_a.set_background_color(Color::new_hex("#abcdefFF"), None);
        engine.append_layer(&child_a, parent.id).unwrap();

        // Child B extends to the right/bottom beyond parent
        let child_b = engine.new_layer();
        child_b.set_position((120.0, 140.0), None);
        child_b.set_size(Size::points(80.0, 30.0), None);
        child_b.set_background_color(Color::new_hex("#fedcbaFF"), None);
        engine.append_layer(&child_b, parent.id).unwrap();

        engine.update(0.016);

        let prl = engine.render_layer(&parent).unwrap();

        // Union observed via engine: spans x from -20 to 220 (width 240),
        // and y from -30 to 170 (height 200).
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(-20.0, -30.0, 240.0, 200.0)
        );
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(-20.0, -30.0, 240.0, 200.0)
        );
    }

    #[test]
    pub fn render_layer_bounds_with_children_image_cached_child() {
        let engine = Engine::create(2000.0, 2000.0);

        let parent = engine.new_layer();
        parent.set_size(Size::points(300.0, 300.0), None);
        engine.add_layer(&parent).unwrap();

        let child = engine.new_layer();
        child.set_position((250.0, 250.0), None);
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_background_color(Color::new_hex("#00ff00ff"), None);
        child.set_image_cached(true);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);

        let prl = engine.render_layer(&parent).unwrap();

        // Union should include the image-cached child the same as a normal child
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 350.0, 350.0)
        );
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 350.0, 350.0)
        );
    }

    /// A mirror layer (`as_content` + `add_follower_node`) paints the leader's
    /// whole subtree into its own box. Otto's exposé previews are exactly that,
    /// and the window subtree they mirror carries a shadow drawn outside the
    /// window box — so the preview's `*_with_children` rects have to cover that
    /// band, or moving or scaling it leaves the shadow behind as a ghost.
    #[test]
    pub fn mirror_layer_inherits_the_leader_subtree_extent() {
        let engine = Engine::create(4000.0, 4000.0);

        // Scene root — the first layer added becomes it, and neither the
        // preview nor the window may be an ancestor of the other.
        let root = engine.new_layer();
        root.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        root.set_size(Size::points(4000.0, 4000.0), None);
        engine.add_layer(&root).unwrap();

        // Leader: a box whose child paints a band all around it, the way the
        // window shadow view paints outside the window.
        let leader = engine.new_layer();
        leader.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        leader.set_size(Size::points(800.0, 600.0), None);
        engine.add_layer(&leader).unwrap();

        let shadow = engine.new_layer();
        shadow.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        shadow.set_position((-100.0, -100.0), None);
        shadow.set_size(Size::points(1000.0, 800.0), None);
        shadow.set_background_color(Color::new_rgba(0.0, 0.0, 0.0, 0.3), None);
        engine.append_layer(&shadow, leader.id).unwrap();

        // Follower: the preview. Same size as the leader, no children of its own.
        let mirror = engine.new_layer();
        mirror.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        mirror.set_size(Size::points(800.0, 600.0), None);
        mirror.set_position((1000.0, 1000.0), None);
        mirror.set_draw_content(leader.as_content());
        mirror.set_picture_cached(false);
        leader.add_follower_node(&mirror);
        engine.add_layer(&mirror).unwrap();

        engine.update(0.016);
        engine.update(0.016);

        let leader_extent = leader.render_layer().bounds_with_children;
        assert!(
            leader_extent.left() < 0.0 && leader_extent.top() < 0.0,
            "the leader subtree must reach outside its own box: {:?}",
            leader_extent
        );

        engine.clear_damage();
        let painted_before = mirror
            .render_layer()
            .global_transformed_bounds_with_children;
        mirror.set_position((1400.0, 1000.0), None);
        engine.update(0.016);
        let painted_after = mirror
            .render_layer()
            .global_transformed_bounds_with_children;

        assert_eq!(
            mirror.render_layer().bounds_with_children,
            leader_extent,
            "the preview must carry the leader's subtree extent"
        );
        assert!(
            painted_before.width() > mirror.render_layer().bounds.width(),
            "the preview's painted rect should be wider than its own box: {:?}",
            painted_before
        );

        let damage = engine.damage();
        assert!(
            damage.contains(painted_before),
            "damage {:?} misses what the preview painted at the old position {:?}",
            damage,
            painted_before
        );
        assert!(
            damage.contains(painted_after),
            "damage {:?} misses what the preview paints at the new position {:?}",
            damage,
            painted_after
        );
    }

    /// The exposé drag: the preview is scaled down as it moves toward the
    /// workspace row, several steps per second. Each step has to damage what
    /// the preview covered at the PREVIOUS, larger scale — a shrink damaged
    /// only at its new size leaves a ring of the old shadow on screen.
    #[test]
    pub fn mirror_layer_scaled_down_damages_the_previous_size() {
        let engine = Engine::create(4000.0, 4000.0);

        let root = engine.new_layer();
        root.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        root.set_size(Size::points(4000.0, 4000.0), None);
        engine.add_layer(&root).unwrap();

        let leader = engine.new_layer();
        leader.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        leader.set_size(Size::points(800.0, 600.0), None);
        engine.append_layer(&leader, root.id).unwrap();

        let shadow = engine.new_layer();
        shadow.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        shadow.set_position((-100.0, -100.0), None);
        shadow.set_size(Size::points(1000.0, 800.0), None);
        shadow.set_background_color(Color::new_rgba(0.0, 0.0, 0.0, 0.3), None);
        engine.append_layer(&shadow, leader.id).unwrap();

        let mirror = engine.new_layer();
        mirror.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        mirror.set_size(Size::points(800.0, 600.0), None);
        mirror.set_position((1500.0, 1500.0), None);
        mirror.set_anchor_point((0.5, 0.5), None);
        mirror.set_draw_content(leader.as_content());
        mirror.set_picture_cached(false);
        leader.add_follower_node(&mirror);
        engine.append_layer(&mirror, root.id).unwrap();

        engine.update(0.016);
        engine.update(0.016);

        // Drag upward while shrinking, the way `update_drag_scale` ramps the
        // preview down toward the workspace-selector scale.
        let steps = [(1400.0, 0.75), (1300.0, 0.5), (1200.0, 0.3), (1100.0, 0.2)];
        for (y, scale) in steps {
            let painted_before = mirror
                .render_layer()
                .global_transformed_bounds_with_children;
            assert!(
                painted_before.width() > mirror.render_layer().global_transformed_bounds.width(),
                "the preview must know it paints the mirrored shadow outside its box: {painted_before:?} vs {:?}",
                mirror.render_layer().global_transformed_bounds
            );

            engine.clear_damage();
            mirror.set_position((1500.0, y), None);
            mirror.set_scale((scale, scale), None);
            engine.update(0.016);

            let painted_after = mirror
                .render_layer()
                .global_transformed_bounds_with_children;
            assert!(
                painted_before.width() > painted_after.width(),
                "step {scale} should be a shrink: {painted_before:?} -> {painted_after:?}"
            );

            let damage = engine.damage();
            assert!(
                damage.contains(painted_before),
                "damage {damage:?} misses what the preview covered at the previous scale {painted_before:?}"
            );
            assert!(
                damage.contains(painted_after),
                "damage {damage:?} misses what the preview covers now {painted_after:?}"
            );
        }
    }
}
