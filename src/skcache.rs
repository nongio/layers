use crate::layer::RenderLayer;
use crate::rendering::render_layer;

use skia_safe::{Picture, PictureRecorder, Rect};

pub fn render_layer_cache(layer: &RenderLayer) -> Option<Picture> {
    let mut recorder = PictureRecorder::new();

    let canvas = recorder.begin_recording(
        Rect::from_xywh(0.0, 0.0, layer.size.x as f32, layer.size.y as f32),
        None,
    );
    render_layer(canvas, layer);
    recorder.finish_recording_as_picture(None)
}
