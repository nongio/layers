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

        let canvas = recorder.begin_recording(Rect::from_xywh(0.0, 0.0, r.width, r.height), None);
        self.draw(canvas);
        recorder.finish_recording_as_picture(None)
    }
}
