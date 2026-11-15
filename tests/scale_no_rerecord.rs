//! A layer's picture is recorded in its own local space, so scaling it — or
//! an ancestor — replays the same picture under a different transform. It
//! must not run the content closure again: Otto's exposé scales a mirror of
//! every window on every frame of its animation, and each re-record replays
//! the window's whole subtree.

#[cfg(test)]
mod tests {
    use layers::prelude::*;
    use layers::skia;
    use layers::types::{Point, Size};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn counted(counter: Arc<AtomicUsize>) -> impl Fn(&skia::Canvas, f32, f32) -> skia::Rect {
        move |c: &skia::Canvas, w: f32, h: f32| {
            counter.fetch_add(1, Ordering::SeqCst);
            let paint = skia::Paint::new(skia::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
            let r = skia::Rect::from_xywh(0.0, 0.0, w, h);
            c.draw_rect(r, &paint);
            r
        }
    }

    fn absolute() -> layers::taffy::Style {
        layers::taffy::Style {
            position: layers::taffy::Position::Absolute,
            ..Default::default()
        }
    }

    #[test]
    fn scaling_a_layer_replays_its_picture() {
        let engine = Engine::create(1000.0, 1000.0);
        let root = engine.new_layer();
        root.set_size(Size::points(1000.0, 1000.0), None);
        engine.add_layer(&root).unwrap();

        let parent = engine.new_layer();
        parent.set_layout_style(absolute());
        parent.set_size(Size::points(400.0, 400.0), None);
        engine.append_layer(&parent, root.id).unwrap();

        let own = Arc::new(AtomicUsize::new(0));
        let scaled = engine.new_layer();
        scaled.set_layout_style(absolute());
        scaled.set_size(Size::points(100.0, 100.0), None);
        scaled.set_draw_content(counted(own.clone()));
        engine.append_layer(&scaled, root.id).unwrap();

        let nested = Arc::new(AtomicUsize::new(0));
        let child = engine.new_layer();
        child.set_layout_style(absolute());
        child.set_size(Size::points(100.0, 100.0), None);
        child.set_draw_content(counted(nested.clone()));
        engine.append_layer(&child, parent.id).unwrap();

        engine.update(0.016);
        let (own_before, nested_before) =
            (own.load(Ordering::SeqCst), nested.load(Ordering::SeqCst));
        assert!(
            own_before >= 1 && nested_before >= 1,
            "recorded once to begin with"
        );

        scaled.set_scale(Point { x: 0.3, y: 0.3 }, Transition::linear(1.0));
        parent.set_scale(Point { x: 0.3, y: 0.3 }, Transition::linear(1.0));
        for _ in 0..12 {
            engine.update(0.1);
        }
        assert_eq!(
            own.load(Ordering::SeqCst),
            own_before,
            "own scale re-recorded"
        );
        assert_eq!(
            nested.load(Ordering::SeqCst),
            nested_before,
            "ancestor scale re-recorded the child"
        );

        // A real size change still re-records.
        scaled.set_size(Size::points(150.0, 150.0), None);
        engine.update(0.016);
        assert!(
            own.load(Ordering::SeqCst) > own_before,
            "size change re-records"
        );
    }
}
