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
        assert_eq!(prl.bounds, skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0));

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

        // Local union equals as before
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
        // Parent global children bounds = union shifted by gp + parent offsets
        assert_eq!(
            prl.global_transformed_bounds_with_children,
            skia_safe::Rect::from_xywh(15.0, 26.0, 120.0, 130.0)
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

        // Now the parent's bounds_with_children should reflect the new child union
        // Parent spans 0..100 in both axes; child at (90,90) size 40x40 -> union 0..130, so 130x130
        let prl_moved = engine.render_layer(&parent).unwrap();
        assert_eq!(
            prl_moved.bounds_with_children,
            skia_safe::Rect::from_xywh(0.0, 0.0, 130.0, 130.0)
        );
    }
}
