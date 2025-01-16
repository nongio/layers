use std::sync::Arc;

use crate::{
    engine::{
        self,
        // pointer::{ButtonState, PointerHandler},
    },
    layers::layer::Layer,
};

/// C api to the LayersEngine
#[no_mangle]
pub extern "C" fn create_engine(width: f32, height: f32) -> *const engine::LayersEngine {
    let engine = Arc::new(engine::LayersEngine::new(width, height));
    Arc::into_raw(engine)
}

#[no_mangle]
pub extern "C" fn engine_update(engine: *const engine::LayersEngine, delta: f32) -> bool {
    let engine = unsafe { &*engine };
    engine.update(delta)
}

#[no_mangle]
pub extern "C" fn engine_add_layer_to_scene(
    engine: *const engine::LayersEngine,
    layer: *const Layer,
) -> usize {
    let engine = unsafe { &*engine };
    let layer = unsafe { &*layer };
    let layer = (*layer).clone();

    engine.add_layer(layer).0.into()
}

#[no_mangle]
pub extern "C" fn engine_create_layer(engine: *const engine::LayersEngine) -> *const Layer {
    let engine = unsafe { &*engine };
    let layer = Arc::new(engine.new_layer());
    Arc::into_raw(layer)
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
