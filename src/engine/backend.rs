use gl::types::*;
use gl_rs as gl;
use glutin::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile, PixelFormat,
};

use skia_bindings::GrDirectContext;
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    Canvas, Color4f, ColorType, Paint, Rect, Surface,
};
use std::{cell::Cell, sync::Arc};

type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

pub struct SkiaRenderer {
    pub gr_context: skia_safe::gpu::DirectContext,
    pub surface: Surface,
}
impl SkiaRenderer {
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
        let backend_render_target = BackendRenderTarget::new_gl((width, height), 1, 8, fb_info);

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
