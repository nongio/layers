//! Fading a `BackgroundBlur` layer must be energy-preserving: mid-fade the
//! region may never be darker than a linear interpolation between the
//! untouched backdrop and the fully-faded-in panel.
//!
//! Two paths are checked, mirroring how a compositor consumes the scene:
//!  - "plane": the subtree renders into its own transparent buffer with an
//!    external (pre-blurred) backdrop seed, and the caller composites that
//!    buffer over the lower planes — Otto's KMS scanout path.
//!  - "direct": the subtree paints straight onto the already-drawn lower
//!    content, doing a real in-place backdrop blur.

use layers::{
    drawing::{render_node_tree, ExternalBackdrop},
    prelude::*,
    skia, taffy,
    types::{Color, Size},
};

const W: i32 = 200;
const H: i32 = 200;
/// Lower-plane luminance ("what is behind the panel").
const LOWER: u8 = 153;
/// Panel material: white at alpha 60/255.
const MAT_A: u8 = 60;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

fn solid_image(v: u8) -> skia::Image {
    let mut surface = skia::surfaces::raster_n32_premul((W, H)).unwrap();
    surface
        .canvas()
        .clear(skia::Color4f::new(v as f32 / 255.0, v as f32 / 255.0, v as f32 / 255.0, 1.0));
    surface.image_snapshot()
}

/// Scene: root (transparent) → panel (BackgroundBlur, thin white material).
fn scene(opacity: f32) -> (std::sync::Arc<Engine>, NodeRef) {
    let engine = Engine::create(W as f32, H as f32);
    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(W as f32, H as f32), None);
    root.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let panel = engine.new_layer();
    engine.append_layer(&panel, Some(root.id)).unwrap();
    panel.set_layout_style(absolute());
    panel.set_position((50.0, 50.0), None);
    panel.set_size(Size::points(100.0, 100.0), None);
    panel.set_background_color(Color::new_rgba255(255, 255, 255, MAT_A), None);
    panel.set_blend_mode(BlendMode::BackgroundBlur);
    panel.set_opacity(opacity, None);

    engine.update(0.016);
    let id = root.id;
    (engine, id)
}

fn read_center(surface: &mut skia::Surface) -> [u8; 4] {
    let image = surface.image_snapshot();
    let info = skia::ImageInfo::new(
        (1, 1),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Unpremul,
        None,
    );
    let mut px = [0u8; 4];
    assert!(image.read_pixels(
        &info,
        &mut px,
        4,
        skia::IPoint::new(W / 2, H / 2),
        skia::image::CachingHint::Disallow,
    ));
    px
}

/// Render the subtree into its own transparent buffer with a pre-blurred
/// external backdrop, then composite that buffer over the lower plane.
fn plane_path(opacity: f32) -> [u8; 4] {
    let (engine, root) = scene(opacity);
    let lower = solid_image(LOWER);

    let mut plane = skia::surfaces::raster_n32_premul((W, H)).unwrap();
    {
        let canvas = plane.canvas();
        canvas.clear(skia::Color::TRANSPARENT);
        let scene_ref = engine.scene();
        scene_ref.with_arena(|arena| {
            scene_ref.with_renderable_arena(|renderables| {
                render_node_tree(
                    root,
                    arena,
                    renderables,
                    canvas,
                    1.0,
                    None,
                    None,
                    Some(ExternalBackdrop {
                        image: &lower,
                        scale: 1.0,
                        blurred: true,
                        raw_image: None,
                    }),
                );
            });
        });
    }
    let plane_img = plane.image_snapshot();

    let mut out = skia::surfaces::raster_n32_premul((W, H)).unwrap();
    {
        let canvas = out.canvas();
        canvas.draw_image(&lower, (0, 0), None);
        canvas.draw_image(&plane_img, (0, 0), None);
    }
    read_center(&mut out)
}

/// Paint the subtree straight onto the already-drawn lower content.
fn direct_path(opacity: f32) -> [u8; 4] {
    let (engine, root) = scene(opacity);
    let lower = solid_image(LOWER);

    let mut out = skia::surfaces::raster_n32_premul((W, H)).unwrap();
    {
        let canvas = out.canvas();
        canvas.draw_image(&lower, (0, 0), None);
        let scene_ref = engine.scene();
        scene_ref.with_arena(|arena| {
            scene_ref.with_renderable_arena(|renderables| {
                render_node_tree(root, arena, renderables, canvas, 1.0, None, None, None);
            });
        });
    }
    read_center(&mut out)
}

/// The value a linear fade between the untouched lower plane and the
/// fully-faded-in panel would produce at the panel centre.
fn expected(opacity: f32) -> f32 {
    let ma = MAT_A as f32 / 255.0;
    let full = ma * 255.0 + (1.0 - ma) * LOWER as f32;
    LOWER as f32 + opacity * (full - LOWER as f32)
}

#[test]
fn blur_fade_is_energy_preserving() {
    for &o in &[0.25f32, 0.5, 0.75, 1.0] {
        let want = expected(o);
        let plane = plane_path(o);
        let direct = direct_path(o);
        println!(
            "opacity {o}: expected ~{want:.1} plane={plane:?} direct={direct:?}"
        );
        assert!(
            (plane[0] as f32 - want).abs() <= 6.0,
            "plane path at opacity {o}: got {plane:?}, expected ~{want:.1}"
        );
        // The direct path runs the real backdrop filter, whose vibrancy tone map
        // lifts contrast a little, so only the "never dips dark" half is asserted.
        assert!(
            direct[0] as f32 >= want - 6.0,
            "direct path at opacity {o}: got {direct:?}, darker than the ~{want:.1} linear fade"
        );
    }
}
