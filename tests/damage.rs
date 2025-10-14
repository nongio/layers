#[cfg(test)]
mod tests {
    use lay_rs::{
        drawing::draw_layer,
        prelude::*,
        renderer::skia_image::SkiaImageRenderer,
        types::{Color, PaintColor, Size},
    };

    #[test]
    pub fn damage_render_layer_transparent() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.add_layer(&layer);

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
        engine.append_layer(&layer.id, None);

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

        engine.add_layer(&layer);

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

        engine.add_layer(&layer.id);

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

        engine.add_layer(&layer.id);

        engine.update(0.016);

        let scene_damage = engine.damage();
        // test empty layer
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );

        // test layer with blend blur
        layer.set_blend_mode(lay_rs::types::BlendMode::BackgroundBlur);

        engine.update(0.016);

        let scene_damage = engine.damage();

        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(75.0, 75.0, 150.0, 150.0)
        );
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
        engine.add_layer(&layer.id);
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
        engine.add_layer(&layer.id);
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
        engine.add_layer(&layer.id);

        let layer2 = engine.new_layer();
        layer2.set_position((100.0, 100.0), None);
        layer2.set_size(Size::points(100.0, 100.0), None);
        engine.append_layer(&layer2, layer.id);

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
        engine.add_layer(&layer.id);

        let child_a = engine.new_layer();
        child_a.set_position((25.0, 15.0), None);
        child_a.set_size(Size::points(100.0, 100.0), None);
        engine.append_layer(&child_a, layer.id);

        let child_b = engine.new_layer();
        child_b.set_position((10.0, 12.0), None);
        child_b.set_size(Size::points(80.0, 80.0), None);
        engine.append_layer(&child_b, child_a.id);

        let child_c = engine.new_layer();
        child_c.set_position((4.0, 3.0), None);
        child_c.set_size(Size::points(60.0, 60.0), None);
        engine.append_layer(&child_c, child_b.id);

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
        engine.add_layer(&layer.id);

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
        engine.add_layer(&layer.id);

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
        engine.add_layer(&layer.id);

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
        engine.add_layer(&parent.id);

        let child = engine.new_layer();
        child.set_position((0.0, 0.0), None);
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.append_layer(&child, parent.id);

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
    pub fn damage_opacity() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        engine.add_layer(&layer);
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
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
        layer.set_blend_mode(lay_rs::types::BlendMode::BackgroundBlur);
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
        engine.add_layer(&layer);
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
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
        engine.add_layer(&layer);
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
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
        engine.add_layer(&layer);
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
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
        engine.add_layer(&root);
        root.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
            ..Default::default()
        });
        root.set_position((0.0, 0.0), None);
        root.set_size(Size::points(400.0, 400.0), None);

        let child = engine.new_layer();
        child.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
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

        engine.append_layer(&child, root.id);

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

        engine.add_layer(&wrap);

        let layer = engine.new_layer();
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((-50.0, -50.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);

        engine.append_layer(&layer, wrap.id);

        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer with a parent with opacity 0 should not damage the scene
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(50.0, 50.0, 100.0, 100.0)
        );
    }
}
