#![deny(warnings)]

pub mod easing;
pub mod engine;
pub mod layers;
pub mod rendering;
pub mod types;

// use std::sync::Arc;

// use layers::layer::{BlendMode, ModelLayer};
// use layers::ModelChange;

// use crate::engine::Scene;
// use crate::layers::layer::Layer;
// use crate::types::*;
// use skia_safe::Matrix;

// #[no_mangle]
// pub extern "C" fn new_state() -> *mut Scene {
//     let pointer = Box::new(Scene::new());
//     Box::into_raw(pointer)
// }
// /// # Safety
// /// This is unsafe because it is the caller's responsibility to ensure that the pointer is valid.
// #[no_mangle]
// pub unsafe extern "C" fn state_add_layer(state_pointer: *mut Scene, model: *mut ModelLayer) {
//     let state = state_pointer.as_mut().unwrap();

//     let model = model.as_ref().unwrap();

//     state.add_model(Arc::new(model.clone()));
// }
// /// # Safety
// /// This is unsafe because it is the caller's responsibility to ensure that the pointer is valid.
// #[no_mangle]
// pub unsafe extern "C" fn model_from_layer(layer: Layer) -> *mut ModelLayer {
//     let layer = Layer {
//         content: None,
//         background_color: PaintColor::Solid {
//             color: Color::new(1.0, 1.0, 1.0, 0.5),
//         },
//         border_color: PaintColor::Solid {
//             color: Color::new(1.0, 1.0, 1.0, 0.3),
//         },
//         border_width: 1.0,
//         size: Point { x: 1000.0, y: 90.0 },
//         border_style: BorderStyle::Solid,
//         border_corner_radius: BorderRadius::new_single(25.0),
//         shadow_offset: Point { x: 00.0, y: 0.0 },
//         shadow_color: Color::new(0.0, 0.0, 0.0, 0.3),
//         shadow_radius: 20.0,
//         shadow_spread: 0.0,
//         matrix: Matrix::new_identity(),
//         blend_mode: BlendMode::Normal,
//     };

//     let pointer = Box::new(ModelLayer::from(layer));
//     Box::into_raw(pointer)
// }
// /// # Safety
// /// This function is unsafe because it is unsafe to call `Box::from_raw` on a pointer that was not created by `Box::into_raw`.
// /// This is because the pointer is not guaranteed to be valid.
// #[no_mangle]
// pub unsafe extern "C" fn state_update(state_pointer: *mut Scene, dt: f64) {
//     let mut state = state_pointer.as_mut().unwrap();

//     state.update(dt);
// }

// /// # Safety
// /// This function is unsafe because it is unsafe to call `Box::from_raw` on a pointer that was not created by `Box::into_raw`.
// /// This is because the pointer is not guaranteed to be valid.
// #[no_mangle]
// pub unsafe extern "C" fn state_commit(
//     state_pointer: *mut Scene,
//     change: *const ModelChange<Point>,
// ) {
//     let mut state = state_pointer.as_mut().unwrap();

//     let change = unsafe { Arc::from_raw(change) };

//     state.add_change(change);
// }

// /// # Safety
// /// This function is unsafe because it is unsafe to call `Box::from_raw` on a pointer that was
// /// not created by `Box::into_raw`.
// #[no_mangle]
// pub unsafe extern "C" fn model_change_position(
//     model_pointer: *mut ModelLayer,
//     x: f64,
//     y: f64,
// ) -> *const ModelChange<Point> {
//     let model = model_pointer.as_mut().unwrap();

//     Arc::into_raw(model.position(Point { x, y }, None))
// }
// /// # Safety
// /// This function is unsafe because it dereferences the pointer.
// #[no_mangle]
// pub unsafe extern "C" fn debug_change(change_pointer: *const ModelChange<Point>) {
//     let change = change_pointer.as_ref().unwrap();
//     println!("{:?}", change);
// }

// /// # Safety
// /// This function is unsafe because it dereferences the pointer.
// #[no_mangle]
// pub unsafe extern "C" fn debug_model(model_pointer: *mut ModelLayer) {
//     let model = model_pointer.as_ref().unwrap();
//     println!("{:?}", model.position.value());
// }
