//! Headless demo of `render_subtrees_to_buffers`.
//!
//! Builds a desktop-composition scene — wallpaper / window N / window N+1 (with
//! a frosted titlebar) / overlay — renders each plane into its OWN buffer the way
//! an external compositor would consume them, and writes:
//!   - `out/plane_{z}_{key}.png`     one independent buffer per subtree, and
//!   - `out/composited.png`          the buffers re-stacked in z-order.
//!
//! The frosted titlebar of window N+1 is a `BackgroundBlur` layer: even though it
//! lives in its own isolated buffer, its blur samples the composition of the
//! wallpaper + window N below it (cross-buffer vibrancy).
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

    // Render every plane into its own buffer, bottom -> top.
    let roots = [wallpaper.id, window_n.id, window_n1.id, overlay.id];
    let names = ["wallpaper", "window_n", "window_n1", "overlay"];
    let buffers = render_subtrees_to_buffers(engine.scene(), &roots, None);

    println!("rendered {} subtree buffers:", buffers.len());
    for (b, name) in buffers.iter().zip(names) {
        println!(
            "  z{} {:<10} origin=({:.0},{:.0}) size={}x{}",
            b.z_index, name, b.origin.x, b.origin.y, b.size.width, b.size.height
        );
        save_png(&b.image, &format!("out/plane_{}_{}.png", b.z_index, name));
    }

    // Re-stack the independent buffers the way an external compositor would, to
    // confirm the result is a coherent desktop with working cross-buffer blur.
    let mut composite =
        skia::surfaces::raster_n32_premul((W as i32, H as i32)).expect("composite surface");
    {
        let canvas = composite.canvas();
        canvas.clear(skia::Color::BLACK);
        for b in &buffers {
            canvas.draw_image(&b.image, (b.origin.x, b.origin.y), None);
        }
    }
    save_png(&composite.image_snapshot(), "out/composited.png");
    println!("wrote out/composited.png");
}
