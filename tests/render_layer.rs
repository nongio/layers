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
}
