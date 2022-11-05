use crate::engine::node::SceneNode;
use crate::engine::Scene;
use crate::types::Rectangle;

use indextree::{Arena, NodeId};

use skia_safe::{Canvas, Color4f, Matrix, Rect};
use skia_safe::{Picture, PictureRecorder};

/// A trait for objects that can be drawn to a canvas.
pub trait Drawable {
    /// Draws the entity on the canvas.
    fn draw(&self, canvas: &mut Canvas);
    /// Returns the area that this drawable occupies.
    fn bounds(&self) -> Rectangle;
    fn transform(&self) -> Matrix;
}

/// A trait for objects that can be drawn to a PictureRecorder.
pub trait DrawToPicture {
    fn draw_to_picture(&self) -> Option<Picture>;
}

impl<T> DrawToPicture for T
where
    T: Drawable,
{
    fn draw_to_picture(&self) -> Option<Picture> {
        let mut recorder = PictureRecorder::new();

        let r = self.bounds();

        let canvas = recorder.begin_recording(
            Rect::from_xywh(0.0, 0.0, r.width as f32, r.height as f32),
            None,
        );
        self.draw(canvas);
        recorder.finish_recording_as_picture(None)
    }
}

pub fn draw_single_scene_node(canvas: &mut Canvas, node: &SceneNode) {
    if let Some(picture) = node.draw_cache.read().unwrap().picture.clone() {
        let transform = node.model.transform();
        canvas.concat(&transform);
        canvas.draw_picture(picture, None, None);
    } else {
        node.model.draw(canvas);
    }
}
pub fn draw_tree_on_canvas(canvas: &mut Canvas, arena: &Arena<SceneNode>, node_id: &NodeId) {
    canvas.save();
    let node = arena.get(*node_id).unwrap().get();
    draw_single_scene_node(canvas, node);
    for child_id in node_id.children(arena) {
        let child = arena.get(child_id).unwrap().get();
        draw_single_scene_node(canvas, child);
    }

    canvas.restore();
}

pub fn draw(canvas: &mut Canvas, _: &Scene) {
    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

    // draw_tree_on_canvas(canvas, &scene.nodes, &scene.root);
}
