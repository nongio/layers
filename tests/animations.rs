use layers::{
    engine::{animation::Transition, LayersEngine},
    prelude::{spring::Spring, Layer, TimingFunction},
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

/// it should call the finish handler when the transaction is finished 1 time
#[test]
pub fn spring_animation() {
    let solver = Spring::with_duration_and_bounce(1.0, 0.2);
    println!("solver: {:?}", solver);
    let mut spring = Spring::new(1.0, 100.0, 2.0);
    let scale = 5;
    let mut i = 0.0;
    while !spring.done(i) && i < 2.0 {
        i += 0.016;

        let update_value = spring.update_at(i);
        // println!("update_value: {}", update_value);
        // let update_value = update_value.clamp(-1.0, 1.0);
        let num_x = (update_value * scale as f32).round();
        if num_x >= 0.0 {
            let num_x = num_x as usize;
            println!(
                "[{:.4}][{:.4}]{}|{}",
                i,
                update_value,
                " ".repeat(scale),
                "x".repeat(num_x)
            );
        } else {
            let num_x = num_x.abs() as usize;
            let num_x = num_x.clamp(0, scale);
            println!(
                "[{:.4}][{:.4}]{}{}|",
                i,
                update_value,
                " ".repeat(scale - num_x),
                "x".repeat(num_x)
            );
        }
    }
    // assert_eq!(spring.update(i), 0.0);
    // assert!(i < 1.0)
}

/// it should call the finish handler when the transaction is finished 1 time
#[test]
pub fn merge_spring_animation() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.scene_add_layer(layer.clone());

    let tr = layer.set_position((10.0, 10.0), Transition::spring(2.0, 0.2));

    engine.update(0.5);
    engine.update(0.5);

    let tr = engine.get_transaction(tr).unwrap();
    let animation_id = tr.animation_id.unwrap();
    let animation_state = engine.get_animation(animation_id).unwrap();
    let animation = animation_state.animation;
    let current_velocity = match animation.timing {
        TimingFunction::Spring(spring) => {
            let (_current_position, current_velocity) =
                spring.update_pos_vel_at(animation_state.time);
            current_velocity
        }
        _ => 0.0,
    };
    layer.set_position(
        (100.0, 100.0),
        Transition::spring_with_velocity(2.0, 0.2, current_velocity),
    );
    engine.update(0.5);
    engine.update(0.5);
}
