//! A fading layer keeps the surface's subpixel layout for its text, and still
//! fades exactly as it did.
//!
//! A fade is a group, and Skia gives a group greyscale text unless told it
//! may keep the layout. On a surface that declares one — which is opaque —
//! the group starts from the surface and keeps it, so text does not visibly
//! change weight the moment a menu or a window finishes fading in.

use layers::{prelude::*, skia, taffy, types::Size};

const W: i32 = 200;
const H: i32 = 120;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

fn rgb_surface() -> skia::Surface {
    let info = skia::ImageInfo::new_n32_premul((W, H), None);
    let props = skia::SurfaceProps::new(Default::default(), skia::PixelGeometry::RGBH);
    skia::surfaces::raster(&info, None, Some(&props)).unwrap()
}

fn pixels(surface: &mut skia::Surface) -> Vec<u8> {
    let info = skia::ImageInfo::new(
        (W, H),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Premul,
        None,
    );
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    assert!(surface.read_pixels(&info, &mut pixels, (W * 4) as usize, (0, 0)));
    pixels
}

/// Render a layer whose content `draw` paints, at `opacity`, over white on a
/// surface laid out RGB.
fn render(
    opacity: f32,
    picture_cached: bool,
    draw: impl Fn(&skia::Canvas, f32, f32) -> skia::Rect + Send + Sync + 'static,
) -> Vec<u8> {
    let engine = Engine::create(W as f32, H as f32);
    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_size(Size::points(W as f32, H as f32), None);

    let layer = engine.new_layer();
    engine.append_layer(&layer, Some(root.id)).unwrap();
    layer.set_layout_style(absolute());
    layer.set_size(Size::points(W as f32, H as f32), None);
    layer.set_draw_content(draw);
    layer.set_picture_cached(picture_cached);
    layer.set_opacity(opacity, None);
    engine.update(0.016);

    let mut surface = rgb_surface();
    surface.canvas().clear(skia::Color::WHITE);
    draw_scene(surface.canvas(), engine.scene(), root.id);
    pixels(&mut surface)
}

fn text(canvas: &skia::Canvas, w: f32, h: f32) -> skia::Rect {
    let typeface = skia::FontMgr::new()
        .legacy_make_typeface(None, skia::FontStyle::normal())
        .expect("a system typeface");
    let mut font = skia::Font::from_typeface(typeface, 24.0);
    font.set_edging(skia::font::Edging::SubpixelAntiAlias);
    let paint = skia::Paint::new(skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
    canvas.draw_str("Hamburgefonts", (8.0, 60.0), &font, &paint);
    skia::Rect::from_wh(w, h)
}

fn black(canvas: &skia::Canvas, w: f32, h: f32) -> skia::Rect {
    let r = skia::Rect::from_wh(w, h);
    canvas.draw_rect(
        r,
        &skia::Paint::new(skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None),
    );
    r
}

/// How far apart the colour channels get anywhere: zero for greyscale text.
fn channel_spread(pixels: &[u8]) -> u8 {
    pixels
        .chunks(4)
        .map(|px| px[..3].iter().max().unwrap() - px[..3].iter().min().unwrap())
        .max()
        .unwrap()
}

#[test]
fn text_in_a_fading_layer_keeps_subpixel_coverage() {
    for picture_cached in [false, true] {
        assert!(
            channel_spread(&render(1.0, picture_cached, text)) > 20,
            "picture_cached={picture_cached}: subpixel text at rest"
        );
        let faded = channel_spread(&render(0.6, picture_cached, text));
        assert!(
            faded > 10,
            "picture_cached={picture_cached}: subpixel text mid-fade, got a spread of {faded}"
        );
    }
}

#[test]
fn a_fade_on_a_laid_out_surface_is_the_same_fade() {
    for picture_cached in [false, true] {
        let green = render(0.5, picture_cached, black)[((H / 2 * W + W / 2) * 4 + 1) as usize];
        assert!(
            (124..=131).contains(&green),
            "picture_cached={picture_cached}: black at 0.5 over white leaves green 127, got {green}"
        );
    }
}
