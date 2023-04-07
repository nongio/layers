use crate::engine::node::SceneNode;
use crate::engine::Scene;
use crate::types::Rectangle;

use indextree::{Arena, NodeId};

use skia_safe::{Canvas, Color4f, ColorSpace, Image, Matrix, Paint, Rect};
use skia_safe::{Picture, PictureRecorder};

use super::NodeRef;

/// A trait for objects that can be drawn to a canvas.
pub trait Drawable {
    /// Draws the entity on the canvas.
    fn draw(&self, canvas: &mut Canvas);
    /// Returns the area that this drawable occupies.
    fn bounds(&self) -> Rectangle;
    fn scaled_bounds(&self) -> Rectangle;
    fn transform(&self) -> Matrix;
    fn scale(&self) -> (f32, f32);
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
            Rect::from_xywh(-50.0, -50.0, r.width + 100.0, r.height + 100.0),
            None,
        );
        self.draw(canvas);
        recorder.finish_recording_as_picture(None)
    }
}

pub fn draw_single_scene_node(canvas: &mut Canvas, node: &SceneNode) {
    let draw_cache = node.draw_cache.read().unwrap();
    if let Some(draw_cache) = &*draw_cache {
        let transform = node.model.transform();
        canvas.concat(&transform);
        canvas.draw_picture(draw_cache.picture(), None, None);
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

pub fn render_node(node: &SceneNode, canvas: &mut skia_safe::Canvas) {
    let draw_cache = node.draw_cache.read().unwrap();
    let matrix = node.transformation.read().unwrap();

    if let Some(draw_cache) = &*draw_cache {
        canvas.draw_picture(draw_cache.picture(), Some(&matrix), None);
    }
}

pub fn render_node_to_image(node: &SceneNode) -> Option<Image> {
    let draw_cache = node.draw_cache.read().unwrap();
    // let matrix = node.transformation.read().unwrap().clone();
    let mut image = None;
    if let Some(draw_cache) = &*draw_cache {
        let picture = draw_cache.picture();

        let mut p = Paint::default();
        let (sx, sy) = *node.scale.read().unwrap();

        let mut m = skia_safe::Matrix::scale((sx, sy));
        m.set_translate_x(0.0);
        m.set_translate_y(0.0);
        p.set_anti_alias(false);

        let img = Image::from_picture(
            picture,
            (
                node.model.scaled_bounds().width as i32 + 200,
                node.model.scaled_bounds().height as i32 + 200,
            ),
            Some(&m),
            Some(&p),
            skia_safe::image::BitDepth::F16,
            ColorSpace::new_srgb(),
        );
        let img = img.unwrap();
        // println!("img id: {}", img.unique_id());
        let img = img;
        image = Some(img)
    }
    image
}

pub fn render_node_children(
    node_id: NodeRef,
    arena: &Arena<SceneNode>,
    canvas: &mut skia_safe::Canvas,
) {
    let node_id = node_id.into();
    let node = arena.get(node_id).unwrap().get();
    let sc = canvas.save();
    let matrix = *node.transformation.read().unwrap();
    canvas.concat(&matrix);
    node_id.children(arena).for_each(|child_id| {
        if let Some(child) = arena.get(child_id) {
            render_node(child.get(), canvas);
        }
    });
    canvas.restore_to_count(sc);
}
