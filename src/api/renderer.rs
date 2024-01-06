use skia_safe::ColorType;

use crate::{drawing::scene::DrawScene, engine, renderer::skia_fbo::SkiaFboRenderer};

#[no_mangle]
pub extern "C" fn create_skia_renderer(
    width: i32,
    height: i32,
    sample_count: usize,
    stencil_bits: usize,
    fboid: u32,
) -> *mut SkiaFboRenderer {
    let renderer = SkiaFboRenderer::new(
        width,
        height,
        sample_count,
        stencil_bits,
        fboid,
        ColorType::RGBA8888,
        skia_safe::gpu::SurfaceOrigin::BottomLeft,
        None,
    );
    Box::into_raw(Box::new(renderer))
}

#[no_mangle]
pub extern "C" fn render_scene(
    renderer: *mut SkiaFboRenderer,
    engine: *const engine::LayersEngine,
) {
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
    if let Some(root) = engine.scene_root() {
        renderer.draw_scene(engine.scene(), root);
    }
    // renderer.surface.flush_and_submit();
}
