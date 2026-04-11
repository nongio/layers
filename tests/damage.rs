#[cfg(test)]
mod tests {
    use layers::{
        drawing::draw_layer,
        prelude::*,
        renderer::skia_image::SkiaImageRenderer,
        types::{Color, PaintColor, Size},
    };
    use skia_safe::Contains;

    #[test]
    pub fn damage_render_layer_transparent() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer).unwrap();

        engine.update(0.016);

        let render_layer = engine.render_layer(&layer.id).unwrap();
        let renderable = engine.renderable(&layer.id).unwrap();

        let renderer = SkiaImageRenderer::new(1000, 1000, "damage.png");
        let mut surface = renderer.surface();
        let canvas = surface.canvas();
        let damage = draw_layer(canvas, &render_layer, 1.0, &renderable);
        // test empty layer
        assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    pub fn damage_render_layer_background() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();

        layer.set_size(Size::points(100.0, 100.0), None);
        engine.append_layer(&layer.id, None).unwrap();

        engine.update(0.016);

        let renderer = SkiaImageRenderer::new(1000, 1000, "damage.png");
        let mut surface = renderer.surface();
        let canvas = surface.canvas();

        // // test layer with background damage
        let _tr = layer.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 1.0), None);

        engine.update(0.016);

        let render_layer = engine.render_layer(&layer.id).unwrap();
        let renderable = engine.renderable(&layer.id).unwrap();
        println!("{:#?}", render_layer);

        let damage = draw_layer(canvas, &render_layer, 1.0, &renderable);
        assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0));
    }

    #[test]
    pub fn damage_render_layer_border() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer).unwrap();

        engine.update(0.016);

        let render_layer = engine.render_layer(&layer.id).unwrap();
        let renderable = engine.renderable(&layer.id).unwrap();

        let renderer = SkiaImageRenderer::new(1000, 1000, "damage.png");
        let mut surface = renderer.surface();
        let canvas = surface.canvas();
        let damage = draw_layer(canvas, &render_layer, 1.0, &renderable);
        // test empty layer
        assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

        layer.set_background_color(Color::new_hex("#ff0000ff"), None);

        // test layer with border damage
        layer.set_border_color(Color::new_hex("#ff0000ff"), None);
        layer.set_border_width(10.0, None);
        engine.update(0.016);

        let render_layer = engine.render_layer(&layer).unwrap();
        let renderable = engine.renderable(&layer).unwrap();
        let damage = draw_layer(canvas, &render_layer, 1.0, &renderable);
        assert_eq!(damage, skia_safe::Rect::from_xywh(-5.0, -5.0, 110.0, 110.0));
    }

    #[test]
    pub fn damage_render_layer_shadow() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer.id).unwrap();

        engine.update(0.016);

        let render_layer = engine.render_layer(&layer).unwrap();
        let renderable = engine.renderable(&layer).unwrap();

        let renderer = SkiaImageRenderer::new(1000, 1000, "damage.png");
        let mut surface = renderer.surface();
        let canvas = surface.canvas();
        let damage = draw_layer(canvas, &render_layer, 1.0, &renderable);
        // test empty layer
        assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

        // test layer with shadow
        layer.set_shadow_color(Color::new_hex("#ff0000ff"), None);
        layer.set_shadow_offset((-10.0, -10.0), None);
        layer.set_shadow_radius(20.0, None);
        layer.set_shadow_spread(20.0, None);
        engine.update(0.016);
        let render_layer = engine.render_layer(&layer).unwrap();
        let renderable = engine.renderable(&layer).unwrap();
        let damage = draw_layer(canvas, &render_layer, 1.0, &renderable);
        assert_eq!(
            damage,
            skia_safe::Rect::from_xywh(-50.0, -50.0, 180.0, 180.0)
        );
    }

    #[test]
    pub fn damage_render_layer_backblur() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();

        layer.set_background_color(Color::new_hex("#ffffffff"), None);
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer.id).unwrap();

        engine.update(0.016);

        let scene_damage = engine.damage();
        // test empty layer
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );

        // test layer with blend blur
        layer.set_blend_mode(layers::types::BlendMode::BackgroundBlur);

        engine.update(0.016);

        let scene_damage = engine.damage();

        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(75.0, 75.0, 150.0, 150.0)
        );
    }

    #[test]
    pub fn backdrop_blur_does_not_bubble_through_hidden_parent() {
        let engine = Engine::create(1000.0, 1000.0);

        let root = engine.new_layer();
        root.set_position((0.0, 0.0), None);
        root.set_size(Size::points(600.0, 600.0), None);
        engine.add_layer(&root.id).unwrap();

        let hidden_parent = engine.new_layer();
        hidden_parent.set_position((100.0, 100.0), None);
        hidden_parent.set_size(Size::points(300.0, 300.0), None);
        hidden_parent.set_hidden(true);
        engine
            .append_layer(&hidden_parent.id, Some(root.id))
            .unwrap();

        let child = engine.new_layer();
        child.set_position((20.0, 20.0), None);
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_blend_mode(layers::types::BlendMode::BackgroundBlur);
        engine
            .append_layer(&child.id, Some(hidden_parent.id))
            .unwrap();

        engine.update(0.016);

        let root_render_layer = engine.render_layer(&root.id).unwrap();
        assert!(root_render_layer.backdrop_blur_region.is_none());
    }

    #[test]
    pub fn damage_rect() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.add_layer(&layer.id).unwrap();
        engine.update(0.016);

        let scene_damage = engine.damage();

        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );
    }

    #[test]
    pub fn damage_content() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&layer.id).unwrap();
        engine.update(0.016);
        let scene_damage = engine.damage();
        // adding an empty layer should not damage the content
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
        // we draw and clear the damage
        engine.clear_damage();

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0)
        };
        layer.set_draw_content(draw_func);
        engine.update(0.016);
        let scene_damage = engine.damage();

        // changing the draw function should damage the content
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 10.0, 10.0)
        );
    }

    #[test]
    pub fn damage_content_nested() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        // layer.set_image_cache(true);
        engine.add_layer(&layer.id).unwrap();

        let layer2 = engine.new_layer();
        layer2.set_position((100.0, 100.0), None);
        layer2.set_size(Size::points(100.0, 100.0), None);
        engine.append_layer(&layer2, layer.id).unwrap();

        engine.update(0.016);
        let scene_damage = engine.damage();
        // adding an empty layer should not damage the content
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0)
        };
        layer2.set_draw_content(draw_func);
        // the problem is with prev_transformed_bounds and new_transformed_bounds
        // it is changing...
        engine.update(0.016);
        let scene_damage = engine.damage();

        // changing the draw function should damage the content
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(200.0, 200.0, 10.0, 10.0)
        );
        engine.clear_damage();

        layer2.set_draw_content(draw_func);
        engine.update(0.016);
        let scene_damage = engine.damage();

        // changing the draw function should damage the content
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(200.0, 200.0, 10.0, 10.0)
        );
    }

    #[test]
    pub fn damage_content_nested_deep() {
        let engine = Engine::create(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((50.0, 70.0), None);
        layer.set_size(Size::points(120.0, 120.0), None);
        engine.add_layer(&layer.id).unwrap();

        let child_a = engine.new_layer();
        child_a.set_position((25.0, 15.0), None);
        child_a.set_size(Size::points(100.0, 100.0), None);
        engine.append_layer(&child_a, layer.id).unwrap();

        let child_b = engine.new_layer();
        child_b.set_position((10.0, 12.0), None);
        child_b.set_size(Size::points(80.0, 80.0), None);
        engine.append_layer(&child_b, child_a.id).unwrap();

        let child_c = engine.new_layer();
        child_c.set_position((4.0, 3.0), None);
        child_c.set_size(Size::points(60.0, 60.0), None);
        engine.append_layer(&child_c, child_b.id).unwrap();

        engine.update(0.016);
        let scene_damage = engine.damage();
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
        engine.clear_damage();

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 12.0, 9.0)
        };
        child_c.set_draw_content(draw_func);

        engine.update(0.016);
        let scene_damage = engine.damage();

        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(89.0, 100.0, 12.0, 9.0)
        );

        engine.clear_damage();
        engine.update(0.016);
        let scene_damage = engine.damage();
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    pub fn damage_empty() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&layer.id).unwrap();

        engine.update(0.016);
        let scene_damage = engine.damage();
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    pub fn damage_rect_content() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.add_layer(&layer.id).unwrap();

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0)
        };
        layer.set_draw_content(draw_func);
        engine.update(0.016);
        let scene_damage = engine.damage();
        // if the layer has a background the damage is the union of the background and the content
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );
    }

    #[test]
    pub fn damage_move_layer() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.add_layer(&layer.id).unwrap();

        engine.update(0.016);
        let scene_damage = engine.damage();

        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );
        engine.clear_damage();

        layer.set_position((200.0, 200.0), None);
        engine.update(0.016);
        let scene_damage = engine.damage();

        // layer moved from 100,100 to 200,200
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 200.0, 200.0)
        );
        engine.clear_damage();

        layer.set_position((300.0, 300.0), None);
        engine.update(0.016);
        let scene_damage = engine.damage();
        // layer moved from 200,200 to 300,300
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(200.0, 200.0, 200.0, 200.0)
        );
    }

    #[test]
    pub fn damage_move_parent_with_visible_child() {
        let engine = Engine::create(1000.0, 1000.0);

        let parent = engine.new_layer();
        parent.set_position((100.0, 100.0), None);
        parent.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&parent.id).unwrap();

        let child = engine.new_layer();
        child.set_position((0.0, 0.0), None);
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);
        let scene_damage = engine.damage();
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );
        engine.clear_damage();

        parent.set_position((200.0, 200.0), None);
        engine.update(0.016);
        let scene_damage = engine.damage();

        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 200.0, 200.0)
        );
    }

    #[test]
    pub fn damage_move_layout_only_parent_passes_children_damage_only() {
        let engine = Engine::create(1000.0, 1000.0);

        let parent = engine.new_layer();
        parent.set_position((100.0, 100.0), None);
        parent.set_size(Size::points(300.0, 300.0), None);
        engine.add_layer(&parent.id).unwrap();

        let child = engine.new_layer();
        child.set_position((100.0, 100.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        child.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);
        engine.clear_damage();

        parent.set_position((200.0, 200.0), None);
        engine.update(0.016);
        let scene_damage = engine.damage();

        // Child moves from (200,200)-(250,250) to (300,300)-(350,350),
        // so damage is the union of those two child rects only.
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(200.0, 200.0, 150.0, 150.0)
        );
    }

    #[test]
    pub fn damage_opacity() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();
        layer.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);
        layer.set_opacity(0.0, None);

        engine.update(0.016);

        let render_layer = engine.render_layer(&layer.id).unwrap();
        let renderable = engine.renderable(&layer.id).unwrap();

        let renderer = SkiaImageRenderer::new(1000, 1000, "damage.png");
        let mut surface = renderer.surface();
        let canvas = surface.canvas();

        let damage = draw_layer(canvas, &render_layer, 1.0, &renderable);

        // a layer with opacity 0 should not damage the scene
        assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
        engine.clear_damage();
        layer.set_opacity(0.1, None);
        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer with opacity 0.1 should damage the scene
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );
        engine.clear_damage();
        layer.set_opacity(0.0, None);

        engine.update(0.016);
        let scene_damage = engine.damage();
        // a layer fading out should damage the scene
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );

        engine.clear_damage();
        layer.set_blend_mode(layers::types::BlendMode::BackgroundBlur);
        layer.set_opacity(0.1, None);

        engine.update(0.016);
        let scene_damage = engine.damage();
        // a layer fading in with a blend mode should damage the scene
        // the damage is bigger because of the blend mode
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(75.0, 75.0, 150.0, 150.0)
        );
    }

    #[test]
    pub fn damage_animation_updates_scene() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();
        layer.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );

        engine.update(0.016);
        engine.clear_damage();

        layer.set_position((300.0, 0.0), Transition::linear(1.0));

        engine.update(0.1);
        let first_damage = engine.damage();
        assert!(
            !first_damage.is_empty(),
            "running animation should damage the scene on first update"
        );

        engine.clear_damage();

        engine.update(0.1);
        let second_damage = engine.damage();
        assert!(
            !second_damage.is_empty(),
            "running animation should continue to damage the scene"
        );
    }

    #[test]
    pub fn damage_opacity_animation() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();
        layer.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        layer.set_opacity(0.0, None);

        engine.update(0.016);
        engine.clear_damage();

        layer.set_opacity(1.0, Transition::linear(1.0));

        for _ in 0..5 {
            engine.update(0.1);
            let damage = engine.damage();

            assert!(
                !damage.is_empty(),
                "opacity animation should damage the scene on first update"
            );
            engine.clear_damage();
        }
    }

    #[test]
    pub fn damage_size_animation() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer).unwrap();
        layer.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );

        engine.update(0.016);
        engine.clear_damage();

        layer.set_size(Size::points(200.0, 200.0), Transition::linear(1.0));

        for _ in 0..5 {
            engine.update(0.1);
            let damage = engine.damage();

            assert!(
                !damage.is_empty(),
                "opacity animation should damage the scene on first update"
            );
            engine.clear_damage();
        }
    }

    #[test]
    pub fn damage_layer_removal() {
        let engine = Engine::create(1000.0, 1000.0);
        let root = engine.new_layer();
        engine.add_layer(&root).unwrap();
        root.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        root.set_position((0.0, 0.0), None);
        root.set_size(Size::points(400.0, 400.0), None);

        let child = engine.new_layer();
        child.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        child.set_position((100.0, 100.0), None);
        child.set_size(Size::points(50.0, 50.0), None);
        child.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#00ff00ff"),
            },
            None,
        );

        engine.append_layer(&child, root.id).unwrap();

        engine.update(0.016);
        engine.clear_damage();

        child.remove();

        engine.update(0.016);
        let damage = engine.damage();
        assert_eq!(
            damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 50.0, 50.0),
            "removing a layer should damage its previous bounds"
        );
    }

    #[test]
    pub fn damage_parent_offset() {
        let engine = Engine::create(1000.0, 1000.0);

        let wrap = engine.new_layer();
        wrap.set_position((100.0, 100.0), None);
        wrap.set_size(Size::points(0.0, 0.0), None);

        engine.add_layer(&wrap).unwrap();

        let layer = engine.new_layer();
        layer.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((-50.0, -50.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);

        engine.append_layer(&layer, wrap.id).unwrap();

        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer with a parent with opacity 0 should not damage the scene
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(50.0, 50.0, 100.0, 100.0)
        );
    }

    #[test]
    pub fn damage_color_filter() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_picture_cached(false);
        // layer.set_image_cached(true);
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.add_layer(&layer.id).unwrap();

        engine.update(0.016);

        let frame_1 = engine
            .scene_get_node(layer.id())
            .unwrap()
            .get()
            .frame_number();
        let damage = engine.damage();
        assert_eq!(
            damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );
        engine.clear_damage();
        engine.update(0.016);
        let frame_1_after = engine
            .scene_get_node(layer.id())
            .unwrap()
            .get()
            .frame_number();
        assert_eq!(
            frame_1, frame_1_after,
            "frame_number should not increase if there are no changes to the layer"
        );
        println!("frame_1: {}, frame_1_after: {}", frame_1, frame_1_after);
        // Add a color filter — frame_number should increase and bounds should be damaged
        let mut cm = skia_safe::ColorMatrix::default();
        cm.set_saturation(0.0);
        let filter = skia_safe::color_filters::matrix(&cm, None);
        layer.set_color_filter(filter);
        engine.update(0.016);

        let frame_2 = engine
            .scene_get_node(layer.id())
            .unwrap()
            .get()
            .frame_number();
        assert!(
            frame_2 > frame_1,
            "frame_number should increase after adding a color filter"
        );
        let damage = engine.damage();
        assert_eq!(
            damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0),
            "adding a color filter should damage the layer bounds"
        );
        engine.clear_damage();

        // No changes — no damage, frame_number stays the same
        engine.update(0.016);
        let frame_3 = engine
            .scene_get_node(layer.id())
            .unwrap()
            .get()
            .frame_number();
        let damage = engine.damage();
        assert_eq!(
            damage,
            skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0),
            "no changes should produce no damage"
        );
        assert_eq!(
            frame_3, frame_2,
            "frame_number should not change when there is nothing to repaint"
        );

        // Remove the filter — frame_number should increase and bounds should be damaged again
        layer.set_color_filter(None);
        engine.update(0.016);

        let frame_4 = engine
            .scene_get_node(layer.id())
            .unwrap()
            .get()
            .frame_number();
        assert!(
            frame_4 > frame_3,
            "frame_number should increase after removing the color filter"
        );
        let damage = engine.damage();
        assert_eq!(
            damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0),
            "removing a color filter should damage the layer bounds"
        );
    }

    /// Tests the bug case: a layer with no visible drawables acting as a
    /// filter container must still bump frame_number and produce damage when
    /// a color filter is added or removed, because the filter visually affects
    /// how its children are drawn.
    #[test]
    pub fn damage_color_filter_container_no_background() {
        let engine = Engine::create(1000.0, 1000.0);

        // Parent: pure filter container — no background, no content
        let container = engine.new_layer();
        container.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        container.set_position((100.0, 100.0), None);
        container.set_size(Size::points(200.0, 200.0), None);
        engine.add_layer(&container.id).unwrap();

        // Child: has background so subtree is visually non-empty
        let child = engine.new_layer();
        child.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        child.set_position((0.0, 0.0), None);
        child.set_size(Size::points(200.0, 200.0), None);
        child.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.append_layer(&child.id, Some(container.id)).unwrap();

        engine.update(0.016);
        engine.clear_damage();

        let frame_before = engine
            .scene_get_node(container.id())
            .unwrap()
            .get()
            .frame_number();

        // Add a color filter to the container (no background on container)
        let mut cm = skia_safe::ColorMatrix::default();
        cm.set_saturation(0.0);
        let filter = skia_safe::color_filters::matrix(&cm, None);
        container.set_color_filter(filter);
        engine.update(0.016);

        let frame_after_add = engine
            .scene_get_node(container.id())
            .unwrap()
            .get()
            .frame_number();
        assert!(
            frame_after_add > frame_before,
            "frame_number must increase when a color filter is added to a filter-container node"
        );
        let damage = engine.damage();
        assert!(
            !damage.is_empty(),
            "adding a color filter to a filter-container must produce scene damage"
        );
        engine.clear_damage();

        // No changes — no damage, frame stays
        engine.update(0.016);
        let frame_stable = engine
            .scene_get_node(container.id())
            .unwrap()
            .get()
            .frame_number();
        assert_eq!(
            frame_stable, frame_after_add,
            "frame_number must not change when nothing changed"
        );
        assert_eq!(engine.damage(), skia_safe::Rect::default());
        engine.clear_damage();

        // Remove the filter
        container.set_color_filter(None);
        engine.update(0.016);

        let frame_after_remove = engine
            .scene_get_node(container.id())
            .unwrap()
            .get()
            .frame_number();
        assert!(
            frame_after_remove > frame_stable,
            "frame_number must increase when the color filter is removed"
        );
        let damage = engine.damage();
        assert!(
            !damage.is_empty(),
            "removing a color filter from a filter-container must produce scene damage"
        );
    }

    /// damage from the leader is correctly transformed to the follower's
    /// global coordinates.
    #[test]
    pub fn damage_follower_as_content_with_scale_and_translate() {
        let engine = Engine::create(1000.0, 1000.0);

        // Create an explicit root to hold both leader and follower as siblings
        let root = engine.new_layer();
        root.set_size(Size::points(1000.0, 1000.0), None);
        engine.add_layer(&root).unwrap();

        // Leader layer A: at position (0,0), size 100x100
        let leader = engine.new_layer();
        leader.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        leader.set_position((0.0, 0.0), None);
        leader.set_size(Size::points(100.0, 100.0), None);
        // Add a draw function to the leader - just draws a background
        leader.set_draw_content(
            |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
                skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0)
            },
        );
        engine.append_layer(&leader, root.id).unwrap();

        // Initial update to establish leader's state
        engine.update(0.016);

        // Follower layer B: manually uses as_content() to replicate leader
        // Position (200,200) with scale 0.5x
        let follower = engine.new_layer();
        follower.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        follower.set_position((200.0, 200.0), None);
        follower.set_size(Size::points(100.0, 100.0), None);
        follower.set_scale((0.5, 0.5), None);

        // Use as_content() - this is what LayerTreeBuilder does with replicate_node
        let leader_content = leader.as_content();
        follower.set_draw_content(leader_content);

        // Register the follower relationship so the follower gets damaged
        // when the leader's content changes (this is what LayerTreeBuilder does)
        leader.add_follower_node(follower.id());

        // Add follower as sibling to leader (not child)
        engine.append_layer(&follower, root.id).unwrap();

        // This update should NOT cause stack overflow anymore
        engine.update(0.016);
        engine.clear_damage();

        // Change the leader - should damage both leader and follower
        leader.set_draw_content(
            |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
                skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0)
            },
        );
        engine.update(0.016);

        let scene_damage = engine.damage();

        // Leader damage: (0, 0) to (100, 100)
        // Follower damage: position (200, 200), size 100x100 scaled 0.5x = (200, 200) to (250, 250)
        // Combined damage should be: (0, 0) to (250, 250)
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(0.0, 0.0, 250.0, 250.0),
            "Expected combined damage from leader (0,0)-(100,100) and follower (200,200)-(250,250)"
        );
    }

    // -------------------------------------------------------------------
    // External damage channel (add_damage / set_damage)
    //
    // These tests pin the contract for the "case 3" path: content that is
    // produced externally (e.g. a Wayland surface texture) and whose damage
    // cannot be expressed by the draw closure return value alone.
    //
    // Shape: closure stays installed; callers report damage via
    //   layer.add_damage(rect)  -- unions into pending
    //   layer.set_damage(rect)  -- replaces pending
    // Both take layer-local coordinates, same space as the closure return.
    // In do_repaint, pending is unioned with the closure return into
    // repaint_damage, then cleared.
    // -------------------------------------------------------------------

    /// Priming a layer's cache with a closure, then reporting external
    /// damage via add_damage, must produce scene damage equal to the union
    /// of the closure return and the pending rect, mapped to global coords.
    #[test]
    pub fn damage_add_damage_union_with_closure() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&layer).unwrap();

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 1.0, 1.0)
        };
        layer.set_draw_content(draw_func);
        engine.update(0.016);
        engine.clear_damage();

        // External damage in layer-local coords.
        layer.add_damage(skia_safe::Rect::from_xywh(5.0, 5.0, 20.0, 20.0));
        engine.update(0.016);

        let scene_damage = engine.damage();
        // closure (0,0,1,1) ∪ pending (5,5,20,20) = (0,0,25,25) local
        // → global offset by (100,100) = (100,100,25,25)
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 25.0, 25.0)
        );
    }

    /// Multiple add_damage calls before a single paint must accumulate
    /// (union), and must be fully cleared after the paint consumes them.
    #[test]
    pub fn damage_add_damage_accumulates_and_clears() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&layer).unwrap();

        // Closure returns empty so only the pending rect contributes to damage.
        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::default()
        };
        layer.set_draw_content(draw_func);
        engine.update(0.016);
        engine.clear_damage();

        // Two rects: (5,5,10,10) and (30,30,5,5). Union = (5,5,30,30) → (5,5 w25 h25)
        layer.add_damage(skia_safe::Rect::from_xywh(5.0, 5.0, 10.0, 10.0));
        layer.add_damage(skia_safe::Rect::from_xywh(30.0, 30.0, 5.0, 5.0));
        engine.update(0.016);

        let scene_damage = engine.damage();
        // union in local = (5,5) to (35,35) = (5,5,30,30)
        // global offset (100,100) = (105,105,30,30)
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(105.0, 105.0, 30.0, 30.0)
        );

        // After paint, pending must be cleared: no further add_damage → no damage.
        engine.clear_damage();
        engine.update(0.016);
        let scene_damage = engine.damage();
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
    }

    /// set_damage replaces pending rather than unioning. add_damage(A)
    /// followed by set_damage(B) must yield B alone (plus the closure
    /// return, which is empty in this test).
    #[test]
    pub fn damage_set_damage_replaces_pending() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(&layer).unwrap();

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::default()
        };
        layer.set_draw_content(draw_func);
        engine.update(0.016);
        engine.clear_damage();

        layer.add_damage(skia_safe::Rect::from_xywh(5.0, 5.0, 50.0, 50.0));
        // set_damage replaces whatever was pending.
        layer.set_damage(skia_safe::Rect::from_xywh(10.0, 10.0, 20.0, 20.0));
        engine.update(0.016);

        let scene_damage = engine.damage();
        // Only (10,10,20,20) in local → (110,110,20,20) global.
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(110.0, 110.0, 20.0, 20.0)
        );
    }

    /// External damage on a leader must propagate to its followers so
    /// mirror regions are redrawn. Mirrors may over-damage (full bounds) —
    /// that is acceptable; the requirement is that the follower's region
    /// ends up in scene damage.
    #[test]
    pub fn damage_add_damage_propagates_to_follower() {
        let engine = Engine::create(1000.0, 1000.0);

        let root = engine.new_layer();
        root.set_size(Size::points(1000.0, 1000.0), None);
        engine.add_layer(&root).unwrap();

        // Leader at (0,0) 100x100 with a draw closure (returns empty).
        let leader = engine.new_layer();
        leader.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        leader.set_position((0.0, 0.0), None);
        leader.set_size(Size::points(100.0, 100.0), None);
        leader.set_draw_content(
            |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
                skia_safe::Rect::default()
            },
        );
        engine.append_layer(&leader, root.id).unwrap();
        engine.update(0.016);

        // Follower at (200,200), same size, mirrors the leader.
        let follower = engine.new_layer();
        follower.set_layout_style(layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        });
        follower.set_position((200.0, 200.0), None);
        follower.set_size(Size::points(100.0, 100.0), None);
        follower.set_draw_content(leader.as_content());
        leader.add_follower_node(follower.id());
        engine.append_layer(&follower, root.id).unwrap();

        engine.update(0.016);
        engine.clear_damage();

        // Report partial damage on the leader.
        leader.add_damage(skia_safe::Rect::from_xywh(10.0, 10.0, 20.0, 20.0));
        engine.update(0.016);

        let scene_damage = engine.damage();

        // Leader's partial rect in global = (10,10,20,20).
        // Follower must contribute damage covering its own region at (200,200).
        // The minimum acceptable result is that scene_damage contains both
        // the leader rect and at least the follower's (200,200) corner.
        assert!(
            scene_damage.contains(skia_safe::Point::new(15.0, 15.0)),
            "leader partial damage missing from scene: {:?}",
            scene_damage
        );
        assert!(
            scene_damage.contains(skia_safe::Point::new(200.0, 200.0))
                && scene_damage.contains(skia_safe::Point::new(299.0, 299.0)),
            "follower mirror region missing from scene: {:?}",
            scene_damage
        );
    }

    // -------------------------------------------------------------------
    // Occlusion-aware damage — `Engine::compute_output_state`
    //
    // Single-pass walker that folds occlusion + per-output scene damage.
    // When a layer is fully occluded by an opaque layer above, its damage
    // must not appear in the returned scene damage region.
    //
    // v1 opaque criteria (hint-driven):
    //   content_opaque == true
    //   && premultiplied_opacity == 1.0
    //   && blend_mode == Normal
    //   && !hidden
    // Occluder shape = the layer's global_transformed_bounds.
    //
    // See `project_occlusion_damage_plan.md` for full design.
    // -------------------------------------------------------------------

    /// A layer fully occluded by an opaque sibling above it must contribute
    /// zero damage to the scene, even when its draw closure reports damage.
    /// This is the starting TDD test — `engine.damage()` is expected to
    /// fold occlusion into its result, so the consumer-visible API stays
    /// the same and the effect is transparent.
    #[test]
    pub fn occluded_layer_damage_is_dropped_from_scene() {
        let engine = Engine::create(1000.0, 1000.0);

        // Back layer — will be fully occluded by `front`.
        let back = engine.new_layer();
        back.set_position((100.0, 100.0), None);
        back.set_size(Size::points(200.0, 200.0), None);
        back.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.add_layer(&back).unwrap();

        // Front layer — same bounds as `back`, marked content_opaque so it
        // acts as an occluder even with a transparent background.
        let front = engine.new_layer();
        front.set_position((100.0, 100.0), None);
        front.set_size(Size::points(200.0, 200.0), None);
        front.set_content_opaque(true);
        engine.add_layer(&front).unwrap();

        // First update to establish initial state; clear any startup damage.
        engine.update(0.016);
        engine.clear_damage();

        // Report damage on `back` via its draw closure. `front` is unchanged.
        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 50.0, 50.0)
        };
        back.set_draw_content(draw_func);
        engine.update(0.016);

        let scene_damage = engine.damage();
        assert!(
            scene_damage.is_empty(),
            "scene damage must be empty — back's damage is fully occluded by front, \
             but got {:?}",
            scene_damage
        );
    }
}
