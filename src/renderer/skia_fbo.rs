use indextree::{Arena, NodeId};
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    ColorType, Image, Rect, Surface,
};
use std::cell::Cell;

use crate::engine::{
    node::{DrawCacheManagement, RenderableFlags, SceneNode},
    rendering::render_node,
    scene::Scene,
    NodeRef,
};
use crate::{drawing::scene::DrawScene, engine::storage::FlatStorage};

pub struct SkiaFboRenderer {
    pub gr_context: skia_safe::gpu::DirectContext,
    pub surface: Surface,
    pub raster_cache: FlatStorage<Image>,
}
impl SkiaFboRenderer {
    pub fn new(
        width: i32,
        height: i32,
        sample_count: usize,
        stencil_bits: usize,
        fboid: usize,
    ) -> Self {
        let fb_info = {
            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
            }
        };
        let backend_render_target =
            BackendRenderTarget::new_gl((width, height), sample_count, stencil_bits, fb_info);

        let mut gr_context: skia_safe::gpu::DirectContext =
            skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();

        let surface = Surface::from_backend_render_target(
            &mut gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();

        let raster_cache: FlatStorage<Image> = FlatStorage::new();
        Self {
            gr_context,
            surface,
            raster_cache,
        }
    }

    pub fn create(
        width: i32,
        height: i32,
        sample_count: usize,
        stencil_bits: usize,
        fboid: usize,
    ) -> Cell<Self> {
        Cell::new(Self::new(width, height, sample_count, stencil_bits, fboid))
    }

    pub fn surface(&self) -> Surface {
        self.surface.clone()
    }
    fn draw_node_children(
        &self,
        node_id: NodeRef,
        arena: &Arena<SceneNode>,
        canvas: &mut skia_safe::Canvas,
    ) {
        let node_id: NodeId = node_id.into();
        let layer = arena.get(node_id).unwrap().get();
        let bounds = layer.model.bounds();
        let bounds = Rect::from_xywh(
            0.0, //bounds.x as f32,
            0.0, //bounds.y as f32,
            bounds.width as f32,
            bounds.height as f32,
        );

        canvas.clip_rect(bounds, None, None);

        node_id.children(arena).for_each(|child_id| {
            let childindex: usize = child_id.into();
            let node = arena.get(child_id).unwrap().get();

            let flags = node.flags.read().unwrap();

            // FIXME we can't raster because shadows are being cropped
            let can_raster = !flags.contains(RenderableFlags::ANIMATING);

            drop(flags);
            // TODO find a logic to decide when to raster

            // node rastering should me moved to a separate thread
            // if node.need_raster() && can_raster {
            //     let img = render_node_to_image(node);
            //     if let Some(img) = img {
            //         self.raster_cache.insert_with_id(img.clone(), childindex);
            //         node.set_need_raster(false);
            //     }
            //     //     // println!("rastering child: {}", childindex);
            // }

            let matrix = node.transformation.read().unwrap();

            let s = canvas.save();
            canvas.concat(&matrix);
            let mut cached = false;

            if can_raster {
                if let Some(image) = self.raster_cache.get(&childindex) {
                    canvas.draw_image(&image, (0, 0), None);
                    // println!(
                    //     "using cache for child: {} animating {}",
                    //     childindex, !can_raster
                    // );
                    cached = true;
                }
            }
            if !cached {
                // println!(
                //     "no raster for child: {} animating {}",
                //     childindex, !can_raster
                // );
                // node.set_need_raster(true);
                let draw_cache = node.draw_cache.read().unwrap();
                if let Some(draw_cache) = &*draw_cache {
                    canvas.draw_picture(draw_cache.picture(), None, None);
                } else {
                    node.set_need_repaint(true);
                    println!("no picture for child: {}", childindex);
                }
            }
            self.draw_node_children(NodeRef(child_id), arena, canvas);
            canvas.restore_to_count(s);
        });
    }
}

impl DrawScene for SkiaFboRenderer {
    fn draw_scene(&self, scene: &Scene, root_id: NodeRef) {
        let mut surface = self.surface();
        let canvas = surface.canvas();

        let arena = scene.nodes.data();
        let arena = &*arena.read().unwrap();
        if let Some(_root) = scene.get_node(root_id) {
            let root = arena.get(root_id.into()).unwrap().get();
            render_node(root, canvas);
            let matrix = root.transformation.read().unwrap();
            let sc = canvas.save();
            canvas.concat(&matrix);

            self.draw_node_children(root_id, arena, canvas);
            canvas.restore_to_count(sc);
        }

        surface.flush_and_submit();
    }
}
