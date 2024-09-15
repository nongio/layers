use layers::engine::LayersEngine;

#[test]
pub fn state_for_layer() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    layer.with_mut_state(|state| {
        state.insert("age", 33);
    });
    let age = layer.with_state(|state| {
        if let Some(age) = state.get::<i32>("age") {
            println!("age: {:?}", age);
            return age;
        }
        0
    });
    assert!(age == 33);
}
