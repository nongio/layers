#[cfg(test)]
mod tests {
    use lay_rs::{
        drawing::draw_layer,
        prelude::*,
        renderer::skia_image::SkiaImageRenderer,
        skia,
        types::{Color, PaintColor, Point, Size},
        view::{BuildLayerTree, LayerTreeBuilder},
    };

    #[test]
    pub fn damage_render_layer() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        let node = engine.add_layer(layer.clone());

        engine.update(0.016);

        let scene_node = engine.scene_get_node(&node).unwrap();
        let scene_node = scene_node.get();
        let render_layer = scene_node.render_layer();
        let renderer = SkiaImageRenderer::new(1000, 1000, "damage.png");
        let mut surface = renderer.surface();
        let canvas = surface.canvas();
        engine.scene().with_arena(|arena| {
            let damage = draw_layer(canvas, &render_layer, 1.0, arena);

            // test empty layer
            assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
        });

        // test layer with background damage
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);
        engine.update(0.016);
        let render_layer = scene_node.render_layer();
        engine.scene().with_arena(|arena| {
            let damage = draw_layer(canvas, &render_layer, 1.0, arena);
            assert_eq!(damage, skia_safe::Rect::from_xywh(0.0, 0.0, 100.0, 100.0));
        });

        // test layer with border damage
        layer.set_border_color(Color::new_hex("#ff0000ff"), None);
        layer.set_border_width(10.0, None);
        engine.update(0.016);
        let render_layer = scene_node.render_layer();
        engine.scene().with_arena(|arena| {
            let damage = draw_layer(canvas, &render_layer, 1.0, arena);
            assert_eq!(damage, skia_safe::Rect::from_xywh(-5.0, -5.0, 110.0, 110.0));
        });

        // test layer with shadow
        layer.set_shadow_color(Color::new_hex("#ff0000ff"), None);
        layer.set_shadow_offset((-10.0, -10.0), None);
        layer.set_shadow_radius(20.0, None);
        layer.set_shadow_spread(20.0, None);
        engine.update(0.016);
        let render_layer = scene_node.render_layer();
        engine.scene().with_arena(|arena| {
            let damage = draw_layer(canvas, &render_layer, 1.0, arena);
            assert_eq!(
                damage,
                skia_safe::Rect::from_xywh(-50.0, -50.0, 180.0, 180.0)
            );
        });

        // test layer with blend blur
        layer.set_blend_mode(lay_rs::types::BlendMode::BackgroundBlur);

        engine.update(0.016);
        let render_layer = scene_node.render_layer();
        engine.scene().with_arena(|arena| {
            let damage = draw_layer(canvas, &render_layer, 1.0, arena);

            assert_eq!(
                damage,
                skia_safe::Rect::from_xywh(-75.0, -75.0, 230.0, 230.0)
            );
        })
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
        engine.add_layer(layer.clone());
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
        engine.add_layer(layer.clone());
        engine.update(0.016);
        let scene_damage = engine.damage();
        // adding an empty layer should not damage the content
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

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
        engine.add_layer(layer.clone());

        let layer2 = engine.new_layer();
        layer2.set_position((100.0, 100.0), None);
        layer2.set_size(Size::points(100.0, 100.0), None);
        engine.append_layer(layer2.clone(), layer.id);

        engine.update(0.016);
        let scene_damage = engine.damage();
        // adding an empty layer should not damage the content
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0)
        };
        layer2.set_draw_content(draw_func);
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
    pub fn damage_empty() {
        let engine = Engine::create(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.add_layer(layer.clone());

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
        engine.add_layer(layer.clone());

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
        engine.add_layer(layer.clone());

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
    pub fn damage_opacity() {
        let engine = Engine::create(1000.0, 1000.0);
        let wrap = engine.new_layer();
        wrap.set_size(Size::percent(1.0, 1.0), None);
        engine.add_layer(wrap.clone());
        let layer = engine.new_layer();
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);
        layer.set_opacity(0.0, None);
        engine.append_layer(layer.clone(), wrap.id);

        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer with opacity 0 should not damage the scene
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
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
    pub fn damage_parent_opacity() {
        let engine = Engine::create(1000.0, 1000.0);
        let wrap = engine.new_layer();
        wrap.set_size(Size::percent(1.0, 1.0), None);
        engine.add_layer(wrap.clone());

        let layer = engine.new_layer();
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);
        wrap.set_opacity(0.0, None);

        engine.append_layer(layer.clone(), wrap.id);

        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer with a parent with opacity 0 should not damage the scene
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

        engine.clear_damage();
        layer.set_opacity(0.1, None);
        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer with opacity 0.1 and parent 0.0 should not damage the scene
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

        engine.clear_damage();
        layer.set_blend_mode(lay_rs::types::BlendMode::BackgroundBlur);
        wrap.set_opacity(1.0, None);

        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer fading in with a blend mode should damage the scene
        // the damage is bigger because of the blend mode (outset: 25.0)
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(75.0, 75.0, 150.0, 150.0)
        );
    }

    #[test]
    pub fn damage_parent_offset() {
        let engine = Engine::create(1000.0, 1000.0);

        let wrap = engine.new_layer();
        wrap.set_position((100.0, 100.0), None);
        wrap.set_size(Size::points(0.0, 0.0), None);

        engine.add_layer(wrap.clone());

        let layer = engine.new_layer();
        layer.set_layout_style(lay_rs::taffy::Style {
            position: lay_rs::taffy::Position::Absolute,
            ..Default::default()
        });
        layer.set_position((-50.0, -50.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(Color::new_hex("#ff0000ff"), None);

        engine.append_layer(layer.clone(), wrap.id);

        engine.update(0.016);
        let scene_damage = engine.damage();

        // a layer with a parent with opacity 0 should not damage the scene
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(50.0, 50.0, 100.0, 100.0)
        );
    }

    #[test]
    pub fn damage_parent_parent() {
        let w = 500.0;
        let h = 500.0;
        const CHILD_OUTSET: f32 = 100.0;

        let engine = Engine::create(1000.0, 1000.0);
        let wrap = engine.new_layer();
        let layer = engine.new_layer();

        wrap.set_position((200.0, 200.0), None);
        layer.set_size(Size::points(0.0, 0.0), None);

        engine.add_layer(wrap.clone());
        engine.append_layer(layer.clone(), wrap.id);

        let draw_shadow = move |_: &lay_rs::skia::Canvas, w: f32, h: f32| {
            lay_rs::skia::Rect::from_xywh(0.0, 0.0, w, h)
        };
        let tree = LayerTreeBuilder::default()
            .key("a")
            .layout_style(taffy::Style {
                position: taffy::Position::Absolute,
                ..Default::default()
            })
            .position(Point { x: 0.0, y: 0.0 })
            .size(Size::points(w, h))
            .children(vec![LayerTreeBuilder::default()
                .key("b")
                .layout_style(taffy::Style {
                    position: taffy::Position::Absolute,
                    ..Default::default()
                })
                .position((
                    Point {
                        x: -CHILD_OUTSET,
                        y: -CHILD_OUTSET,
                    },
                    None,
                ))
                .size(Size::points(w + CHILD_OUTSET * 2.0, h + CHILD_OUTSET * 2.0))
                .content(Some(draw_shadow))
                // .image_cache(true)
                .build()
                .unwrap()])
            .build()
            .unwrap();

        layer.build_layer_tree(&tree);

        engine.update(0.016);
        let damage = engine.damage();
        println!(
            "{},{} {}x{}",
            damage.x(),
            damage.y(),
            damage.width(),
            damage.height()
        );
        assert_eq!(damage, skia::Rect::from_xywh(100.0, 100.0, 700.0, 700.0));

        engine.clear_damage();

        layer.build_layer_tree(&tree);
        engine.update(0.016);
        let damage = engine.damage();
        println!(
            "{},{} {}x{}",
            damage.x(),
            damage.y(),
            damage.width(),
            damage.height()
        );
    }
}
