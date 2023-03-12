use skia_safe::Canvas;

use crate::engine::{
    rendering::{render_node, render_node_children},
    scene::Scene,
    NodeRef,
};

pub trait DrawScene {
    fn draw_scene(&self, scene: &Scene, root_id: NodeRef);
}

pub(crate) fn draw_scene(canvas: &mut Canvas, scene: &Scene, root_id: NodeRef) {
    let arena = scene.nodes.data();
    let arena = arena.read().unwrap();
    if let Some(root) = scene.get_node(root_id) {
        render_node(root.get(), canvas);
        render_node_children(root_id, &arena, canvas);
    }
}
