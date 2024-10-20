use layers::{
    drawing::scene::debug_scene,
    engine::{node::RenderableFlags, LayersEngine},
    skia,
    types::{Point, Size},
    view::{BuildLayerTree, LayerTreeBuilder, RenderLayerTree},
};

#[test]
pub fn node_flags() {
    // empty flags do not check for need paint
    let mut flags: RenderableFlags = RenderableFlags::empty();
    assert!(!flags.contains(RenderableFlags::NEEDS_PAINT));

    // set needs paint and check, needs_layout should be false
    flags.set(RenderableFlags::NEEDS_PAINT, true);
    assert!(flags.contains(RenderableFlags::NEEDS_PAINT));
    assert!(!flags.contains(RenderableFlags::NEEDS_LAYOUT));

    // set both needs paint and needs layout and check
    let mut flags: RenderableFlags = RenderableFlags::empty();
    let new_flags = RenderableFlags::NEEDS_PAINT | RenderableFlags::NEEDS_LAYOUT;
    flags.insert(new_flags);
    assert!(flags.contains(RenderableFlags::NEEDS_PAINT));
    assert!(flags.contains(RenderableFlags::NEEDS_LAYOUT));
}

#[test]
pub fn bounds_with_children() {
    let w = 500.0;
    let h = 500.0;
    const SAFE_AREA: f32 = 100.0;

    let engine = LayersEngine::new(1000.0, 1000.0);
    let wrap = engine.new_layer();
    let layer = engine.new_layer();

    layer.set_position((100.0, 100.0), None);
    layer.set_size(Size::points(0.0, 0.0), None);

    engine.scene_add_layer(wrap.clone());
    engine.scene_add_layer_to(layer.clone(), wrap.clone());

    let draw_shadow = move |canvas: &layers::skia::Canvas, w: f32, h: f32| {
        layers::skia::Rect::from_xywh(0.0, 0.0, w, h)
    };
    let tree = LayerTreeBuilder::default()
        .key("a")
        .layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        })
        .position(Point { x: 0.0, y: 0.0 })
        .size(Size::points(w, h))
        .children(vec![LayerTreeBuilder::default()
            .key("b")
            .layout_style(taffy::Style {
                position: taffy::Position::Absolute,
                ..Default::default()
            })
            .position((
                Point {
                    x: -SAFE_AREA,
                    y: -SAFE_AREA,
                },
                None,
            ))
            .size(Size::points(w + SAFE_AREA * 2.0, h + SAFE_AREA * 2.0))
            .content(Some(draw_shadow))
            // .image_cache(true)
            .build()
            .unwrap()])
        .build()
        .unwrap();

    layer.build_layer_tree(&tree);

    engine.update(0.016);
    let bounds_with_children = wrap.render_bounds_with_children();

    assert_eq!(
        bounds_with_children,
        skia::Rect::from_xywh(-100.0, -100.0, 700.0, 700.0)
    );
}

// #[test]
// pub fn bounds_with_children_image_cache() {
//     let w = 500.0;
//     let h = 500.0;
//     const SAFE_AREA: f32 = 100.0;

//     let engine = LayersEngine::new(1000.0, 1000.0);
//     let layer = engine.new_layer();
//     layer.set_position((100.0, 100.0), None);
//     layer.set_size(Size::points(0.0, 0.0), None);

//     engine.scene_add_layer(layer.clone());

//     let draw_shadow = move |canvas: &layers::skia::Canvas, w: f32, h: f32| {
//         layers::skia::Rect::from_xywh(0.0, 0.0, w, h)
//     };
//     let tree = LayerTreeBuilder::default()
//         .key("window_shadow")
//         .size(Size::points(w, h))
//         .children(vec![LayerTreeBuilder::default()
//             .key("window_shadow_inner")
//             .layout_style(taffy::Style {
//                 position: taffy::Position::Absolute,
//                 ..Default::default()
//             })
//             .position((
//                 Point {
//                     x: -SAFE_AREA,
//                     y: -SAFE_AREA,
//                 },
//                 None,
//             ))
//             .size(Size::points(w + SAFE_AREA * 2.0, h + SAFE_AREA * 2.0))
//             .content(Some(draw_shadow))
//             .image_cache(true)
//             .build()
//             .unwrap()])
//         .build()
//         .unwrap();

//     layer.build_layer_tree(&tree);

//     engine.update(0.016);
//     let bounds_with_children = layer.render_bounds_with_children();

//     assert_eq!(
//         bounds_with_children,
//         skia::Rect::from_xywh(-100.0, -100.0, 700.0, 700.0)
//     );
// }
