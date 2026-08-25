//! Changing `opacity` on a layer that already painted must reach the screen.
//!
//! Content honours `opacity` by fading as a group inside `draw_layer`, so the
//! alpha is baked into the layer's recorded picture. A picture is re-recorded
//! only on `NEEDS_PAINT`, and an opacity change used to raise `NEEDS_LAYOUT`
//! alone — correct while opacity was applied by the paint at replay time,
//! wrong once content started to fade with it. The layer then kept the alpha
//! it happened to be recorded with: a menu popup fading in from 0 stayed a
//! ghost for as long as nothing else dirtied it, and snapped to full opacity
//! on the next unrelated repaint.

#[cfg(test)]
mod tests {
    use layers::prelude::*;
    use layers::skia;

    struct Fixture {
        engine: std::sync::Arc<Engine>,
        layer: Layer,
        surface: skia::Surface,
    }

    impl Fixture {
        /// One full-bleed red content layer over white, picture caching left
        /// at its default — the case a popup or a dock badge hits.
        fn new() -> Self {
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

            Self {
                engine,
                layer,
                surface: skia::surfaces::raster_n32_premul((100, 100)).unwrap(),
            }
        }

        /// Advance one frame and draw the scene; returns the green channel at
        /// the centre — 0 where the red content covers the white background,
        /// 255 where the background shows through untouched.
        fn draw_frame(&mut self) -> u8 {
            self.engine.update(0.016);

            let canvas = self.surface.canvas();
            canvas.clear(skia::Color::WHITE);
            draw_scene(
                canvas,
                self.engine.scene(),
                self.engine.scene_root().unwrap(),
            );

            let image = self.surface.image_snapshot();
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
            pixels[(50 * 100 + 50) * 4 + 1]
        }
    }

    #[test]
    fn opacity_change_after_first_paint_reaches_the_screen() {
        let mut f = Fixture::new();

        // Frame 1 records the picture at full opacity.
        assert_eq!(
            f.draw_frame(),
            0,
            "opaque content should fully cover the background"
        );

        // Nothing else changes — only opacity.
        f.layer.set_opacity(0.5, None);
        let half = f.draw_frame();
        assert!(
            half > 60 && half < 200,
            "content should blend with the background after opacity drops to 0.5, got green={half}"
        );

        // And back up again, from a picture recorded at 0.5.
        f.layer.set_opacity(1.0, None);
        assert_eq!(
            f.draw_frame(),
            0,
            "content should cover the background again at opacity 1.0"
        );
    }

    #[test]
    fn animated_fade_in_is_visible_before_it_ends() {
        let mut f = Fixture::new();

        // The popup case: created transparent, painted once while invisible,
        // then faded in. Every frame of the fade has to reach the screen.
        f.layer.set_opacity(0.0, None);
        assert_eq!(
            f.draw_frame(),
            255,
            "transparent content should not be drawn at all"
        );

        let transition = Transition::ease_out(0.3);
        f.layer.set_opacity(1.0, Some(transition));

        // Halfway through the fade the content must already be partly there.
        let mut mid = 255;
        for _ in 0..10 {
            mid = f.draw_frame();
        }
        assert!(
            mid < 200,
            "content should be fading in partway through the animation, got green={mid}"
        );

        // And fully opaque once it finishes.
        for _ in 0..20 {
            f.draw_frame();
        }
        assert_eq!(
            f.draw_frame(),
            0,
            "content should be fully opaque when the fade ends"
        );
    }
}
