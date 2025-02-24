use skia_safe::{Picture, PictureRecorder};

use crate::{drawing::draw_layer, layers::layer::render_layer::RenderLayer};

#[derive(Clone)]
pub struct DrawDebugInfo {
    pub info: String,
    pub frame: usize,
    pub render_layer: RenderLayer,
}

pub(crate) fn draw_layer_to_picture(
    render_layer: &RenderLayer,
) -> (Option<Picture>, skia_safe::Rect) {
    let mut recorder = PictureRecorder::new();

    // FIXME - this is a hack to make sure we don't clip the edges of the picture
    // and the shadow. We should find a better way to handle this.
    const SAFE_MARGIN: f32 = 50.0;

    let bounds_safe = render_layer.bounds.with_outset((SAFE_MARGIN, SAFE_MARGIN));

    let canvas = recorder.begin_recording(bounds_safe, None);
    let damage = draw_layer(canvas, render_layer, 1.0);

    (recorder.finish_recording_as_picture(None), damage)
}
