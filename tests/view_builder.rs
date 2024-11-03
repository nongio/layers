use lay_rs::{drawing::print_scene, layer_trees, layer_trees_opt, prelude::*, types::Size};

pub fn render_one_child_view(state: &bool, _view: &View<bool>) -> LayerTree {
    LayerTreeBuilder::default()
        .key("one_child_view")
        .position((Point { x: 0.0, y: 0.0 }, None))
        .size((
            Size {
                width: taffy::Dimension::Length(50.0),
                height: taffy::Dimension::Length(50.0),
            },
            None,
        ))
        .children(layer_trees_opt!(if *state {
            Some(
                LayerTreeBuilder::default()
                    .position((Point { x: 0.0, y: 0.0 }, None))
                    .size((
                        Size {
                            width: taffy::Dimension::Length(50.0),
                            height: taffy::Dimension::Length(50.0),
                        },
                        None,
                    ))
                    .build()
                    .unwrap(),
            )
        } else {
            None
        }))
        .build()
        .unwrap()
}
pub fn render_main_view(state: &bool, view: &View<bool>) -> LayerTree {
    let mut position = 0.0;
    if *state {
        position = 100.0;
    }
    let view = view.clone();

    LayerTreeBuilder::default()
        .key("main_view")
        .position((
            Point {
                x: position,
                y: position,
            },
            None,
        ))
        .size((
            Size {
                width: taffy::Dimension::Length(50.0),
                height: taffy::Dimension::Length(50.0),
            },
            None,
        ))
        .on_pointer_move(move |_, _, _| {
            println!("pointer move!!!!");
            view.update_state(&true);
        })
        .children(layer_trees!(
            LayerTreeBuilder::default()
                .key("text_view")
                .position((Point { x: 0.0, y: 0.0 }, None))
                .size((
                    Size {
                        width: taffy::Dimension::Length(50.0),
                        height: taffy::Dimension::Length(50.0),
                    },
                    None,
                ))
                .build()
                .unwrap(),
            View::new("sub_view", false, render_one_child_view),
            View::new("sub_view", *state, render_one_child_view),
        ))
        .build()
        .unwrap()
}

#[test]
pub fn simple_build() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.scene_add_layer(layer.clone());

    let lt = LayerTreeBuilder::default()
        .children(vec![LayerTreeBuilder::default().build().unwrap()])
        .build()
        .unwrap();
    layer.build_layer_tree(&lt);

    print_scene(engine.scene(), engine.scene_root().unwrap());
}

#[test]
pub fn build_a_view() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.scene_add_layer(layer.clone());

    let initial = false;
    let view = View::new("test_view", initial, render_one_child_view);
    view.mount_layer(layer);

    engine.update(0.016);

    // let x = view.layer.unwrap().render_position().x;
    print_scene(engine.scene(), engine.scene_root().unwrap());
    // assert!(x == 0.0);
}

#[test]
pub fn rebuild_a_view() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.scene_add_layer(layer.clone());

    let initial = false;
    let view = View::new("test_view", initial, render_one_child_view);
    view.mount_layer(layer);

    engine.update(0.016);
    let layer = view.layer.read().unwrap().clone().unwrap();

    print_scene(engine.scene(), engine.scene_root().unwrap());
    let num_children = layer.children_nodes().len();
    assert!(num_children == 0);

    view.update_state(&true);
    engine.update(0.016);
    print_scene(engine.scene(), engine.scene_root().unwrap());
    let num_children = layer.children_nodes().len();
    assert!(num_children == 1);

    view.update_state(&false);
    engine.update(0.016);
    print_scene(engine.scene(), engine.scene_root().unwrap());
    let num_children = layer.children_nodes().len();
    assert!(num_children == 0);

    view.update_state(&true);
    engine.update(0.016);
    print_scene(engine.scene(), engine.scene_root().unwrap());
    let num_children = layer.children_nodes().len();
    assert!(num_children == 1);
}

#[test]
pub fn nested_views() {
    let engine = LayersEngine::new(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.scene_add_layer(layer.clone());

    let initial = false;
    let view = View::new("parent_view", initial, render_main_view);
    view.mount_layer(layer);

    engine.update(0.016);
    let layer = view.layer.read().unwrap().clone().unwrap();

    print_scene(engine.scene(), engine.scene_root().unwrap());
    let num_children = layer.children_nodes().len();
    assert!(num_children == 3);

    println!("--");
    view.update_state(&true);
    engine.update(0.016);
    print_scene(engine.scene(), engine.scene_root().unwrap());

    println!("--");
    view.update_state(&false);
    engine.update(0.016);
    print_scene(engine.scene(), engine.scene_root().unwrap());

    println!("--");
    view.update_state(&true);
    engine.update(0.016);
    print_scene(engine.scene(), engine.scene_root().unwrap());

    let num_children = layer.children_nodes().len();
    assert!(num_children == 3);
}

// #[test]
// pub fn layer_tree_from_css() {
//     // let engine = LayersEngine::new(1000.0, 1000.0);
//     // let layer = engine.new_layer();

//     // engine.scene_add_layer(layer.clone());

//     let layer_tree = LayerTreeBuilder::default()
//         .key("test_layer")
//         .size(Size::points(0.0, 0.0))
//         // .scale((2.0, 2.0))
//         .position(Point::new(200.0, 200.0))
//         .background_color(Color::new_rgba255(255, 0, 0, 255))
//         .border_color(Color::new_rgba255(0, 0, 0, 255))
//         .border_corner_radius(BorderRadius::new_single(50.0))
//         .border_width((5.0, None))
//         .image_cache(true)
//         .build()
//         .unwrap();

//     println!("{:?}", layer_tree);
// }
