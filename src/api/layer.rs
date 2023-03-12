// use std::sync::Arc;

// use crate::types;

// use super::super::engine::animations::{Easing, Transition};
// use super::super::layers::layer::ModelLayer;
// use super::super::types::BorderRadius;

// #[no_mangle]
// pub extern "C" fn create_layer() -> *const ModelLayer {
//     let layer = ModelLayer::create();
//     Arc::into_raw(layer)
// }

// #[no_mangle]
// pub extern "C" fn layer_backgroundcolor_to(
//     layer: *const ModelLayer,
//     r: f64,
//     g: f64,
//     b: f64,
//     a: f64,
//     t: Transition<Easing>,
// ) {
//     let layer = unsafe { &*layer };
//     let bg = types::PaintColor::Solid {
//         color: types::Color::new_rgba(r, g, b, a),
//     };
//     layer.set_background_color(bg, Some(t));
// }
// #[no_mangle]
// pub extern "C" fn layer_backgroundcolor_set(
//     layer: *const ModelLayer,
//     r: f64,
//     g: f64,
//     b: f64,
//     a: f64,
// ) {
//     let layer = unsafe { &*layer };
//     let bg = types::PaintColor::Solid {
//         color: types::Color::new_rgba(r, g, b, a),
//     };
//     layer.set_background_color(bg, None);
// }
// #[no_mangle]
// pub extern "C" fn layer_size_to(layer: *const ModelLayer, x: f64, y: f64, t: Transition<Easing>) {
//     let layer = unsafe { &*layer };
//     let size = types::Point { x, y };
//     layer.set_size(size, Some(t));
// }

// #[no_mangle]
// pub extern "C" fn layer_border_radius_to(layer: *const ModelLayer, r: f64, t: Transition<Easing>) {
//     let layer = unsafe { &*layer };

//     layer.set_border_corner_radius(BorderRadius::new_single(r), Some(t));
// }
// #[no_mangle]
// pub extern "C" fn layer_position_to(
//     layer: *const ModelLayer,
//     x: f64,
//     y: f64,
//     t: Transition<Easing>,
// ) {
//     let layer = unsafe { &*layer };
//     layer.set_position(types::Point { x, y }, Some(t));
// }

// #[no_mangle]
// pub extern "C" fn layer_position_get(layer: *const ModelLayer) -> types::Point {
//     let layer = unsafe { &*layer };
//     layer.position.value()
// }

// #[no_mangle]
// pub extern "C" fn layer_on_click(layer: *const ModelLayer, _callback: unsafe extern "C" fn()) {
//     let _layer = unsafe { &*layer };
//     // layer.add_on_click_handler(move || unsafe {
//     // callback();
//     // });
// }
