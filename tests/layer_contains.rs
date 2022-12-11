use std::f64::consts::PI;
use std::sync::Arc;

use layers::drawing::scene::DrawScene;
use layers::engine::node::{ContainsPoint, RenderNode};
use layers::engine::rendering::Drawable;
use layers::models::layer::ModelLayer;
use layers::types::{BorderRadius, Color, PaintColor, Point, Point3d};
use skia_safe::{Color4f, Paint, Rect};

#[test]
pub fn layer_contains() {
    let layer = ModelLayer::create();

    layer.set_size(Point { x: 100.0, y: 100.0 }, None);

    assert!(layer.bounds().contains(Point { x: 50.0, y: 50.0 }));
    assert!(!layer.bounds().contains(Point { x: 200.0, y: 50.0 }));

    layer.set_position(Point { x: 100.0, y: 100.0 }, None);
    // bounds rectangle is in layer coordinates, origin is 0,0
    assert!(!layer.bounds().contains(Point { x: 200.0, y: 200.0 }));
    assert!(layer.bounds().contains(Point { x: 50.0, y: 50.0 }));
}

#[test]
pub fn scene_node_contains() {
    let engine = layers::engine::Engine::create();
    let layer = ModelLayer::create();
    engine.scene.add(layer.clone() as Arc<dyn RenderNode>);

    const RECT_WIDTH: f32 = 100.0;
    const RECT_HEIGHT: f32 = 100.0;

    layer.set_anchor_point(Point { x: 0.5, y: 0.5 }, None);
    layer.set_size(
        Point {
            x: RECT_WIDTH as f64,
            y: RECT_HEIGHT as f64,
        },
        None,
    );

    layer.set_position(Point { x: 160.0, y: 160.0 }, None);

    engine.update(0.0);

    let point = Point { x: 200.0, y: 200.0 };

    let node = engine.layer_at(point);

    assert!(node.is_some());

    layer.set_rotation(
        Point3d {
            x: 0.0,
            y: 0.0,
            z: PI / 4.0,
        },
        None,
    );
    engine.update(0.0);
    let node = engine.layer_at(point);
    assert!(node.is_none());
}

#[test]
pub fn debug_scene_node_contains() {
    let mut renderer = layers::renderer::skia_image::SkiaImageRenderer::new(
        500,
        500,
        "test_scene_node_contains.png".to_string(),
    );
    let engine = layers::engine::Engine::create();
    let layer = ModelLayer::create();
    let layer2 = ModelLayer::create();
    let nodeid2 = engine.scene.add(layer2.clone() as Arc<dyn RenderNode>);
    let nodeid = engine.scene.add(layer.clone() as Arc<dyn RenderNode>);

    const RECT_WIDTH: f32 = 100.0;
    const RECT_HEIGHT: f32 = 100.0;
    layer.set_anchor_point(Point { x: 0.5, y: 0.5 }, None);
    layer.set_size(
        Point {
            x: RECT_WIDTH as f64,
            y: RECT_HEIGHT as f64,
        },
        None,
    );
    layer2.set_anchor_point(Point { x: 0.5, y: 0.5 }, None);
    layer2.set_size(
        Point {
            x: (RECT_WIDTH / 2.0) as f64,
            y: (RECT_HEIGHT / 2.0) as f64,
        },
        None,
    );
    layer.set_border_corner_radius(BorderRadius::new_single(5.0), None);
    layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba(0.6, 0.5, 1.0, 1.0),
        },
        None,
    );
    layer2.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba(0.9, 0.5, 0.0, 1.0),
        },
        None,
    );
    layer.set_position(Point { x: 160.0, y: 160.0 }, None);
    layer2.set_position(Point { x: 180.0, y: 180.0 }, None);
    engine.update(0.0);

    let point = Point { x: 200.0, y: 200.0 };
    let node = engine.layer_at(point);

    assert!(node.is_some());

    assert_eq!(node.unwrap().0, nodeid.0);
    layer.set_rotation(
        Point3d {
            x: 0.0,
            y: 0.0,
            z: PI / 4.0,
        },
        None,
    );
    engine.update(0.0);

    renderer.draw_scene(&engine.scene);
    let surface = renderer.surface();
    let canvas = surface.canvas();

    let matrix = layer.transform();
    let inverse = matrix.invert().unwrap();

    let mapped_point = inverse.map_point(point);

    canvas.draw_circle(
        point,
        2.0,
        &Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None),
    );

    canvas.draw_circle(
        mapped_point,
        2.0,
        &Paint::new(Color4f::new(0.0, 1.0, 0.3, 1.0), None),
    );

    renderer.save();
    let node = engine.layer_at(Point { x: 200.0, y: 200.0 });
    assert!(node.is_some());
    assert_eq!(node.unwrap().0, nodeid2.0);
}
