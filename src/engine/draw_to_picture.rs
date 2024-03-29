use skia_safe::{Picture, PictureRecorder, Rect};

use super::rendering::Drawable;

/// A trait for objects that can be drawn to a PictureRecorder.
pub trait DrawToPicture {
    fn draw_to_picture(&self) -> Option<Picture>;
}
/// Drawable can be drawn to a picture.
impl<T> DrawToPicture for T
where
    T: Drawable,
{
    fn draw_to_picture(&self) -> Option<Picture> {
        let mut recorder = PictureRecorder::new();

        let r = self.bounds();
        const SAFE_MARGIN: f32 = 500.0; // FIXME - this is a hack to make sure we don't clip the edges of the picture

        let canvas = recorder.begin_recording(
            Rect::from_xywh(
                -SAFE_MARGIN,
                -SAFE_MARGIN,
                r.width + SAFE_MARGIN * 2.0,
                r.height + SAFE_MARGIN * 2.0,
            ),
            None,
        );
        self.draw(canvas);
        recorder.finish_recording_as_picture(None)
    }
}
