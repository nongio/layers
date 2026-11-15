//! A layer's opacity is applied once, whichever path draws it.
//!
//! The paths each applied it on their own terms: a cached picture was
//! recorded with the layer's opacity in it and played back with it again,
//! `draw_layer` multiplied its content by an opacity that already held the
//! layer's own, and an image-cached subtree faded its children inside the
//! cache and again when the cache was drawn. A layer at 0.5 came out at 0.25.
//!
//! The image-cached path needs a GPU surface; `vulkan_image_cache` covers it.

use layers::prelude::*;
use layers::skia;

#[derive(Clone, Copy, Debug)]
enum Fill {
    /// The layer's background colour.
    Background,
    /// What its draw closure paints.
    Content,
}

#[derive(Clone, Copy, Debug)]
enum Path {
    Direct,
    Picture,
}

fn black_layer(engine: &Engine, fill: Fill, path: Path) -> Layer {
    let layer = engine.new_layer();
    layer.set_size(layers::types::Size::points(100.0, 100.0), None);
    match fill {
        Fill::Background => {
            layer.set_background_color(layers::types::Color::new_rgba255(0, 0, 0, 255), None);
        }
        Fill::Content => {
            layer.set_draw_content(|canvas: &skia::Canvas, w: f32, h: f32| {
                let mut paint = skia::Paint::new(skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
                paint.set_anti_alias(false);
                let r = skia::Rect::from_xywh(0.0, 0.0, w, h);
                canvas.draw_rect(r, &paint);
                r
            });
        }
    }
    layer.set_picture_cached(matches!(path, Path::Picture));
    layer
}

/// The green channel at the centre of a 100×100 white surface after drawing
/// `engine`'s scene: 255 × (1 − opacity) for black at `opacity`.
fn green(engine: &Engine) -> u8 {
    engine.update(0.016);
    let mut surface = skia::surfaces::raster_n32_premul((100, 100)).unwrap();
    let canvas = surface.canvas();
    canvas.clear(skia::Color::WHITE);
    draw_scene(canvas, engine.scene(), engine.scene_root().unwrap());

    let mut pixels = vec![0u8; 100 * 100 * 4];
    let info = skia::ImageInfo::new(
        (100, 100),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Premul,
        None,
    );
    assert!(surface.read_pixels(&info, &mut pixels, 100 * 4, (0, 0)));
    pixels[(50 * 100 + 50) * 4 + 1]
}

fn assert_half(what: String, green: u8) {
    assert!(
        (124..=131).contains(&green),
        "{what}: black at 0.5 over white leaves green 127, got {green}"
    );
}

#[test]
fn a_layer_fades_once() {
    for fill in [Fill::Background, Fill::Content] {
        for path in [Path::Direct, Path::Picture] {
            let engine = Engine::create(100.0, 100.0);
            let layer = black_layer(&engine, fill, path);
            engine.add_layer(&layer).unwrap();
            layer.set_opacity(0.5, None);
            assert_half(format!("{fill:?} {path:?}"), green(&engine));
        }
    }
}

#[test]
fn a_child_fades_once_with_its_parent() {
    for path in [Path::Direct, Path::Picture] {
        let engine = Engine::create(100.0, 100.0);
        let parent = engine.new_layer();
        parent.set_size(layers::types::Size::points(100.0, 100.0), None);
        engine.add_layer(&parent).unwrap();
        let child = black_layer(&engine, Fill::Content, path);
        engine.append_layer(&child, Some(parent.id)).unwrap();
        parent.set_opacity(0.5, None);
        assert_half(format!("child {path:?}"), green(&engine));
    }
}
