//! Text in a fading `BackgroundBlur` layer keeps the surface's subpixel
//! layout.
//!
//! The fade group starts from the surface, so it is as opaque as the surface
//! is, and per-channel coverage drawn into it lands on the surface intact.
//! Skia would otherwise drop the group's text to greyscale for the length of
//! every fade, and the text would visibly change when the fade ended.

use layers::{prelude::*, skia, taffy, types::Size};

const W: i32 = 200;
const H: i32 = 120;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

/// How far apart the colour channels get anywhere on a frosted panel with
/// black text, faded to `opacity`, over grey, on a surface laid out RGB.
/// Greyscale text keeps every pixel's channels equal.
fn channel_spread(opacity: f32) -> u8 {
    let engine = Engine::create(W as f32, H as f32);
    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_size(Size::points(W as f32, H as f32), None);

    let panel = engine.new_layer();
    engine.append_layer(&panel, Some(root.id)).unwrap();
    panel.set_layout_style(absolute());
    panel.set_position((10.0, 10.0), None);
    panel.set_size(Size::points(180.0, 100.0), None);
    panel.set_background_color(layers::types::Color::new_rgba255(255, 255, 255, 200), None);
    panel.set_blend_mode(BlendMode::BackgroundBlur);
    panel.set_draw_content(|canvas: &skia::Canvas, w: f32, h: f32| {
        let typeface = skia::FontMgr::new()
            .legacy_make_typeface(None, skia::FontStyle::normal())
            .expect("a system typeface");
        let mut font = skia::Font::from_typeface(typeface, 24.0);
        font.set_edging(skia::font::Edging::SubpixelAntiAlias);
        let paint = skia::Paint::new(skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
        canvas.draw_str("Hamburgefonts", (8.0, 60.0), &font, &paint);
        skia::Rect::from_wh(w, h)
    });
    panel.set_opacity(opacity, None);
    engine.update(0.016);

    let info = skia::ImageInfo::new_n32_premul((W, H), None);
    let props = skia::SurfaceProps::new(Default::default(), skia::PixelGeometry::RGBH);
    let mut surface = skia::surfaces::raster(&info, None, Some(&props)).unwrap();
    surface.canvas().clear(skia::Color::from_rgb(128, 128, 128));
    draw_scene(surface.canvas(), engine.scene(), root.id);

    let read = skia::ImageInfo::new(
        (W, H),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Premul,
        None,
    );
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    assert!(surface.image_snapshot().read_pixels(
        &read,
        &mut pixels,
        (W * 4) as usize,
        (0, 0),
        skia::image::CachingHint::Allow
    ));
    pixels
        .chunks(4)
        .map(|px| px[..3].iter().max().unwrap() - px[..3].iter().min().unwrap())
        .max()
        .unwrap()
}

#[test]
fn text_in_a_fading_frosted_layer_keeps_subpixel_coverage() {
    // At rest there is no group: the text lands on the surface directly.
    assert!(channel_spread(1.0) > 20, "subpixel text at rest");
    assert!(
        channel_spread(0.6) > 10,
        "subpixel text mid-fade, got a spread of {}",
        channel_spread(0.6)
    );
}
