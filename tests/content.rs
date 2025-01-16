use lay_rs::prelude::Layer;
// use lay_rs::types::Point;
use lay_rs::types::Size;

pub fn setup_layer(layer: &Layer) {
    layer.set_size(Size::points(100.0, 100.0), None);
    layer.set_position((100.0, 100.0), None);
    layer.set_background_color(lay_rs::types::Color::new_hex("#4043d1"), None);
    layer.set_border_corner_radius(20.0, None);
}
// #[test]
// pub fn load_content_from_encoded_buffer() {
//     let mut renderer = lay_rs::renderer::skia_image::SkiaImageRenderer::new(
//         500,
//         500,
//         "./tests/content/test_scene_node_content_encoded.png".to_string(),
//     );

//     let engine = LayersEngine::new(1000.0, 1000.0);
//     let layer = engine.new_layer();
//     setup_layer(&layer);
//     let _id = engine.add_layer(layer.clone());

//     // let data = std::fs::read("./assets/fill.png").unwrap();
//     // layer.set_content_from_data_encoded(&data);

//     engine.update(0.0);
//     renderer.draw_scene(engine.scene(), engine.scene_root().unwrap(), None);
//     renderer.save();
// }

// #[test]
// pub fn load_content_from_decoded_buffer() {
//     let mut renderer = lay_rs::renderer::skia_image::SkiaImageRenderer::new(
//         1000,
//         1000,
//         "./tests/content/test_scene_node_content_decoded.png".to_string(),
//     );

//     let engine = Engine::create();
//     let scene = engine.scene.clone();
//     let layer = create_layer();
//     let _id = engine.scene.add(layer.clone() as Arc<dyn RenderNode>);

//     let image = image::open("./assets/fill.png").unwrap();
//     let w = image.width();
//     let h = image.height();

//     // decode image into a buffer
//     let image = image.into_rgba8();
//     let data = image.into_vec();
//     layer.set_content_from_data_raster_rgba8(data, w as i32, h as i32);

//     engine.update(0.0);
//     renderer.draw_scene(&engine.scene);
//     renderer.save();
// }
