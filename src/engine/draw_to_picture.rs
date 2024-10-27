use skia_safe::{Picture, PictureRecorder};

use crate::layers::layer::render_layer::RenderLayer;

use super::rendering::Drawable;

#[derive(Clone)]
pub struct DrawDebugInfo {
    pub info: String,
    pub frame: usize,
    pub render_layer: RenderLayer,
}
/// A trait for objects that can be drawn to a PictureRecorder.
pub trait DrawToPicture {
    fn draw_to_picture(&self) -> (Option<Picture>, skia_safe::Rect);
}
/// Drawable can be drawn to a picture.
impl<T> DrawToPicture for T
where
    T: Drawable,
{
    fn draw_to_picture(&self) -> (Option<Picture>, skia_safe::Rect) {
        let mut recorder = PictureRecorder::new();

        // FIXME - this is a hack to make sure we don't clip the edges of the picture
        // and the shadow. We should find a better way to handle this.
        const SAFE_MARGIN: f32 = 50.0;
        let bounds = self.bounds();
        let bounds_safe = skia::Rect::from_wh(bounds.width(), bounds.height())
            .with_outset((SAFE_MARGIN, SAFE_MARGIN));

        let canvas = recorder.begin_recording(bounds_safe, None);
        let damage = self.draw(canvas);

        (recorder.finish_recording_as_picture(None), damage)
    }
}
