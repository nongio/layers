#![deny(warnings)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
pub mod api;
pub mod drawing;
mod easing;
pub mod engine;
pub mod layers;
pub mod types;

use std::sync::*;

use drawing::scene::DrawScene;
use engine::backend::SkiaRenderer;

#[no_mangle]
pub extern "C" fn create_text() -> *const layers::text::ModelText {
    let text = layers::text::ModelText::create();
    Arc::into_raw(text)
}

#[no_mangle]
pub extern "C" fn create_skia_renderer(
    width: i32,
    height: i32,
    sample_count: usize,
    stencil_bits: usize,
    fboid: usize,
) -> *mut SkiaRenderer {
    let renderer = SkiaRenderer::new(width, height, sample_count, stencil_bits, fboid);
    Box::into_raw(Box::new(renderer))
}

#[no_mangle]
pub extern "C" fn render_scene(renderer: *mut SkiaRenderer, engine: *const engine::Engine) {
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
