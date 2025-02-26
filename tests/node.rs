use lay_rs::{
    prelude::*,
    skia,
    types::{Point, Size},
    view::LayerTreeBuilder,
};

#[test]
pub fn bounds_with_children() {
    let w = 500.0;
    let h = 500.0;
    const SAFE_AREA: f32 = 100.0;

    let engine = Engine::create(1000.0, 1000.0);
    let wrap = engine.new_layer();
    let layer = engine.new_layer();

    layer.set_position((100.0, 100.0), None);
    layer.set_size(Size::points(0.0, 0.0), None);

    engine.add_layer(wrap.clone());
    engine.append_layer(layer.clone(), wrap.id);

    let draw_shadow = move |_canvas: &lay_rs::skia::Canvas, w: f32, h: f32| {
        lay_rs::skia::Rect::from_xywh(0.0, 0.0, w, h)
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

//     let engine = Engine::create(1000.0, 1000.0);
//     let layer = engine.new_layer();
//     layer.set_position((100.0, 100.0), None);
//     layer.set_size(Size::points(0.0, 0.0), None);

//     engine.add_layer(layer.clone());

//     let draw_shadow = move |canvas: &lay_rs::skia::Canvas, w: f32, h: f32| {
//         lay_rs::skia::Rect::from_xywh(0.0, 0.0, w, h)
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
