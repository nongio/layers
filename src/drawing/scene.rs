use skia_safe::Canvas;

use crate::engine::{
    node::{render_node, render_node_children},
    scene::Scene,
};

pub trait DrawScene {
    fn draw_scene(&mut self, scene: &Scene);
}

pub(crate) fn draw_scene(canvas: &mut Canvas, scene: &Scene) {
    let root_id = *scene.root.read().unwrap();
    let arena = scene.nodes.data();
    let arena = arena.read().unwrap();
    if let Some(root) = scene.get_node(root_id) {
        render_node(root.get(), canvas);
        render_node_children(root_id, &arena, canvas);
    }
}
