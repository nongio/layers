use std::sync::Arc;

use crate::{
    engine::{
        self,
        Engine,
        // pointer::{ButtonState, PointerHandler},
    },
    layers::layer::Layer,
};

/// C api to the Engine
#[no_mangle]
pub extern "C" fn create_engine(width: f32, height: f32) -> *const engine::Engine {
    let engine = Engine::create(width, height);
    Arc::into_raw(engine)
}

#[no_mangle]
pub extern "C" fn engine_update(engine: *const engine::Engine, delta: f32) -> bool {
    let engine = unsafe { &*engine };
    engine.update(delta)
}

#[no_mangle]
pub extern "C" fn engine_add_layer_to_scene(
    engine: *const engine::Engine,
    layer: *const Layer,
) -> usize {
    let engine = unsafe { &*engine };
    let layer = unsafe { &*layer };
    let layer = (*layer).clone();

    engine.append_layer(layer, None).into()
}

#[no_mangle]
pub extern "C" fn engine_create_layer(engine: *const engine::Engine) -> *const Layer {
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
