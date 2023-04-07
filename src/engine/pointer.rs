use super::Engine;

#[repr(C)]
pub enum ButtonState {
    Pressed,
    Released,
}

pub trait PointerHandler {
    fn on_pointer_down(&self, timestamp: u32);
    fn on_pointer_up(&self, timestamp: u32);
    fn on_pointer_move(&self, pointer_x: f32, pointer_y: f32, timestamp: u32);
}

impl PointerHandler for Engine {
    fn on_pointer_down(&self, timestamp: u32) {
        println!("engine pointer down {}", timestamp);
        // self.scene.root.
    }
    fn on_pointer_up(&self, timestamp: u32) {
        println!("engine pointer up {}", timestamp);
    }
    fn on_pointer_move(&self, _pointer_x: f32, _pointer_y: f32, _timestamp: u32) {}
}
