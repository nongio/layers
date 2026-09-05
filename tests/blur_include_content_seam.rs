//! A `BackgroundBlur` layer with `blur_include_content` set, rendered with an
//! external backdrop, must blur same-pass content and the seeded backdrop
//! *together*: the boundary between a window painted earlier in the pass and
//! the seeded wallpaper behind it must come out as one smooth ramp.

use layers::drawing::{render_node_tree, ExternalBackdrop};
use layers::prelude::*;
use layers::skia;
use layers::types::{Color, Size};

fn abs_layer(engine: &Engine, w: f32, h: f32, x: f32, y: f32) -> Layer {
    let l = engine.new_layer();
    l.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    });
    l.set_size(Size::points(w, h), None);
    l.set_position((x, y), None);
    l
}

/// Returns the red channel along y=30, x in 300..500.
pub fn render(include_content: bool, opaque_ground: bool) -> (Vec<u8>, Vec<u8>) {
    let engine = Engine::create(800.0, 400.0);
    let root = abs_layer(&engine, 800.0, 400.0, 0.0, 0.0);
    engine.add_layer(&root).unwrap();
    if opaque_ground {
        let g = abs_layer(&engine, 800.0, 400.0, 0.0, 0.0);
        g.set_background_color(Color::new_rgba(0.0, 0.0, 1.0, 1.0), None);
        engine.append_layer(&g, root.id).unwrap();
    }
    // Window behind: opaque red, left half.
    let back = abs_layer(&engine, 400.0, 400.0, 0.0, 0.0);
    back.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
    engine.append_layer(&back, root.id).unwrap();
    // Window in front, its titlebar frosted.
    let front = abs_layer(&engine, 600.0, 400.0, 200.0, 0.0);
    engine.append_layer(&front, root.id).unwrap();
    let deco = abs_layer(&engine, 600.0, 60.0, 0.0, 0.0);
    deco.set_blend_mode(layers::types::BlendMode::BackgroundBlur);
    deco.set_blur_include_content(include_content);
    engine.append_layer(&deco, front.id).unwrap();
    engine.update(0.016);
    engine.update(0.016);

    // Backdrop = flat blue wallpaper at 1/4 scale, both copies.
    let mut bd = skia::surfaces::raster_n32_premul((200, 100)).unwrap();
    bd.canvas().clear(skia::Color4f::new(0.0, 0.0, 1.0, 1.0));
    let bd_img = bd.image_snapshot();

    let mut surface = skia::surfaces::raster_n32_premul((800, 400)).unwrap();
    surface.canvas().clear(skia::Color::TRANSPARENT);
    let scene = engine.scene();
    scene.with_arena(|arena| {
        scene.with_renderable_arena(|ra| {
            render_node_tree(
                engine.scene_root().unwrap(),
                arena,
                ra,
                surface.canvas(),
                1.0,
                None,
                None,
                Some(ExternalBackdrop {
                    image: &bd_img,
                    scale: 0.25,
                    blurred: true,
                    raw_image: Some(&bd_img),
                }),
            );
        });
    });
    let img = surface.image_snapshot();
    let mut px = vec![0u8; 800 * 400 * 4];
    let info = skia::ImageInfo::new(
        (800, 400),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Premul,
        None,
    );
    assert!(img.read_pixels(
        &info,
        &mut px,
        800 * 4,
        (0, 0),
        skia::image::CachingHint::Allow
    ));
    let reds = (300..500).map(|x| px[(30 * 800 + x) * 4]).collect();
    let alphas = (300..500).map(|x| px[(30 * 800 + x) * 4 + 3]).collect();
    (reds, alphas)
}

fn max_step(v: &[u8]) -> i32 {
    v.windows(2)
        .map(|w| (w[0] as i32 - w[1] as i32).abs())
        .max()
        .unwrap()
}

#[test]
fn seeded_backdrop_and_same_pass_content_blur_as_one() {
    let (reds, alphas) = render(true, false);
    let (reference, _) = render(true, true);
    println!("seeded : {reds:?}\nalphas : {alphas:?}\nopaque : {reference:?}");
    let step = max_step(&reds);
    let ref_step = max_step(&reference);
    assert!(
        step <= ref_step + 2,
        "the seam between window and seeded wallpaper steps by {step} (opaque ground: {ref_step})"
    );
}
