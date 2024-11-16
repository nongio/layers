#[cfg(test)]
mod tests {
    use lay_rs::{
        drawing::{node_tree_list, node_tree_list_visible},
        engine::LayersEngine,
        types::*,
    };
    #[test]
    pub fn render_list() {
        let engine = LayersEngine::new(1000.0, 1000.0);

        let layer = engine.new_layer();
        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        engine.scene_add_layer(layer.clone());

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            assert_eq!(nodes.len(), 3);
        });
    }

    #[test]
    pub fn render_list_occluded() {
        let engine = LayersEngine::new(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((100.0, 100.0), None);
        layer.set_size(Size::points(150.0, 150.0), None);

        engine.scene_add_layer(layer.clone());

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            let nodes = node_tree_list_visible(nodes.iter(), arena);

            assert_eq!(nodes.len(), 2);
        });
    }
    #[test]
    pub fn render_list_opacity() {
        let engine = LayersEngine::new(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_opacity(0.9, None);
        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(150.0, 150.0), None);
        layer.set_blend_mode(lay_rs::prelude::BlendMode::BackgroundBlur);
        engine.scene_add_layer(layer.clone());

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            let nodes = node_tree_list_visible(nodes.iter(), arena);

            assert_eq!(nodes.len(), 3);
        });
    }
    #[test]
    pub fn render_list_children() {
        let engine = LayersEngine::new(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(50.0, 50.0), None);
        layer.set_opacity(1.0, None);
        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(150.0, 150.0), None);
        layer.set_opacity(0.9, None);
        engine.scene_add_layer(layer.clone());

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            let nodes = node_tree_list_visible(nodes.iter(), arena);

            assert_eq!(nodes.len(), 3);
        });
    }
    #[test]
    pub fn render_list_hidden() {
        let engine = LayersEngine::new(1000.0, 1000.0);

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);

        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(100.0, 100.0), None);
        layer.set_opacity(0.0, None);
        engine.scene_add_layer(layer.clone());

        let layer = engine.new_layer();
        layer.set_position((0.0, 0.0), None);
        layer.set_size(Size::points(150.0, 150.0), None);
        layer.set_blend_mode(lay_rs::prelude::BlendMode::BackgroundBlur);
        layer.set_hidden(true);
        engine.scene_add_layer(layer.clone());

        engine.update(0.016);

        engine.scene().with_arena(|arena| {
            let nodes = node_tree_list(engine.scene_root().unwrap(), arena, 1.0);
            let nodes = node_tree_list_visible(nodes.iter(), arena);

            assert_eq!(nodes.len(), 1);
        });
    }
}
