#[cfg(test)]
mod tests {
    use layers::{
        engine::LayersEngine,
        types::{Color, PaintColor, Size},
    };

    #[test]
    pub fn damage_rect() {
        let engine = LayersEngine::new(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.scene_add_layer(layer.clone());

        engine.update(0.016);

        let scene_damage = engine.damage();

        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 100.0, 100.0)
        );
    }

    #[test]
    pub fn damage_content() {
        let engine = LayersEngine::new(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.scene_add_layer(layer.clone());
        engine.update(0.016);
        let scene_damage = engine.damage();
        // adding an empty layer should not damage the content
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0)
        };
        layer.set_draw_content(Some(draw_func));
        engine.update(0.016);
        let scene_damage = engine.damage();

        // changing the draw function should damage the content
        assert_eq!(
            scene_damage,
            skia_safe::Rect::from_xywh(100.0, 100.0, 10.0, 10.0)
        );
    }

    #[test]
    pub fn damage_empty() {
        let engine = LayersEngine::new(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        engine.scene_add_layer(layer.clone());

        engine.update(0.016);
        let scene_damage = engine.damage();
        assert_eq!(scene_damage, skia_safe::Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    pub fn damage_rect_content() {
        let engine = LayersEngine::new(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.scene_add_layer(layer.clone());

        let draw_func = |_c: &skia_safe::Canvas, _w: f32, _h: f32| -> skia_safe::Rect {
            skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0)
        };
        layer.set_draw_content(Some(draw_func));
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
        let engine = LayersEngine::new(1000.0, 1000.0);
        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#ff0000ff"),
            },
            None,
        );
        engine.scene_add_layer(layer.clone());

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
}
