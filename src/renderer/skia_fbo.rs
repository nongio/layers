#![allow(warnings)]

use indextree::{Arena, NodeId};
use skia_safe::{
    // canvas::SaveLayerRec,
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    // image_filters::{blur, CropRect},
    ColorType,
    Rect,
    Surface,
};
use std::cell::Cell;

use crate::{
    drawing::scene::{render_node, DrawScene},
    layers::layer::render_layer,
};
use crate::{
    engine::{
        node::{DrawCacheManagement, SceneNode},
        scene::Scene,
        NodeRef,
    },
    prelude::Drawable,
};

#[derive(Clone)]
pub struct SkiaFboRenderer {
    pub gr_context: skia_safe::gpu::DirectContext,
    pub surface: Surface,
    // pub raster_cache: FlatStorage<Image>,
}
impl SkiaFboRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: impl Into<i32>,
        height: impl Into<i32>,
        sample_count: impl Into<usize>,
        stencil_bits: impl Into<usize>,
        fboid: impl Into<u32>,
        color_type: ColorType,
        surface_origin: SurfaceOrigin,
        context: Option<&skia_safe::gpu::DirectContext>,
    ) -> Self {
        let fb_info = {
            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
            }
        };
        let backend_render_target = BackendRenderTarget::new_gl(
            (width.into(), height.into()),
            sample_count.into(),
            stencil_bits.into(),
            fb_info,
        );

        let mut gr_context: skia_safe::gpu::DirectContext = if let Some(context) = context {
            context.clone()
        } else {
            skia_safe::gpu::DirectContext::new_gl(None, None).unwrap()
        };
        gr_context.reset(None);
        let surface = Surface::from_backend_render_target(
            &mut gr_context,
            &backend_render_target,
            surface_origin,
            color_type,
            None,
            Some(&skia_safe::SurfaceProps::new(
                Default::default(),
                skia_safe::PixelGeometry::Unknown, // for font rendering optimisations
            )),
        )
        .unwrap();

        // let raster_cache: FlatStorage<Image> = FlatStorage::new();
        Self {
            gr_context,
            surface,
            // raster_cache,
        }
    }

    pub fn create(
        width: impl Into<i32>,
        height: impl Into<i32>,
        sample_count: impl Into<usize>,
        stencil_bits: impl Into<usize>,
        color_type: ColorType,
        surface_origin: SurfaceOrigin,
        fboid: impl Into<u32>,
    ) -> Cell<Self> {
        Cell::new(Self::new(
            width.into(),
            height.into(),
            sample_count.into(),
            stencil_bits.into(),
            fboid.into(),
            color_type,
            surface_origin,
            None,
        ))
    }

    pub fn surface(&self) -> Surface {
        self.surface.clone()
    }
}
pub fn draw_node_children(
    node_id: NodeRef,
    arena: &Arena<SceneNode>,
    canvas: &mut skia_safe::Canvas,
    context_opacity: f32,
) {
    let node_id: NodeId = node_id.into();
    let parent_layer = arena.get(node_id).unwrap().get();
    let size = parent_layer.render_layer.read().unwrap().size;
    let bounds = Rect::from_xywh(
        0.0, //bounds.x as f32,
        0.0, //bounds.y as f32,
        size.x, size.y,
    );
    let context_opacity = parent_layer.layer.opacity() * context_opacity;
    // canvas.clip_rect(bounds, None, None);

    node_id.children(arena).for_each(|child_id| {
        let childindex: usize = child_id.into();
        let node = arena.get(child_id).unwrap().get();
        let render_layer = node.render_layer.read().unwrap();
        let child_opacity = render_layer.opacity;
        let opacity = context_opacity * child_opacity;
        let flags = node.flags.read().unwrap();
        drop(flags);

        let blend_mode = render_layer.blend_mode;
        let s = canvas.save();
        canvas.concat(&render_layer.transform);

        let draw_cache = node.draw_cache.read().unwrap();
        if let Some(draw_cache) = &*draw_cache {
            let restore_to = canvas.save();

            let bounds = Rect::from_xywh(0.0, 0.0, render_layer.size.x, render_layer.size.y);

            let mut paint = skia_safe::Paint::default();
            paint.set_alpha_f(opacity);

            if blend_mode == crate::prelude::BlendMode::BackgroundBlur {
                let border_corner_radius = render_layer.border_corner_radius;
                let rrbounds = skia_safe::RRect::new_rect_radii(
                    bounds,
                    &[
                        skia_safe::Point::new(
                            border_corner_radius.top_left,
                            border_corner_radius.top_left,
                        ),
                        skia_safe::Point::new(
                            border_corner_radius.top_right,
                            border_corner_radius.top_right,
                        ),
                        skia_safe::Point::new(
                            border_corner_radius.bottom_left,
                            border_corner_radius.bottom_left,
                        ),
                        skia_safe::Point::new(
                            border_corner_radius.bottom_right,
                            border_corner_radius.bottom_right,
                        ),
                    ],
                );
                canvas.clip_rrect(rrbounds, None, Some(true));

                let mut save_layer_rec = skia_safe::canvas::SaveLayerRec::default();
                let crop_rect = Some(skia_safe::image_filters::CropRect::from(bounds));

                let blur = skia_safe::image_filters::blur(
                    (50.0, 50.0),
                    skia_safe::TileMode::Clamp,
                    None,
                    crop_rect,
                )
                .unwrap();

                save_layer_rec = save_layer_rec.backdrop(&blur).bounds(&bounds).paint(&paint);
                canvas.save_layer(&save_layer_rec);
            }

            canvas.draw_picture(draw_cache.picture(), None, Some(&paint));
            canvas.restore_to_count(restore_to);
        } else {
            node.set_need_repaint(true);
            println!("no picture for child: {}", childindex);
        }
        draw_node_children(NodeRef(child_id), arena, canvas, opacity);
        canvas.restore_to_count(s);
    });
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
            let matrix = root.render_layer.read().unwrap().transform;
            let sc = canvas.save();
            canvas.concat(&matrix);

            draw_node_children(root_id, arena, canvas, 1.0);
            canvas.restore_to_count(sc);
        }

        surface.flush_and_submit();
    }
}

// implement Debug for SkiaFboRenderer
impl std::fmt::Debug for SkiaFboRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkiaFboRenderer").finish()
    }
}
