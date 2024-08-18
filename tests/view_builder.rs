use std::sync::{Arc, RwLock};

use layers::{
    drawing::scene::debug_scene,
    prelude::{timing::TimingFunction, *},
    types::Size,
};

pub fn render_view(state: &bool, view: &View<bool>) -> ViewLayer {
    let mut position = 0.0;
    if *state {
        position = 100.0;
    }
    let view = view.clone();
    ViewLayerBuilder::default()
        .key("this_is_a_view")
        .position((
            Point {
                x: position,
                y: position,
            },
            None,
        ))
        .size((
            Size {
                width: taffy::Dimension::Points(50.0),
                height: taffy::Dimension::Points(50.0),
            },
            None,
        ))
        .on_pointer_move(move |_, _| {
            println!("pointer move!!!!");
            view.update_state(true);
        })
        .children(vec![ViewLayerBuilder::default()
            .key("this_is_a_text")
            .position((Point { x: 0.0, y: 0.0 }, None))
            .size((
                Size {
                    width: taffy::Dimension::Points(50.0),
                    height: taffy::Dimension::Points(50.0),
                },
                None,
            ))
            .build()
            .unwrap()])
        .build()
        .unwrap()
}

#[test]
pub fn build_a_view() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.scene_add_layer(layer.clone());

    let initial = false;
    let view = layers::prelude::View::new(layer, initial, Box::new(render_view));

    engine.update(0.016);

    let x = view.layer.render_position().x;

    debug_scene(engine.scene(), engine.scene_root().unwrap());
    assert!(x == 0.0);
}

#[test]
pub fn pointer_move() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.scene_add_layer(layer.clone());

    let initial = false;
    let view = layers::prelude::View::new(layer, initial, Box::new(render_view));
    engine.update(0.016);

    let root_id = engine.scene_root().unwrap();
    engine.pointer_move((0.0, 0.0), root_id.0);

    engine.update(0.016);

    debug_scene(engine.scene(), engine.scene_root().unwrap());

    assert!(view.get_state());
    assert_eq!(view.layer.render_position().x, 100.0);
}
