use indextree::Arena;
use skia_safe::Canvas;

use crate::engine::{node::SceneNode, scene::Scene, NodeRef};

pub trait DrawScene {
    fn draw_scene(&self, scene: &Scene, root_id: NodeRef);
}

pub fn draw_scene(canvas: &mut Canvas, scene: &Scene, root_id: NodeRef) {
    let arena = scene.nodes.data();
    let arena = arena.read().unwrap();
    if let Some(root) = scene.get_node(root_id) {
        render_node(root.get(), canvas);
        render_node_children(root_id, &arena, canvas);
    }
}

pub fn render_node(node: &SceneNode, canvas: &mut skia_safe::Canvas) {
    let draw_cache = node.draw_cache.read().unwrap();
    let matrix = node.render_layer.read().unwrap().transform;
    // let opacity = node.model.opacity();
    // let blend_mode = node.model.blend_mode();
    // let bounds = node.model.bounds();
    // let bounds = skia_safe::Rect::from_xywh(
    //     0.0, //bounds.x as f32,
    //     0.0, //bounds.y as f32,
    //     bounds.width,
    //     bounds.height,
    // );
    if let Some(draw_cache) = &*draw_cache {
        // let mut paint = Paint::default();

        let restore_to = canvas.save();
        // if blend_mode == BlendMode::BackgroundBlur && opacity > 0.1 {
        //     let mut save_layer_rec = skia_safe::canvas::SaveLayerRec::default();
        //     let blur = skia_safe::image_filters::blur(
        //         (50.0, 50.0),
        //         skia_safe::TileMode::Clamp,
        //         None,
        //         Some(skia_safe::image_filters::CropRect::from(bounds)),
        //     )
        //     .unwrap();
        //     save_layer_rec = save_layer_rec.backdrop(&blur).bounds(&bounds);
        //     canvas.save_layer(&save_layer_rec);
        // }

        // paint.set_alpha_f(opacity);
        canvas.draw_picture(draw_cache.picture(), Some(&matrix), None);
        canvas.restore_to_count(restore_to);
    }
}

pub fn render_node_children(
    node_id: NodeRef,
    arena: &Arena<SceneNode>,
    canvas: &mut skia_safe::Canvas,
) {
    let node_id = node_id.into();
    let node = arena.get(node_id).unwrap().get();
    let sc = canvas.save();
    let matrix = &node.render_layer.read().unwrap().transform;
    canvas.concat(matrix);
    node_id.children(arena).for_each(|child_id| {
        if let Some(child) = arena.get(child_id) {
            render_node(child.get(), canvas);
        }
    });
    canvas.restore_to_count(sc);
}
