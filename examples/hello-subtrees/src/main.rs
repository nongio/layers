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

    // --- Wallpaper: two colored bands so the blur has something to mix. ---
    let wallpaper = engine.new_layer();
    engine.append_layer(&wallpaper, Some(root.id)).unwrap();
    wallpaper.set_layout_style(absolute());
    wallpaper.set_position((0.0, 0.0), None);
    wallpaper.set_size(Size::points(W, H), None);
    wallpaper.set_background_color(Color::new_hex("#1f6feb"), None);

    let band = engine.new_layer();
    engine.append_layer(&band, Some(wallpaper.id)).unwrap();
    band.set_layout_style(absolute());
    band.set_position((0.0, 0.0), None);
    band.set_size(Size::points(W / 2.0, H), None);
    band.set_background_color(Color::new_hex("#f0883e"), None);

    // --- Window N ---
    let window_n = window(&engine, root.id, 120.0, 120.0, Color::new_hex("#2ea043"));

    // --- Window N+1, overlapping N, with a frosted (BackgroundBlur) titlebar ---
    let window_n1 = window(&engine, root.id, 360.0, 220.0, Color::new_hex("#8957e5"));
    let titlebar = engine.new_layer();
    engine.append_layer(&titlebar, Some(window_n1.id)).unwrap();
    titlebar.set_layout_style(absolute());
    titlebar.set_position((0.0, 0.0), None);
    titlebar.set_size(Size::points(360.0, 56.0), None);
    titlebar.set_background_color(Color::new_rgba255(255, 255, 255, 20), None);
    titlebar.set_blend_mode(BlendMode::BackgroundBlur);

    // --- Overlay: a translucent full-screen frost (e.g. a notification scrim) ---
    let overlay = engine.new_layer();
    engine.append_layer(&overlay, Some(root.id)).unwrap();
    overlay.set_layout_style(absolute());
    overlay.set_position((640.0, 40.0), None);
    overlay.set_size(Size::points(220.0, 120.0), None);
    overlay.set_background_color(Color::new_rgba255(255, 255, 255, 16), None);
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
