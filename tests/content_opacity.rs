//! A layer whose pixels come from `set_draw_content` must honour `opacity`.
//!
//! Background colour, shadow and border have always scaled their paint alpha,
//! but content was played onto the canvas unscaled — so a layer that is
//! *nothing but* content (a mirror, above all) stayed fully opaque through an
//! entire fade and then vanished the moment it was hidden.

#[cfg(test)]
mod tests {
    use layers::prelude::*;
    use layers::skia;

    /// Render one full-bleed red content layer at `opacity` over white and
    /// return the resulting red channel value at the centre.
    fn render_at(opacity: f32) -> u8 {
        let engine = Engine::create(100.0, 100.0);
        let layer = engine.new_layer();
        layer.set_size(layers::types::Size::points(100.0, 100.0), None);
        engine.add_layer(&layer).unwrap();

        layer.set_draw_content(|canvas: &skia::Canvas, w: f32, h: f32| {
            let mut paint = skia::Paint::new(skia::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
            paint.set_anti_alias(false);
            let r = skia::Rect::from_xywh(0.0, 0.0, w, h);
            canvas.draw_rect(r, &paint);
            r
        });
        // The case that matters: a mirror sets `picture_cached(false)`, which
        // skips the cached-picture path (that one already honours opacity by
        // passing a paint) and lands in `draw_layer`.
        layer.set_picture_cached(false);
        layer.set_opacity(opacity, None);
        engine.update(0.016);

        let mut surface = skia::surfaces::raster_n32_premul((100, 100)).unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia::Color::WHITE);
        layers::prelude::draw_scene(canvas, engine.scene(), engine.scene_root().unwrap());

        let image = surface.image_snapshot();
        let mut pixels = vec![0u8; 100 * 100 * 4];
        let info = skia::ImageInfo::new(
            (100, 100),
            skia::ColorType::RGBA8888,
            skia::AlphaType::Premul,
            None,
        );
        assert!(image.read_pixels(
            &info,
            &mut pixels,
            100 * 4,
            (0, 0),
            skia::image::CachingHint::Allow
        ));
        // green channel: white background (255) shows through as content fades
        pixels[(50 * 100 + 50) * 4 + 1]
    }

    #[test]
    fn content_honours_opacity() {
        let opaque = render_at(1.0);
        let half = render_at(0.5);
        let clear = render_at(0.0);

        // Fully opaque red over white: no green left.
        assert_eq!(
            opaque, 0,
            "opaque content should fully cover the background"
        );
        // Fully transparent: the white background is untouched.
        assert_eq!(clear, 255, "transparent content should not be drawn at all");
        // Half: the background must be blending through. Before the fix this
        // was 0 — content ignored opacity entirely and rendered fully opaque.
        assert!(
            half > 60 && half < 200,
            "content at opacity 0.5 should blend with the background, got green={half}"
        );
    }
}
