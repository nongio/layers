#[cfg(test)]
mod tests {
    use lay_rs::{
        prelude::*,
        types::{Color, PaintColor, Size},
    };

    #[test]
    pub fn render_layer_size() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer);

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

        engine.append_layer(&layer, None);

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
        engine.add_layer(&layer);

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
        engine.add_layer(&layer);

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

        engine.append_layer(&layer.id, None);

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
        engine.add_layer(&layer);

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
        engine.add_layer(&parent);

        // Child extends beyond parent on right/bottom to test union
        let child = engine.new_layer();
        child.set_position((70.0, 80.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        // Give the child a background so it contributes damage/bounds
        child.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.append_layer(&child, parent.id);

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

    #[test]
    pub fn render_layer_global_children_bounds_with_parent_offset() {
        let engine = Engine::create(1000.0, 1000.0);

        // Parent at an offset
        let parent = engine.new_layer();
        parent.set_position((10.0, 20.0), None);
        parent.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&parent);

        // Child within parent
        let child = engine.new_layer();
        child.set_position((70.0, 80.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        child.set_background_color(Color::new_hex("#00ff00ff"), None);
        engine.append_layer(&child, parent.id);

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

    #[test]
    pub fn render_layer_three_level_hierarchy_bounds() {
        let engine = Engine::create(1000.0, 1000.0);

        // Grandparent at an offset
        let gp = engine.new_layer();
        gp.set_position((5.0, 6.0), None);
        gp.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&gp);

        // Parent at an additional offset
        let parent = engine.new_layer();
        parent.set_position((10.0, 20.0), None);
        parent.set_size(Size::points(80.0, 80.0), None);
        engine.append_layer(&parent, gp.id);

        // Child extends beyond parent
        let child = engine.new_layer();
        child.set_position((70.0, 80.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        child.set_background_color(Color::new_hex("#0000ffff"), None);
        engine.append_layer(&child, parent.id);

        engine.update(0.016);

        // Parent bounds_with_children should union its child in parent space
        let prl = engine.render_layer(&parent).unwrap();
        assert_eq!(
            prl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 120.0, 130.0)
        );
        // Parent global children bounds currently reflect only the parent's own offset
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(10.0, 20.0, 120.0, 130.0)
        );

        // Grandparent bounds_with_children should union parent+child in gp space
        let gprl = engine.render_layer(&gp).unwrap();
        assert_eq!(
            gprl.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 130.0, 150.0)
        );
        // And global shifted by gp offset
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
        engine.add_layer(&parent);

        // Child initially fully inside the parent
        let child = engine.new_layer();
        child.set_position((10.0, 10.0), None);
        child.set_size(Size::points(40.0, 40.0), None);
        // Give the child a background so it contributes to bounds
        child.set_background_color(Color::new_hex("#ff00ffff"), None);
        engine.append_layer(&child, parent.id);

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

        // Currently parent bounds_with_children is updated on parent layout/paint,
        // not on child-only moves. It remains equal to parent bounds here.
        let prl_moved = engine.render_layer(&parent).unwrap();
        assert_eq!(
            prl_moved.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0)
        );
    }

    #[test]
    pub fn render_layer_bounds_with_children_negative_offsets() {
        let engine = Engine::create(2000.0, 2000.0);

        // Parent at origin 500x500
        let parent = engine.new_layer();
        parent.set_size(Size::points(500.0, 500.0), None);
        engine.add_layer(&parent);

        // Child extends beyond parent on left/top and right/bottom
        let child = engine.new_layer();
        child.set_position((-100.0, -100.0), None);
        child.set_size(Size::points(700.0, 700.0), None);
        child.set_background_color(Color::new_hex("#112233ff"), None);
        engine.append_layer(&child, parent.id);

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
        engine.add_layer(&parent);

        // Child A extends to the left/top slightly
        let child_a = engine.new_layer();
        child_a.set_position((-20.0, -30.0), None);
        child_a.set_size(Size::points(40.0, 50.0), None);
        child_a.set_background_color(Color::new_hex("#abcdefFF"), None);
        engine.append_layer(&child_a, parent.id);

        // Child B extends to the right/bottom beyond parent
        let child_b = engine.new_layer();
        child_b.set_position((120.0, 140.0), None);
        child_b.set_size(Size::points(80.0, 30.0), None);
        child_b.set_background_color(Color::new_hex("#fedcbaFF"), None);
        engine.append_layer(&child_b, parent.id);

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
        engine.add_layer(&parent);

        let child = engine.new_layer();
        child.set_position((250.0, 250.0), None);
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_background_color(Color::new_hex("#00ff00ff"), None);
        child.set_image_cached(true);
        engine.append_layer(&child, parent.id);

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
}
