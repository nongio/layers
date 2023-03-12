use std::sync::Arc;

use crate::{
    engine::{
        self,
        // pointer::{ButtonState, PointerHandler},
    },
    layers::{layer::Layer, text::TextLayer},
};

/// C api to the LayersEngine
#[no_mangle]
pub extern "C" fn create_engine() -> *const engine::LayersEngine {
    let engine = Arc::new(engine::LayersEngine::new());
    Arc::into_raw(engine)
}

#[no_mangle]
pub extern "C" fn engine_update(engine: *const engine::LayersEngine, delta: f64) -> bool {
    let engine = unsafe { &*engine };
    engine.update(delta)
}

#[no_mangle]
pub extern "C" fn engine_add_layer_to_scene(
    engine: *const engine::LayersEngine,
    layer: *const Layer,
) -> usize {
    let engine = unsafe { &*engine };
    let layer = unsafe { Arc::from_raw(layer) };
    let layer = (*layer).clone();

    engine.scene_add_layer(layer).0.into()
}

#[no_mangle]
pub extern "C" fn engine_add_text(
    engine: *const engine::LayersEngine,
    text: *const TextLayer,
) -> usize {
    let engine = unsafe { &*engine };
    let layer = unsafe { Arc::from_raw(text) };
    let layer = (*layer).clone();

    engine.scene_add_layer(layer).0.into()
}

// #[no_mangle]
// pub extern "C" fn engine_handle_pointer_button(
//     engine: *const engine::EngineApi,
//     state: ButtonState,
// ) {
//     let engine = unsafe { &*engine };
//     match state {
//         ButtonState::Pressed => {
//             engine.on_pointer_down(0);
//         }
//         ButtonState::Released => {
//             engine.on_pointer_up(0);
//         }
//     };
// }
