use skia::Matrix;
use skia_safe::{Picture, PictureRecorder, Rect};

use super::rendering::Drawable;

#[derive(Clone)]
pub struct DrawDebugInfo {
    pub info: String,
}
/// A trait for objects that can be drawn to a PictureRecorder.
pub trait DrawToPicture {
    fn draw_to_picture(
        &self,
        debug_info: Option<DrawDebugInfo>,
    ) -> (Option<Picture>, skia_safe::Rect);
}
/// Drawable can be drawn to a picture.
impl<T> DrawToPicture for T
where
    T: Drawable,
{
    fn draw_to_picture(
        &self,
        debug_info: Option<DrawDebugInfo>,
    ) -> (Option<Picture>, skia_safe::Rect) {
        let mut recorder = PictureRecorder::new();

        let r = self.bounds();
        // FIXME - this is a hack to make sure we don't clip the edges of the picture
        // and the shadow. We should find a better way to handle this.
        const SAFE_MARGIN: f32 = 50.0;

        let canvas = recorder.begin_recording(
            Rect::from_xywh(
                -SAFE_MARGIN,
                -SAFE_MARGIN,
                r.width() + SAFE_MARGIN * 2.0,
                r.height() + SAFE_MARGIN * 2.0,
            ),
            None,
        );
        let damage = self.draw(canvas);
        if let Some(dbg) = debug_info {
            draw_debug(canvas, dbg, self.bounds(), self.transform());
        }
        (recorder.finish_recording_as_picture(None), damage)
    }
}

fn draw_debug(
    canvas: &skia_safe::Canvas,
    dbg: DrawDebugInfo,
    bounds: skia_safe::Rect,
    _transform: Matrix,
) {
    let mut paint = skia_safe::Paint::default();
    paint.set_stroke(true);
    paint.set_stroke_width(2.0);
    paint.set_color4f(skia_safe::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
    let font_mgr = skia_safe::FontMgr::new();
    let typeface = font_mgr
        .match_family_style("Inter", skia_safe::FontStyle::default())
        .unwrap();
    let font = skia_safe::Font::from_typeface_with_params(typeface, 14.0, 1.0, 0.0);

    // let mut font = skia_safe::Font::default();
    // font.set_size(50.0);
    canvas.draw_rect(bounds, &paint);

    paint.set_stroke(false);
    paint.set_color4f(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.2), None);
    canvas.draw_rect(bounds, &paint);

    paint.set_color4f(skia_safe::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    canvas.draw_str(dbg.info, (0.0, 14.0), &font, &paint);
}
