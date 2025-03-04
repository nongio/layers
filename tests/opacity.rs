#[cfg(test)]
mod tests {
    use lay_rs::prelude::*;

    #[test]
    pub fn layer_opacity() {
        let engine = Engine::create(1000.0, 1000.0);

        let layer = engine.new_layer();
        engine.add_layer(&layer);

        layer.set_opacity(0.0, None);
        engine.update(0.016);
        let render_layer = engine.render_layer(&layer).unwrap();

        assert_eq!(render_layer.opacity, 0.0);

        layer.set_opacity(0.5, None);
        engine.update(0.016);

        let render_layer = engine.render_layer(&layer).unwrap();

        assert_eq!(render_layer.opacity, 0.5);

        layer.set_opacity(1.0, None);
        engine.update(0.016);

        let render_layer = engine.render_layer(&layer).unwrap();

        assert_eq!(render_layer.opacity, 1.0);
    }
}
