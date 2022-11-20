#![deny(warnings)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
pub mod drawing;
mod easing;
pub mod engine;
pub mod layers;
pub mod types;

use std::sync::*;

use drawing::scene::draw_scene;
use engine::{
    animations::{Easing, Transition},
    // animations::{Easing, Transition},
    backend::SkiaRenderer,
    // backend,
    node::RenderNode,
};
use layers::layer::ModelLayer;

use crate::types::BorderRadius;
// use types::Point;

#[no_mangle]
pub extern "C" fn engine_create() -> *const engine::Engine {
    let engine = engine::Engine::create();
    Arc::into_raw(engine)
}

#[no_mangle]
pub extern "C" fn engine_update(engine: *const engine::Engine, delta: f64) -> bool {
    let engine = unsafe { &*engine };
    engine.update(delta)
}

#[no_mangle]
pub extern "C" fn engine_add_layer(
    engine: *const engine::Engine,
    layer: *const layers::layer::ModelLayer,
) -> usize {
    let engine = unsafe { &*engine };
    let layer = unsafe { Arc::from_raw(layer) };

    engine.scene.add(layer as Arc<dyn RenderNode>).into()
}

#[no_mangle]
pub extern "C" fn engine_add_text(
    engine: *const engine::Engine,
    text: *const layers::text::ModelText,
) -> usize {
    let layer = unsafe { Arc::from_raw(text) };
    let engine = unsafe { &*engine };

    engine.scene.add(layer as Arc<dyn RenderNode>).into()
}

#[no_mangle]
pub extern "C" fn layer_create() -> *const ModelLayer {
    let layer = ModelLayer::create();
    Arc::into_raw(layer)
}

#[no_mangle]
pub extern "C" fn layer_animate(
    layer: *const ModelLayer,
    prop_name: *const libc::c_char,
    value: *mut (),
    t: Transition<Easing>,
) {
    let layer = unsafe { &*layer };

    use std::ffi::CStr;

    let prop_name = unsafe { CStr::from_ptr(prop_name) };
    let prop_name = prop_name.to_str().unwrap();
    match prop_name {
        "position" => {
            let value = unsafe { *(value as *const types::Point) };
            layer.set_position(value, Some(t));
        }
        "size" => {
            let value = unsafe { *(value as *const types::Point) };
            layer.set_size(value, Some(t));
        }
        "border_radius" => {
            let value = unsafe { *(value as *const f64) };
            layer.set_border_corner_radius(BorderRadius::new_single(value), Some(t));
        }
        _ => println!("something else!"),
    }
}

#[no_mangle]
pub extern "C" fn layer_position_to(
    layer: *const ModelLayer,
    x: f64,
    y: f64,
    t: Transition<Easing>,
) {
    let layer = unsafe { &*layer };
    layer.set_position(types::Point { x, y }, Some(t));
}

#[no_mangle]
pub extern "C" fn layer_position_get(layer: *const ModelLayer) -> types::Point {
    let layer = unsafe { &*layer };
    layer.position.value()
}

#[no_mangle]
pub extern "C" fn text_create() -> *const layers::text::ModelText {
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
    paint.set_style(skia_bindings::SkPaint_Style::Fill);
    let renderer = unsafe { &mut *renderer };
    let canvas = renderer.surface.canvas();
    let w = canvas.image_info().width() as f32;
    let h = canvas.image_info().height() as f32;

    canvas.draw_rect(skia_safe::Rect::from_xywh(0.0, 0.0, w, h), &paint);
    let engine = unsafe { &*engine };
    draw_scene(canvas, &engine.scene);
    renderer.surface.flush_and_submit();
}
