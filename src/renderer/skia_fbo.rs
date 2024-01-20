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
    prelude::render_node_tree,
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

impl DrawScene for SkiaFboRenderer {
    fn draw_scene(&self, scene: &Scene, root_id: NodeRef, damage: Option<skia_safe::Rect>) {
        let mut surface = self.surface();
        let canvas = surface.canvas();
        let save_point = canvas.save();
        if let Some(damage) = damage {
            canvas.clip_rect(damage, None, None);
        }
        let arena = scene.nodes.data();
        let arena = &*arena.read().unwrap();
        if let Some(_root) = scene.get_node(root_id) {
            render_node_tree(root_id, arena, canvas, 1.0);
        }
        canvas.restore_to_count(save_point);
        surface.flush_and_submit();
    }
}

// implement Debug for SkiaFboRenderer
impl std::fmt::Debug for SkiaFboRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkiaFboRenderer").finish()
    }
}
