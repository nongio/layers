use layers::{
    engine::{animation::Transition, LayersEngine},
    prelude::Layer,
};

/// it should call the finish handler when the transaction is finished 1 time
#[test]
pub fn linear_animation() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    layer
        .set_opacity(0.0, Some(Transition::linear(1.0)))
        .on_update(
            move |l: &Layer, p| {
                let opacity = l.opacity();
                println!("{} animation opacity: {}", p, opacity);
            },
            false,
        );
    engine.update(0.2);
    engine.update(0.2);
    engine.update(0.2);
    engine.update(0.2);
    engine.update(0.2);
    engine.update(0.2);

    let x1 = 0.0;
    let y1 = 0.0;
    let x2 = 1.0;
    let y2 = 1.0;
    let ease = bezier_easing::bezier_easing(x1, y1, x2, y2).unwrap();
    println!("ease(0.0): {}", ease(0.0));
    println!("ease(0.5): {}", ease(0.5));
    println!("ease(1.0): {}", ease(1.0));
    assert_eq!(ease(0.0), 0.0);
    assert_eq!(ease(0.5), 0.5);
    assert_eq!(ease(1.0), 1.0);
}
