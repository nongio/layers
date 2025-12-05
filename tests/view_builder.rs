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
                    .key("child_view")
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
        .on_pointer_move(move |_: &Layer, _, _| {
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
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.add_layer(&layer);

    let lt = LayerTreeBuilder::default()
        .children(vec![LayerTreeBuilder::default().build().unwrap()])
        .build()
        .unwrap();
    layer.build_layer_tree(&lt);

    print_scene(engine.scene(), engine.scene_root().unwrap());
}

#[test]
pub fn build_a_view() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.add_layer(&layer);

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
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.add_layer(&layer);

    let initial = false;
    let view = View::new("test_view", initial, render_one_child_view);
    view.mount_layer(layer.clone());

    engine.update(0.016);

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
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();

    engine.add_layer(&layer);

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

#[test]
fn layer_tree_builder_children_recover_from_zero_opacity_parent() {
    let engine = Engine::create(1000.0, 1000.0);
    let layer = engine.new_layer();
    engine.add_layer(&layer);

    let child_tree = LayerTreeBuilder::default()
        .key("child")
        .size((
            Size {
                width: taffy::Dimension::Length(10.0),
                height: taffy::Dimension::Length(10.0),
            },
            None,
        ))
        .background_color((
            PaintColor::Solid {
                color: Color::new_hex("#ffffffff"),
            },
            None,
        ))
        .build()
        .unwrap();

    let root_tree = LayerTreeBuilder::default()
        .key("root")
        .opacity((0.0, None))
        .size((
            Size {
                width: taffy::Dimension::Length(10.0),
                height: taffy::Dimension::Length(10.0),
            },
            None,
        ))
        .children(vec![child_tree])
        .build()
        .unwrap();
    layer.build_layer_tree(&root_tree);

    engine.update(0.016);
    let child = layer
        .children()
        .into_iter()
        .next()
        .expect("child layer missing after build");
    assert_eq!(child.render_layer().premultiplied_opacity, 0.0);

    layer.set_opacity(1.0, None);
    engine.update(0.016);
    let child = layer
        .children()
        .into_iter()
        .next()
        .expect("child layer missing after opacity change");
    assert!(child.render_layer().premultiplied_opacity > 0.0);
}

#[test]
fn layer_tree_replicate_node() {
    let engine = Engine::create(1000.0, 1000.0);

    // Create a root layer first so source_layer and replica_holder are siblings
    let root = engine.new_layer();
    root.set_size(Size::points(1000.0, 1000.0), None);
    engine.add_layer(&root);

    // Create a source layer to replicate
    let source_layer = engine.new_layer();
    source_layer.set_size(Size::points(100.0, 100.0), None);
    source_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    root.add_sublayer(&source_layer);

    // Create a layer to hold the replicated content
    let replica_holder = engine.new_layer();
    replica_holder.set_size(Size::points(200.0, 200.0), None);
    root.add_sublayer(&replica_holder);

    // Build a layer tree that replicates the source layer
    // This sets the content_draw_func on replica_holder to render source_layer
    let replicate_tree = LayerTreeBuilder::default()
        .key("replica")
        .size((Size::points(100.0, 100.0), None))
        .replicate_node(source_layer.id())
        .build()
        .unwrap();

    replica_holder.build_layer_tree(&replicate_tree);

    // Update the engine - this should not cause a stack overflow
    engine.update(0.016);
    engine.update(0.016);

    // The test passes if we get here without a stack overflow
    assert!(true);
}

#[test]
fn layer_tree_replicate_node_descendant_follower() {
    // This tests the case where the follower (replica) is a descendant of the leader (source)
    // which previously caused infinite recursion
    let engine = Engine::create(1000.0, 1000.0);

    // Create a source layer - this will be the root AND the leader
    let source_layer = engine.new_layer();
    source_layer.set_size(Size::points(100.0, 100.0), None);
    source_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    engine.add_layer(&source_layer);

    // Create a replica layer as a CHILD of source_layer
    // This creates the circular dependency: rendering source_layer renders replica,
    // and replica's content_draw_func tries to render source_layer again
    let replica = engine.new_layer();
    replica.set_size(Size::points(50.0, 50.0), None);
    source_layer.add_sublayer(&replica);

    // Set up the replica to mirror the source
    let replicate_tree = LayerTreeBuilder::default()
        .key("replica")
        .replicate_node(source_layer.id())
        .build()
        .unwrap();

    replica.build_layer_tree(&replicate_tree);

    // This should not cause a stack overflow - the recursion prevention should kick in
    engine.update(0.016);
    engine.update(0.016);

    // The test passes if we get here without a stack overflow
    assert!(true);
}

// #[test]
// pub fn layer_tree_from_css() {
//     // let engine = Engine::create(1000.0, 1000.0);
//     // let layer = engine.new_layer();

//     // engine.add_layer(&layer);

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
