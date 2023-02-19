#![deny(warnings)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
// negative_impl is used to prevent the compiler from using the default implementation of
// the trait Interpolable for PaintColor
#![feature(negative_impls)]

pub mod api;
pub mod drawing;
mod easing;
pub mod engine;
pub mod models;
pub mod renderer;
pub mod types;

use std::sync::*;

use drawing::scene::DrawScene;
use renderer::skia_fbo::SkiaFboRenderer;

#[no_mangle]
pub extern "C" fn create_text() -> *const models::text::ModelText {
    let text = models::text::ModelText::create();
    Arc::into_raw(text)
}

#[no_mangle]
pub extern "C" fn create_skia_renderer(
    width: i32,
    height: i32,
    sample_count: usize,
    stencil_bits: usize,
    fboid: usize,
) -> *mut SkiaFboRenderer {
    let renderer = SkiaFboRenderer::new(width, height, sample_count, stencil_bits, fboid);
    Box::into_raw(Box::new(renderer))
}

#[no_mangle]
pub extern "C" fn render_scene(renderer: *mut SkiaFboRenderer, engine: *const engine::Engine) {
    let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.6, 0.6, 0.6, 1.0), None);
    paint.set_anti_alias(true);
    // paint.set_style(skia_bindings::SkPaint_Style::Fill);
    let renderer = unsafe { &mut *renderer };
    let canvas = renderer.surface.canvas();
    let w = canvas.image_info().width() as f32;
    let h = canvas.image_info().height() as f32;

    canvas.draw_rect(skia_safe::Rect::from_xywh(0.0, 0.0, w, h), &paint);
    let engine = unsafe { &*engine };
    // draw_scene(canvas, &engine.scene);
    renderer.draw_scene(&engine.scene);
    // renderer.surface.flush_and_submit();
}
