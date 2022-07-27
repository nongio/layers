use crate::engine::entities::HasHierarchy;
use crate::engine::{entities::Entities, Scene};
use crate::layers::layer::{BlendMode, Layer};
use crate::types::{PaintColor, Rectangle};

use skia_safe::canvas::SaveLayerRec;

use skia_safe::image_filters::{blur, CropRect};

use skia_safe::PaintStyle;
use skia_safe::{
    BlurStyle, Canvas, ClipOp, Color4f, MaskFilter, Matrix, Paint, Point, RRect, Rect, TileMode,
};
use skia_safe::{Picture, PictureRecorder};

/// A trait for objects that can be drawn to a canvas.
pub trait Drawable {
    /// Draws the entity on the canvas.
    fn draw(&self, canvas: &mut Canvas);
    /// Returns the area that this drawable occupies.
    fn bounds(&self) -> Rectangle;
    fn transform(&self) -> Matrix;
}

/// A trait for objects that can be drawn to a PictureRecorder.
pub trait DrawCache {
    fn draw_cache(&self) -> Option<Picture>;
}

impl<T> DrawCache for T
where
    T: Drawable,
{
    fn draw_cache(&self) -> Option<Picture> {
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
// impl Drawable for Layer {
pub fn draw_layer(canvas: &mut Canvas, layer: &Layer) {
    let rect = Rect::from_point_and_size((0.0, 0.0), (layer.size.x as f32, layer.size.y as f32));
    let rrect = RRect::new_rect_radii(
        rect,
        &[
            Point::new(
                layer.border_corner_radius.top_left as f32,
                layer.border_corner_radius.top_left as f32,
            ),
            Point::new(
                layer.border_corner_radius.top_right as f32,
                layer.border_corner_radius.top_right as f32,
            ),
            Point::new(
                layer.border_corner_radius.bottom_left as f32,
                layer.border_corner_radius.bottom_left as f32,
            ),
            Point::new(
                layer.border_corner_radius.bottom_right as f32,
                layer.border_corner_radius.bottom_right as f32,
            ),
        ],
    );

    // Draw the background color.

    let mut background_paint = match layer.background_color {
        PaintColor::Solid { color } => Paint::new(Color4f::from(color), None),
        _ => Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
    };
    background_paint.set_anti_alias(true);
    background_paint.set_style(PaintStyle::Fill);

    let bounds = Rect::from_xywh(0.0, 0.0, (layer.size.x) as f32, (layer.size.y) as f32);
    let rrbounds = RRect::new_rect_radii(
        bounds,
        &[
            Point::new(
                layer.border_corner_radius.top_left as f32,
                layer.border_corner_radius.top_left as f32,
            ),
            Point::new(
                layer.border_corner_radius.top_right as f32,
                layer.border_corner_radius.top_right as f32,
            ),
            Point::new(
                layer.border_corner_radius.bottom_left as f32,
                layer.border_corner_radius.bottom_left as f32,
            ),
            Point::new(
                layer.border_corner_radius.bottom_right as f32,
                layer.border_corner_radius.bottom_right as f32,
            ),
        ],
    );
    let mut save_layer_rec = SaveLayerRec::default();

    let blur = blur(
        (40.0, 40.0),
        TileMode::Clamp,
        None,
        Some(CropRect::from(bounds)),
    )
    .unwrap();

    let save_count = canvas.save();

    canvas.clip_rrect(rrbounds, None, None);
    // translucent effect
    match layer.blend_mode {
        BlendMode::BackgroundBlur => {
            save_layer_rec = save_layer_rec.backdrop(&blur).bounds(&bounds);

            canvas.save_layer(&save_layer_rec);
            background_paint.set_blend_mode(skia_safe::BlendMode::SoftLight);
            // canvas.draw_color(layer.background_color, skia_safe::BlendMode::SoftLight);
        }
        BlendMode::Normal => {}
    }

    canvas.draw_paint(&background_paint);
    canvas.restore_to_count(save_count);

    {
        let mut shadow_paint = Paint::new(Color4f::from(layer.shadow_color), None);

        shadow_paint.set_mask_filter(MaskFilter::blur(
            BlurStyle::Normal,
            layer.shadow_radius as f32,
            false,
        ));
        shadow_paint.set_anti_alias(true);

        let shadow_rrect = RRect::new_rect_xy(
            Rect::from_xywh(
                layer.shadow_offset.x as f32,
                layer.shadow_offset.y as f32,
                (layer.size.x) as f32,
                (layer.size.y) as f32,
            ),
            layer.border_corner_radius.top_left as f32,
            layer.border_corner_radius.top_right as f32,
        );
        let save_count = canvas.save();
        canvas.clip_rrect(rrbounds, Some(ClipOp::Difference), Some(true));
        canvas.draw_rrect(shadow_rrect, &shadow_paint);
        canvas.restore_to_count(save_count);
    }
    // Draw border
    if layer.border_width > 0.0 {
        let mut border_paint = match layer.border_color {
            PaintColor::Solid { color } => Paint::new(Color4f::from(color), None),
            _ => Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
        };

        border_paint.set_style(PaintStyle::Stroke);
        border_paint.set_stroke_width(layer.border_width as f32);
        canvas.draw_rrect(rrect, &border_paint);
    }
    // Draw content if any
    if let Some(content) = &layer.content {
        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        paint.set_style(PaintStyle::Fill);
        paint.set_anti_alias(true);
        let image = &*content.data;
        canvas.draw_image(image, (0, 0), Some(&paint));
    }
}

pub fn draw_single_entity(canvas: &mut Canvas, entity: &Entities) {
    match entity {
        Entities::Layer { model, cache, .. } => {
            if let Some(picture) = cache.read().unwrap().picture.clone() {
                let transform = model.transform();
                canvas.concat(&transform);
                canvas.draw_picture(picture, None, None);
            } else {
                model.draw(canvas);
            }
        }
        Entities::Root { .. } => (),
    }
}
pub fn draw_entity(canvas: &mut Canvas, entity: &Entities) {
    canvas.save();
    draw_single_entity(canvas, entity);
    for child in entity.children().iter() {
        draw_entity(canvas, child);
    }
    canvas.restore();
}

pub fn draw(canvas: &mut Canvas, state: &Scene) {
    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

    draw_entity(canvas, &state.root().read().unwrap());
}
