use lay_rs::{
    prelude::*,
    renderer::skia_image::SkiaImageRenderer,
    types::{BorderRadius, Color, Size},
};

// Visual test for image caching
#[test]
pub fn image_cache() {
    let engine = Engine::create(1000.0, 1000.0);

    let layer = engine.new_layer();

    engine.add_layer(&layer);

    layer.set_position((50.0, 50.0), None);
    layer.set_size(Size::points(200.0, 200.0), None);
    layer.set_background_color(Color::new_hex("#ffffff"), None);
    layer.set_border_color(Color::new_hex("#000000"), None);
    layer.set_border_width(2.0, None);
    layer.set_border_corner_radius(BorderRadius::new_single(20.0), None);

    // layer.set_image_cache(true);

    // Create a child layer
    let child_layer = engine.new_layer();
    child_layer.set_opacity(0.5, None);

    child_layer.set_position((75.0, 75.0), None);
    child_layer.set_background_color(Color::new_hex("#ff0000"), None);
    child_layer.set_size(Size::points(50.0, 50.0), None);
    child_layer.set_border_corner_radius(BorderRadius::new_single(3.0), None);
    child_layer.set_border_color(Color::new_hex("#000000"), None);
    child_layer.set_border_width(2.0, None);

    engine.append_layer(&child_layer, Some(layer.id));

    engine.update(0.01);

    // save the image
    {
        let mut renderer = SkiaImageRenderer::new(1000, 1000, "tests/image_cache/render.png");
        renderer.draw_scene(engine.scene(), engine.scene_root().unwrap(), None);
        renderer.save();
    }
    let image_base = image::open("tests/image_cache/base.png")
        .expect("Could not find test-image")
        .into_rgb8();
    let image_two = image::open("tests/image_cache/render.png")
        .expect("Could not find test-image")
        .into_rgb8();
    let result = image_compare::rgb_hybrid_compare(&image_base, &image_two)
        .expect("Images had different dimensions");

    assert_eq!(result.score, 1.0);

    // save the image
    layer.set_image_cached(true);
    engine.update(0.01);
    {
        let mut renderer = SkiaImageRenderer::new(1000, 1000, "tests/image_cache/render_image.png");
        renderer.draw_scene(engine.scene(), engine.scene_root().unwrap(), None);
        renderer.save();
    }

    let image_three = image::open("tests/image_cache/render_image.png")
        .expect("Could not find test-image")
        .into_rgb8();
    let result = image_compare::rgb_hybrid_compare(&image_base, &image_three)
        .expect("Images had different dimensions");

    assert_eq!(result.score, 1.0);

    // save the image
    child_layer.set_image_cached(true);
    engine.update(0.01);
    {
        let mut renderer =
            SkiaImageRenderer::new(1000, 1000, "tests/image_cache/render_image_child.png");
        renderer.draw_scene(engine.scene(), engine.scene_root().unwrap(), None);
        renderer.save();
    }

    let image_four = image::open("tests/image_cache/render_image_child.png")
        .expect("Could not find test-image")
        .into_rgb8();
    let result = image_compare::rgb_hybrid_compare(&image_base, &image_four)
        .expect("Images had different dimensions");

    assert_eq!(result.score, 1.0);
}
