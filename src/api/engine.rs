use std::sync::Arc;

use crate::{
    engine::{
        self,
        node::RenderNode,
        pointer::{ButtonState, PointerHandler},
    },
    layers::{self},
};

#[no_mangle]
pub extern "C" fn create_engine() -> *const engine::Engine {
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
pub extern "C" fn engine_handle_pointer_button(engine: *const engine::Engine, state: ButtonState) {
    let engine = unsafe { &*engine };
    match state {
        ButtonState::Pressed => {
            engine.on_pointer_down(0);
        }
        ButtonState::Released => {
            engine.on_pointer_up(0);
        }
    };
}
