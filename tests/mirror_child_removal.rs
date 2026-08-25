//! Removing a node has to reach the mirrors of its ancestors.
//!
//! A follower replays its leader's subtree, so a child disappearing from the
//! leader changes what the follower shows. Descendant *changes* mark the
//! followers of every ancestor for repaint; a descendant *removal* only
//! attributed damage to the nearest surviving ancestor and left the followers
//! alone, so the mirror kept painting a child that no longer exists.
//!
//! Otto renders the wlr-layer-shell `bottom` layer this way — the container is
//! an offscreen content source and each workspace shows a mirror of it — so a
//! desktop widget whose client exited stayed frozen on the desktop.

#[cfg(test)]
mod tests {
    use layers::prelude::*;
    use layers::skia;
    use layers::types::Size;

    fn red_fill(_c: &skia::Canvas, w: f32, h: f32) -> skia::Rect {
        let mut paint = skia::Paint::new(skia::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
        paint.set_anti_alias(false);
        let r = skia::Rect::from_xywh(0.0, 0.0, w, h);
        _c.draw_rect(r, &paint);
        r
    }

    fn absolute() -> layers::taffy::Style {
        layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        }
    }

    /// Green channel at `(x, y)`: 0 where the red child covers the white
    /// background, 255 where the background shows through.
    fn green_at(engine: &std::sync::Arc<Engine>, surface: &mut skia::Surface, x: i32, y: i32) -> u8 {
        let canvas = surface.canvas();
        canvas.clear(skia::Color::WHITE);
        draw_scene(canvas, engine.scene(), engine.scene_root().unwrap());

        let image = surface.image_snapshot();
        let mut pixels = vec![0u8; 300 * 300 * 4];
        let info = skia::ImageInfo::new(
            (300, 300),
            skia::ColorType::RGBA8888,
            skia::AlphaType::Premul,
            None,
        );
        assert!(image.read_pixels(
            &info,
            &mut pixels,
            300 * 4,
            (0, 0),
            skia::image::CachingHint::Allow
        ));
        pixels[((y as usize) * 300 + x as usize) * 4 + 1]
    }

    #[test]
    fn mirror_drops_a_removed_child() {
        let engine = Engine::create(300.0, 300.0);

        let root = engine.new_layer();
        root.set_size(Size::points(300.0, 300.0), None);
        engine.add_layer(&root).unwrap();

        // The leader: a bare container, like Otto's offscreen
        // `layer_shell_bottom`. Its pixels are entirely its children's.
        let leader = engine.new_layer();
        leader.set_layout_style(absolute());
        leader.set_position((0.0, 0.0), None);
        leader.set_size(Size::points(100.0, 100.0), None);
        engine.append_layer(&leader, root.id).unwrap();

        // The widget inside it.
        let child = engine.new_layer();
        child.set_layout_style(absolute());
        child.set_position((0.0, 0.0), None);
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_draw_content(red_fill);
        engine.append_layer(&child, leader.id).unwrap();

        engine.update(0.016);

        // The mirror, off to the side.
        let follower = engine.new_layer();
        follower.set_layout_style(absolute());
        follower.set_position((150.0, 0.0), None);
        follower.set_size(Size::points(100.0, 100.0), None);
        follower.set_draw_content(leader.as_content());
        leader.add_follower_node(follower.id());
        engine.append_layer(&follower, root.id).unwrap();

        engine.update(0.016);

        let mut surface = skia::surfaces::raster_n32_premul((300, 300)).unwrap();
        assert_eq!(
            green_at(&engine, &mut surface, 200, 50),
            0,
            "the mirror should show the leader's child to begin with"
        );

        // The client goes away.
        child.remove();
        engine.update(0.016);

        assert_eq!(
            green_at(&engine, &mut surface, 50, 50),
            255,
            "the leader itself should be empty once its child is removed"
        );
        assert_eq!(
            green_at(&engine, &mut surface, 200, 50),
            255,
            "the mirror should drop the child too, not keep painting it"
        );
    }
}
