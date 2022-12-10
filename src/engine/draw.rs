use skia_safe::{Picture, PictureRecorder};

/// A trait for objects that can be drawn to a canvas.
pub trait Drawable {
    /// Draws the entity on the canvas.
    fn draw(&self, canvas: &mut Canvas);
    /// Returns the area that this drawable occupies.
    fn bounds(&self) -> Rectangle;
    /// Returns the transformation matrix for this drawable.
    fn transform(&self) -> Matrix;
}

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

        let canvas = recorder.begin_recording(
            Rect::from_xywh(0.0, 0.0, r.width as f32, r.height as f32),
            None,
        );
        self.draw(canvas);
        recorder.finish_recording_as_picture(None)
    }
}
