#[cfg(test)]
mod tests {
    use layers::engine::LayersEngine;

    #[test]
    pub fn layer_opacity() {
        let engine = LayersEngine::new(1000.0, 1000.0);

        let layer = engine.new_layer();
        let node = engine.scene_add_layer(layer.clone());

        layer.set_opacity(0.0, None);
        engine.update(0.016);
        let scene_node = engine.scene_get_node(node).unwrap();
        let scene_node = scene_node.get();
        let render_layer = scene_node.render_layer();

        assert_eq!(render_layer.opacity, 0.0);

        layer.set_opacity(0.5, None);
        engine.update(0.016);

        let render_layer = scene_node.render_layer();

        assert_eq!(render_layer.opacity, 0.5);

        layer.set_opacity(1.0, None);
        engine.update(0.016);

        let render_layer = scene_node.render_layer();

        assert_eq!(render_layer.opacity, 1.0);
    }
    #[test]
    pub fn layer_parent_opacity() {
        let engine = LayersEngine::new(1000.0, 1000.0);
        let wrap = engine.new_layer();

        wrap.set_opacity(0.0, None);

        engine.scene_add_layer(wrap.clone());

        let layer = engine.new_layer();

        let node = engine.scene_add_layer_to(layer.clone(), wrap.clone());

        engine.update(0.016);
        let scene_node = engine.scene_get_node(node).unwrap();
        let scene_node = scene_node.get();
        let render_layer = scene_node.render_layer();

        assert_eq!(render_layer.opacity, 0.0);
    }
}
