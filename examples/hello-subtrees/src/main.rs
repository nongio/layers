//! Headless demo of `render_subtree_to_buffer` — the per-plane API a KMS/DRM
//! compositor would drive.
//!
//! Builds a desktop-composition scene (wallpaper / window N / frosted glass
//! window N+1 / frosted overlay) and renders each plane into its OWN buffer, one
//! at a time, bottom -> top. The caller owns plane order and compositing: it
//! keeps a running backdrop (the stack of lower planes) and passes it to each
//! plane so `BackgroundBlur` layers can bake a blur of the real content behind
//! them — cross-buffer vibrancy — even though every plane is an independent
//! buffer.
//!
//! It renders TWO frames to show the per-subtree cache: nothing changes between
//! them, so every plane reports `from_cache == true` the second time. Writes:
//!   - `out/plane_{z}_{key}.png`   one independent buffer per subtree, and
//!   - `out/composited.png`        the planes the caller stacked (== KMS output).
//!
//! Run with: `cargo run -p hello-subtrees`

use layers::{
    prelude::*,
    skia, taffy,
    types::{Color, Size},
};

const W: f32 = 900.0;
const H: f32 = 600.0;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

fn save_png(image: &skia::Image, path: &str) {
    let data = image
        .encode(None, skia::EncodedImageFormat::PNG, None)
        .expect("encode PNG");
    std::fs::write(path, data.as_bytes()).expect("write PNG");
}

fn window(engine: &Engine, parent: NodeRef, x: f32, y: f32, color: Color) -> Layer {
    let win = engine.new_layer();
    engine.append_layer(&win, Some(parent)).unwrap();
    win.set_layout_style(absolute());
    win.set_position((x, y), None);
    win.set_size(Size::points(360.0, 260.0), None);
    win.set_background_color(color, None);
    win
}

fn main() {
    std::fs::create_dir_all("out").ok();

    let engine = Engine::create(W, H);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(W, H), None);

    // --- Wallpaper: bright vertical stripes (HIGH-FREQUENCY content), so the
    //     blur has sharp edges to visibly smear. Blurring a flat color shows
    //     nothing; blurring stripes turns them into a smooth gradient. ---
    let wallpaper = engine.new_layer();
    engine.append_layer(&wallpaper, Some(root.id)).unwrap();
    wallpaper.set_layout_style(absolute());
    wallpaper.set_position((0.0, 0.0), None);
    wallpaper.set_size(Size::points(W, H), None);
    wallpaper.set_background_color(Color::new_hex("#101010"), None);

    let stripe_colors = [
        "#ff3b30", "#ff9500", "#ffcc00", "#34c759", "#00c7be", "#30b0c7", "#007aff", "#5856d6",
        "#af52de", "#ff2d55",
    ];
    let stripe_w = W / stripe_colors.len() as f32;
    for (i, hex) in stripe_colors.iter().enumerate() {
        let stripe = engine.new_layer();
        engine.append_layer(&stripe, Some(wallpaper.id)).unwrap();
        stripe.set_layout_style(absolute());
        stripe.set_position((i as f32 * stripe_w, 0.0), None);
        stripe.set_size(Size::points(stripe_w, H), None);
        stripe.set_background_color(Color::new_hex(hex), None);
    }

    // --- Window N: an opaque app window. ---
    let window_n = window(&engine, root.id, 90.0, 300.0, Color::new_hex("#2ea043"));

    // --- Window N+1: a FROSTED GLASS window (whole body is BackgroundBlur with
    //     almost no tint), overlapping the stripes and window N. Its buffer is
    //     rendered separately, yet its blur samples the composition behind it. ---
    let window_n1 = engine.new_layer();
    engine.append_layer(&window_n1, Some(root.id)).unwrap();
    window_n1.set_layout_style(absolute());
    window_n1.set_position((250.0, 120.0), None);
    window_n1.set_size(Size::points(420.0, 300.0), None);
    window_n1.set_background_color(Color::new_rgba255(255, 255, 255, 8), None);
    window_n1.set_blend_mode(BlendMode::BackgroundBlur);

    // --- Overlay: a frosted top bar (e.g. a menu/notification bar). ---
    let overlay = engine.new_layer();
    engine.append_layer(&overlay, Some(root.id)).unwrap();
    overlay.set_layout_style(absolute());
    overlay.set_position((120.0, 24.0), None);
    overlay.set_size(Size::points(660.0, 72.0), None);
    overlay.set_background_color(Color::new_rgba255(255, 255, 255, 10), None);
    overlay.set_blend_mode(BlendMode::BackgroundBlur);

    engine.update(0.016);

    // The plane stack, bottom -> top. This is the only ordering the engine needs;
    // it is owned by the caller (here) the way a KMS compositor owns its planes.
    let planes = [
        (wallpaper.id, "wallpaper"),
        (window_n.id, "window_n"),
        (window_n1.id, "window_n1"),
        (overlay.id, "overlay"),
    ];

    // `backdrops[i]` is the cumulative composite of planes BELOW plane `i` — the
    // backdrop fed to plane `i`. We persist these across frames and only refresh
    // a snapshot when a lower plane actually changed, so an idle blur plane keeps
    // receiving the *same* backdrop image and therefore hits the engine cache
    // (the engine keys blur planes on the backdrop's identity).
    let mut backdrops: Vec<Option<skia::Image>> = vec![None; planes.len() + 1];

    // One "frame": render each plane on its own, feeding each the composite of
    // the planes below it. Returns the stacked composite (what KMS would scan out).
    let mut render_frame = |engine: &Engine, label: &str| -> skia::Image {
        let mut acc =
            skia::surfaces::raster_n32_premul((W as i32, H as i32)).expect("backdrop surface");
        acc.canvas().clear(skia::Color::BLACK);
        let mut stack_dirty = false;

        for (i, (root, name)) in planes.iter().enumerate() {
            let buf = engine
                .render_subtree(*root, backdrops[i].as_ref(), None)
                .expect("plane buffer");
            stack_dirty |= !buf.from_cache;

            println!(
                "[{label}] {name:<10} origin=({:.0},{:.0}) size={}x{} cached={}",
                buf.origin.x, buf.origin.y, buf.size.width, buf.size.height, buf.from_cache
            );
            save_png(&buf.image, &format!("out/plane_{}_{}.png", i, name));

            // Fold this plane into the running composite, then publish it as the
            // backdrop for the next plane up — but only re-snapshot when the stack
            // below has changed, so unchanged planes keep a stable backdrop.
            acc.canvas()
                .draw_image(&buf.image, (buf.origin.x, buf.origin.y), None);
            if stack_dirty || backdrops[i + 1].is_none() {
                backdrops[i + 1] = Some(acc.image_snapshot());
            }
        }
        acc.image_snapshot()
    };

    // Frame 1: everything renders fresh.
    let composited = render_frame(&engine, "frame 1");
    save_png(&composited, "out/composited.png");

    // Frame 2: nothing changed, so every plane comes straight from the cache.
    println!("--- re-rendering with no changes (expect cached=true) ---");
    render_frame(&engine, "frame 2");

    println!("wrote out/composited.png");
}
