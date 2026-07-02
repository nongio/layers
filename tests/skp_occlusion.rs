use lay_rs::{prelude::*, types::*};
use skia_safe::{PictureRecorder, Rect};

fn record_ops(with_bottom: bool) -> usize {
    let engine = Engine::create(100.0, 100.0);
    let parent = engine.new_layer();
    parent.set_size(Size::points(100.0, 100.0), None);
    engine.add_layer(&parent);

    let top = engine.new_layer();
    top.set_size(Size::points(100.0, 100.0), None);
    top.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 255, 0, 255),
        },
        None,
    );
    engine.append_layer(&top, parent.id);

    if with_bottom {
        let bottom = engine.new_layer();
        bottom.set_size(Size::points(100.0, 100.0), None);
        bottom.set_background_color(
            PaintColor::Solid {
                color: Color::new_rgba255(255, 0, 0, 255),
            },
            None,
        );
        engine.append_layer(&bottom, parent.id);
    }

    engine.update(0.016);

    let mut recorder = PictureRecorder::new();
    let canvas = recorder.begin_recording(Rect::from_wh(100.0, 100.0), None);
    draw_scene(canvas, engine.scene(), engine.scene_root().unwrap());
    let picture = recorder.finish_recording_as_picture(None).unwrap();
    let path = std::env::temp_dir().join(if with_bottom {
        "occlusion_with_bottom.skp"
    } else {
        "occlusion_top_only.skp"
    });
    std::fs::write(&path, picture.serialize().as_bytes()).unwrap();
    picture.approximate_op_count()
}

#[test]
fn occluded_layer_increases_layer_count() {
    let without_bottom = record_ops(false);
    let with_bottom = record_ops(true);
    // Without the covering layer we record two layers: the parent and the top layer.
    assert_eq!(without_bottom, 2);
    // Adding a bottom layer that covers the others should still result in three recorded layers.
    assert_eq!(with_bottom, 3);
}
