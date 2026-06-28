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

/// Composite already-rendered lower-plane buffers into a scene-global backdrop
/// image — what a KMS-style caller maintains to feed `render_subtree`.
fn composite_backdrop(buffers: &[&SubtreeBuffer], w: i32, h: i32) -> skia::Image {
    let mut surface = skia::surfaces::raster_n32_premul((w, h)).expect("backdrop surface");
    {
        let canvas = surface.canvas();
        canvas.clear(skia::Color::TRANSPARENT);
        for b in buffers {
            canvas.draw_image(&b.image, (b.origin.x, b.origin.y), None);
        }
    }
    surface.image_snapshot()
}

// Renders two overlapping sibling subtrees one at a time, with the caller
// supplying the backdrop, and checks:
//  1. the top buffer is transparent outside its blur shape (independence),
//  2. the blur shape samples the lower plane the caller passed as backdrop
//     (cross-buffer vibrancy),
//  3. the per-subtree cache returns the buffer untouched when nothing changed,
//     and re-renders when the subtree or the backdrop changes.
#[test]
pub fn subtree_buffers_cross_buffer_blur() {
    let engine = Engine::create(600.0, 600.0);

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

    // Top plane: transparent container at (150,150) holding a BackgroundBlur
    // frosted square in its top-left quadrant.
    let top = engine.new_layer();
    engine.append_layer(&top, Some(root.id)).unwrap();
    top.set_layout_style(absolute());
    top.set_position((150.0, 150.0), None);
    top.set_size(Size::points(200.0, 200.0), None);
    top.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let frost = engine.new_layer();
    engine.append_layer(&frost, Some(top.id)).unwrap();
    frost.set_layout_style(absolute());
    frost.set_position((0.0, 0.0), None);
    frost.set_size(Size::points(100.0, 100.0), None);
    frost.set_background_color(Color::new_rgba255(255, 255, 255, 20), None);
    frost.set_blend_mode(BlendMode::BackgroundBlur);

    engine.update(0.016);

    // Bottom plane first (no backdrop). Then the caller composites it into the
    // backdrop for the top plane.
    let bottom_buf = render_subtree_to_buffer(engine.scene(), bottom.id, None, None).unwrap();
    assert!(!bottom_buf.from_cache, "first render should not be cached");
    assert_eq!((bottom_buf.origin.x, bottom_buf.origin.y), (100.0, 100.0));

    let backdrop = composite_backdrop(&[&bottom_buf], 600, 600);
    let top_buf = render_subtree_to_buffer(engine.scene(), top.id, Some(&backdrop), None).unwrap();
    assert!(!top_buf.from_cache);
    assert_eq!((top_buf.origin.x, top_buf.origin.y), (150.0, 150.0));

    let bottom_px = buffer_rgba(&bottom_buf, "tests/subtree_buffers/bottom.png");
    let top_px = buffer_rgba(&top_buf, "tests/subtree_buffers/top.png");

    let bp = bottom_px.get_pixel(50, 50);
    assert!(
        bp[3] > 250 && bp[0] > 200,
        "bottom buffer should be opaque red, got {:?}",
        bp
    );

    // Independence: outside the frost shape (buffer-local ~150,150) is transparent.
    let outside = top_px.get_pixel(150, 150);
    assert_eq!(
        outside[3], 0,
        "top buffer must be transparent outside blur, got {:?}",
        outside
    );

    // Cross-buffer vibrancy: center of the frost (buffer-local ~50,50 == global
    // ~200,200, inside the red plane) is a non-transparent reddish blur.
    let frosted = top_px.get_pixel(50, 50);
    assert!(
        frosted[3] > 0,
        "frost must not be transparent, got {:?}",
        frosted
    );
    assert!(
        frosted[0] as i32 > frosted[1] as i32 && frosted[0] as i32 > frosted[2] as i32,
        "frost should be reddish (blurred red backdrop), got {:?}",
        frosted
    );

    // --- Caching ---
    // Same subtree, same backdrop image -> cache hit.
    let again = render_subtree_to_buffer(engine.scene(), top.id, Some(&backdrop), None).unwrap();
    assert!(
        again.from_cache,
        "unchanged subtree + backdrop should hit cache"
    );
    let bottom_again = render_subtree_to_buffer(engine.scene(), bottom.id, None, None).unwrap();
    assert!(
        bottom_again.from_cache,
        "unchanged bottom plane should hit cache"
    );

    // A different backdrop image (new snapshot) invalidates the blur plane.
    let backdrop2 = composite_backdrop(&[&bottom_buf], 600, 600);
    let after_backdrop =
        render_subtree_to_buffer(engine.scene(), top.id, Some(&backdrop2), None).unwrap();
    assert!(
        !after_backdrop.from_cache,
        "changed backdrop must re-render the blur plane"
    );

    // Changing the subtree's own content invalidates it too.
    frost.set_background_color(Color::new_rgba255(255, 255, 255, 60), None);
    engine.update(0.016);
    let after_change =
        render_subtree_to_buffer(engine.scene(), top.id, Some(&backdrop2), None).unwrap();
    assert!(
        !after_change.from_cache,
        "changed subtree content must re-render"
    );
}

// Proves the backdrop is actually *blurred* (not just sampled): a frost layer
// straddling a sharp red|blue seam in the backdrop shows a blended purple.
#[test]
pub fn subtree_buffers_backdrop_is_blurred() {
    let engine = Engine::create(600.0, 600.0);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(600.0, 600.0), None);

    let backdrop_layer = engine.new_layer();
    engine.append_layer(&backdrop_layer, Some(root.id)).unwrap();
    backdrop_layer.set_layout_style(absolute());
    backdrop_layer.set_position((100.0, 100.0), None);
    backdrop_layer.set_size(Size::points(400.0, 200.0), None);
    backdrop_layer.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let red = engine.new_layer();
    engine.append_layer(&red, Some(backdrop_layer.id)).unwrap();
    red.set_layout_style(absolute());
    red.set_position((0.0, 0.0), None);
    red.set_size(Size::points(200.0, 200.0), None);
    red.set_background_color(Color::new_hex("#ff0000"), None);

    let blue = engine.new_layer();
    engine.append_layer(&blue, Some(backdrop_layer.id)).unwrap();
    blue.set_layout_style(absolute());
    blue.set_position((200.0, 0.0), None);
    blue.set_size(Size::points(200.0, 200.0), None);
    blue.set_background_color(Color::new_hex("#0000ff"), None);

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

    let backdrop_buf =
        render_subtree_to_buffer(engine.scene(), backdrop_layer.id, None, None).unwrap();
    let backdrop = composite_backdrop(&[&backdrop_buf], 600, 600);
    let frame_buf =
        render_subtree_to_buffer(engine.scene(), frame.id, Some(&backdrop), None).unwrap();
    let frost_px = buffer_rgba(&frame_buf, "tests/subtree_buffers/seam.png");

    // The frame origin is global (250,120); the seam is at global x=300, i.e.
    // buffer-local x=50. A 40px-sigma blur mixes red AND blue there.
    let seam = frost_px.get_pixel(50, 80);
    assert!(
        seam[0] > 30 && seam[2] > 30,
        "seam should mix red and blue (proof of blur), got {:?}",
        seam
    );
}

// Regression: the subtree root passed to render_subtree is ITSELF a
// BackgroundBlur layer (like a frosted-glass window plane), so its own blur
// region never bubbles into its `backdrop_blur_region`. The engine must still
// recognize it has blur and bake the caller-supplied backdrop into it.
#[test]
pub fn subtree_buffers_root_is_blur_layer() {
    let engine = Engine::create(600.0, 600.0);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(600.0, 600.0), None);

    // Lower plane: red|blue halves meeting at global x = 300.
    let backdrop_layer = engine.new_layer();
    engine.append_layer(&backdrop_layer, Some(root.id)).unwrap();
    backdrop_layer.set_layout_style(absolute());
    backdrop_layer.set_position((100.0, 100.0), None);
    backdrop_layer.set_size(Size::points(400.0, 200.0), None);
    backdrop_layer.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let red = engine.new_layer();
    engine.append_layer(&red, Some(backdrop_layer.id)).unwrap();
    red.set_layout_style(absolute());
    red.set_position((0.0, 0.0), None);
    red.set_size(Size::points(200.0, 200.0), None);
    red.set_background_color(Color::new_hex("#ff0000"), None);

    let blue = engine.new_layer();
    engine.append_layer(&blue, Some(backdrop_layer.id)).unwrap();
    blue.set_layout_style(absolute());
    blue.set_position((200.0, 0.0), None);
    blue.set_size(Size::points(200.0, 200.0), None);
    blue.set_background_color(Color::new_hex("#0000ff"), None);

    // The plane root is the frosted-glass layer itself.
    let glass = engine.new_layer();
    engine.append_layer(&glass, Some(root.id)).unwrap();
    glass.set_layout_style(absolute());
    glass.set_position((250.0, 120.0), None);
    glass.set_size(Size::points(100.0, 160.0), None);
    glass.set_background_color(Color::new_rgba255(255, 255, 255, 10), None);
    glass.set_blend_mode(BlendMode::BackgroundBlur);

    engine.update(0.016);

    let backdrop_buf =
        render_subtree_to_buffer(engine.scene(), backdrop_layer.id, None, None).unwrap();
    let backdrop = composite_backdrop(&[&backdrop_buf], 600, 600);
    let glass_buf =
        render_subtree_to_buffer(engine.scene(), glass.id, Some(&backdrop), None).unwrap();
    let glass_px = buffer_rgba(&glass_buf, "tests/subtree_buffers/glass.png");

    // Without the fix the glass buffer is empty (blur read a blank backdrop); with
    // it, the seam at buffer-local x=50 is a blurred red|blue mix.
    let seam = glass_px.get_pixel(50, 80);
    assert!(
        seam[3] > 0 && seam[0] > 30 && seam[2] > 30,
        "blur-root plane should bake a blurred red|blue backdrop, got {:?}",
        seam
    );
}

// The per-subtree cache is only populated on render and is never evicted on its
// own, so retired planes would leak their surfaces. `forget_subtree_buffer` (and
// rendering a now-hidden root) must drop the cached entry: the next render is a
// fresh miss rather than a stale hit.
#[test]
pub fn subtree_buffers_forget_evicts_cache() {
    let engine = Engine::create(300.0, 300.0);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(300.0, 300.0), None);

    let plane = engine.new_layer();
    engine.append_layer(&plane, Some(root.id)).unwrap();
    plane.set_layout_style(absolute());
    plane.set_position((20.0, 20.0), None);
    plane.set_size(Size::points(100.0, 100.0), None);
    plane.set_background_color(Color::new_hex("#00ff00"), None);

    engine.update(0.016);

    // First render is a miss; an immediate re-render hits the cache.
    let first = render_subtree_to_buffer(engine.scene(), plane.id, None, None).unwrap();
    assert!(!first.from_cache, "first render should not be cached");
    let cached = render_subtree_to_buffer(engine.scene(), plane.id, None, None).unwrap();
    assert!(cached.from_cache, "unchanged plane should hit cache");

    // Explicit eviction drops the entry: the next render is a miss again.
    assert!(
        forget_subtree_buffer(plane.id),
        "forget_subtree_buffer should report it evicted an existing entry"
    );
    assert!(
        !forget_subtree_buffer(plane.id),
        "second forget should find nothing to evict"
    );
    let after_forget = render_subtree_to_buffer(engine.scene(), plane.id, None, None).unwrap();
    assert!(
        !after_forget.from_cache,
        "render after forget must be a fresh miss"
    );

    // Rendering a hidden root returns no buffer and evicts its cached entry.
    plane.set_hidden(true);
    engine.update(0.016);
    assert!(
        render_subtree_to_buffer(engine.scene(), plane.id, None, None).is_none(),
        "hidden root should produce no buffer"
    );
    assert!(
        !forget_subtree_buffer(plane.id),
        "hidden-root render should already have evicted the cache entry"
    );
}
