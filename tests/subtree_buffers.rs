use layers::{
    prelude::*,
    skia, taffy,
    types::{Color, Size},
};

/// Encode a subtree buffer to a PNG on disk and return its RGBA pixels.
fn buffer_rgba(buffer: &SubtreeBuffer, path: &str) -> image::RgbaImage {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let data = buffer
        .image
        .encode(None, skia::EncodedImageFormat::PNG, None)
        .expect("encode buffer to PNG");
    std::fs::write(path, data.as_bytes()).expect("write buffer PNG");
    image::open(path).expect("reopen buffer PNG").into_rgba8()
}

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

// Renders two overlapping sibling subtrees into separate buffers and checks that:
//  1. one buffer per requested root is produced, in z-order,
//  2. the top buffer stays transparent outside its blur shape (independence),
//  3. the blur shape samples the *lower* subtree (cross-buffer vibrancy):
//     a frosted region over a red plane comes out reddish, not transparent.
#[test]
pub fn subtree_buffers_cross_buffer_blur() {
    let engine = Engine::create(600.0, 600.0);

    // Explicit root container; bottom and top are siblings under it.
    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(600.0, 600.0), None);

    // Bottom plane: opaque red square at (100,100) 200x200.
    let bottom = engine.new_layer();
    engine.append_layer(&bottom, Some(root.id)).unwrap();
    bottom.set_layout_style(absolute());
    bottom.set_position((100.0, 100.0), None);
    bottom.set_size(Size::points(200.0, 200.0), None);
    bottom.set_background_color(Color::new_hex("#ff0000"), None);

    // Top plane: transparent container at (150,150) 200x200 ...
    let top = engine.new_layer();
    engine.append_layer(&top, Some(root.id)).unwrap();
    top.set_layout_style(absolute());
    top.set_position((150.0, 150.0), None);
    top.set_size(Size::points(200.0, 200.0), None);
    top.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    // ... holding a BackgroundBlur frosted square in its top-left quadrant.
    let frost = engine.new_layer();
    engine.append_layer(&frost, Some(top.id)).unwrap();
    frost.set_layout_style(absolute());
    frost.set_position((0.0, 0.0), None);
    frost.set_size(Size::points(100.0, 100.0), None);
    frost.set_background_color(Color::new_rgba255(255, 255, 255, 20), None);
    frost.set_blend_mode(BlendMode::BackgroundBlur);

    engine.update(0.016);

    let buffers = render_subtrees_to_buffers(engine.scene(), &[bottom.id, top.id], None);

    // 1. one buffer per root, in z-order, placed at their global origins.
    assert_eq!(buffers.len(), 2, "expected one buffer per subtree root");
    assert_eq!(buffers[0].z_index, 0);
    assert_eq!(buffers[1].z_index, 1);
    assert_eq!((buffers[0].origin.x, buffers[0].origin.y), (100.0, 100.0));
    assert_eq!((buffers[1].origin.x, buffers[1].origin.y), (150.0, 150.0));

    let bottom_px = buffer_rgba(&buffers[0], "tests/subtree_buffers/bottom.png");
    let top_px = buffer_rgba(&buffers[1], "tests/subtree_buffers/top.png");

    // Bottom buffer is the opaque red plane.
    let bp = bottom_px.get_pixel(50, 50);
    assert!(
        bp[3] > 250 && bp[0] > 200,
        "bottom buffer should be opaque red, got {:?}",
        bp
    );

    // 2. Independence: a point in the top buffer well outside the frost shape
    //    (buffer-local ~150,150 == global ~300,300) must be fully transparent.
    let outside = top_px.get_pixel(150, 150);
    assert_eq!(
        outside[3], 0,
        "top buffer must be transparent outside blur, got {:?}",
        outside
    );

    // 3. Cross-buffer vibrancy: center of the frost shape (buffer-local ~50,50 ==
    //    global ~200,200, inside the red plane) must be a non-transparent,
    //    reddish blur of the LOWER subtree — proving it sampled the accumulator.
    let frosted = top_px.get_pixel(50, 50);
    assert!(
        frosted[3] > 0,
        "frost region must not be transparent, got {:?}",
        frosted
    );
    assert!(
        frosted[0] as i32 > frosted[1] as i32 && frosted[0] as i32 > frosted[2] as i32,
        "frost region should be reddish (blurred red backdrop), got {:?}",
        frosted
    );
}

// Proves the backdrop is actually *blurred* (not just sampled): a frost layer
// straddling a sharp red|blue seam in the lower subtree must show a blended
// purple at the seam, where a non-blurred copy would stay pure red or blue.
#[test]
pub fn subtree_buffers_backdrop_is_blurred() {
    let engine = Engine::create(600.0, 600.0);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(600.0, 600.0), None);

    // Lower subtree: red (left) and blue (right) halves meeting at x = 300.
    let backdrop = engine.new_layer();
    engine.append_layer(&backdrop, Some(root.id)).unwrap();
    backdrop.set_layout_style(absolute());
    backdrop.set_position((100.0, 100.0), None);
    backdrop.set_size(Size::points(400.0, 200.0), None);
    backdrop.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let red = engine.new_layer();
    engine.append_layer(&red, Some(backdrop.id)).unwrap();
    red.set_layout_style(absolute());
    red.set_position((0.0, 0.0), None);
    red.set_size(Size::points(200.0, 200.0), None);
    red.set_background_color(Color::new_hex("#ff0000"), None);

    let blue = engine.new_layer();
    engine.append_layer(&blue, Some(backdrop.id)).unwrap();
    blue.set_layout_style(absolute());
    blue.set_position((200.0, 0.0), None);
    blue.set_size(Size::points(200.0, 200.0), None);
    blue.set_background_color(Color::new_hex("#0000ff"), None);

    // Upper subtree: a frost layer centered on the seam (global x = 300).
    let frame = engine.new_layer();
    engine.append_layer(&frame, Some(root.id)).unwrap();
    frame.set_layout_style(absolute());
    frame.set_position((250.0, 120.0), None);
    frame.set_size(Size::points(100.0, 160.0), None);
    frame.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let frost = engine.new_layer();
    engine.append_layer(&frost, Some(frame.id)).unwrap();
    frost.set_layout_style(absolute());
    frost.set_position((0.0, 0.0), None);
    frost.set_size(Size::points(100.0, 160.0), None);
    frost.set_background_color(Color::new_rgba255(255, 255, 255, 10), None);
    frost.set_blend_mode(BlendMode::BackgroundBlur);

    engine.update(0.016);

    let buffers = render_subtrees_to_buffers(engine.scene(), &[backdrop.id, frame.id], None);
    let frost_px = buffer_rgba(&buffers[1], "tests/subtree_buffers/seam.png");

    // The frame buffer origin is global (250,120); the seam is at global x=300,
    // i.e. buffer-local x=50. Sample the middle of the frost there.
    let seam = frost_px.get_pixel(50, 80);
    // A 40px-sigma blur of a red|blue seam yields meaningful red AND blue at the
    // boundary — impossible from a sharp (unblurred) backdrop, which is one or
    // the other.
    assert!(
        seam[0] > 30 && seam[2] > 30,
        "seam should mix red and blue (proof of blur), got {:?}",
        seam
    );
}
