use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    ColorType, Surface,
};
use std::cell::Cell;

use crate::drawing::scene::{draw_scene, DrawScene};
use crate::engine::scene::Scene;

pub struct SkiaFboRenderer {
    pub gr_context: skia_safe::gpu::DirectContext,
    pub surface: Surface,
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

        Self {
            gr_context,
            surface,
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

    pub fn surface(&mut self) -> &mut Surface {
        &mut self.surface
    }
}

impl DrawScene for SkiaFboRenderer {
    fn draw_scene(&mut self, scene: &Scene) {
        let surface = self.surface();
        let c = surface.canvas();
        draw_scene(c, scene);
        surface.flush_and_submit();
    }
}
