//! Drawing reads the scene's arenas and reads them again inside, whenever a
//! mirror (`Layer::as_content`) replays the subtree it follows. A thread that
//! adds a layer meanwhile queues for the write lock between those two reads;
//! the nested read must not queue behind it, or the drawing thread and the
//! writer wait on each other for good.

#[cfg(test)]
mod tests {
    use layers::prelude::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn a_nested_read_does_not_wait_for_a_queued_writer() {
        let engine = Engine::create(500.0, 500.0);
        let root = engine.new_layer();
        engine.add_layer(&root).unwrap();
        engine.update(0.016);

        let (done_tx, done_rx) = mpsc::channel();
        let drawer_engine = engine.clone();
        std::thread::spawn(move || {
            let scene = drawer_engine.scene().clone();
            scene.with_arena(|_| {
                scene.with_renderable_arena(|_| {
                    // Another thread adds a layer while this one draws: it
                    // takes the write lock of both arenas, so it queues.
                    let writer_engine = drawer_engine.clone();
                    let writer = std::thread::spawn(move || {
                        let _ = writer_engine.new_layer();
                    });
                    std::thread::sleep(Duration::from_millis(200));

                    // What a mirror does inside the draw.
                    let nested = scene
                        .try_with_arena(|_| scene.with_renderable_arena(|_| true))
                        .unwrap_or(false);
                    done_tx.send((nested, writer)).unwrap();
                });
            });
        });

        let (nested, writer) = done_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the nested read waited behind the queued writer");
        assert!(nested, "the nested read got no access to the arenas");
        writer.join().unwrap();
    }
}
