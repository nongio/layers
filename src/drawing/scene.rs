use skia_safe::Canvas;

use crate::engine::{node::render_node, scene::Scene};

pub fn draw_scene(canvas: &mut Canvas, scene: &Scene) {
    let nodes = scene.nodes.data();
    let nodes = nodes.read().unwrap();
    nodes.iter().for_each(|node| {
        let scene_node = node.get();
        render_node(scene_node, canvas);
    });
}
