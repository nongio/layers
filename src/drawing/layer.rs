// use skia_safe::canvas::SaveLayerRec;
use skia_safe::*;

// use skia_safe::image_filters::{blur, CropRect};
use skia_safe::PaintStyle;
use skia_safe::{Canvas, ClipOp, Point, RRect, Rect};

use crate::layers::layer::render_layer::RenderLayer;
use crate::types::PaintColor;

pub fn draw_layer(canvas: &mut Canvas, layer: &RenderLayer) {
    let bounds = Rect::from_xywh(0.0, 0.0, layer.size.x, layer.size.y);
    let rrbounds = RRect::new_rect_radii(
        bounds,
        &[
            Point::new(
                layer.border_corner_radius.top_left,
                layer.border_corner_radius.top_left,
            ),
            Point::new(
                layer.border_corner_radius.top_right,
                layer.border_corner_radius.top_right,
            ),
            Point::new(
                layer.border_corner_radius.bottom_left,
                layer.border_corner_radius.bottom_left,
            ),
            Point::new(
                layer.border_corner_radius.bottom_right,
                layer.border_corner_radius.bottom_right,
            ),
        ],
    );

    let save_count = canvas.save();
    canvas.clip_rrect(rrbounds, None, None);

    // Draw the background color.

    let background_color = match layer.background_color {
        PaintColor::Solid { color } => Color4f::from(color),
        _ => Color4f::new(1.0, 1.0, 1.0, layer.opacity),
    };

    let mut background_paint = Paint::new(background_color, None);
    background_paint.set_anti_alias(true);
    background_paint.set_style(PaintStyle::Fill);
    // background_paint.set_blend_mode(skia_safe::BlendMode::SrcOver);

    canvas.draw_paint(&background_paint);
    canvas.restore_to_count(save_count);

    // Draw shadow
    if layer.shadow_color.a > 0.0 {
        let mut shadow_paint = Paint::new(Color4f::from(layer.shadow_color), None);

        shadow_paint.set_mask_filter(MaskFilter::blur(
            BlurStyle::Normal,
            layer.shadow_radius,
            false,
        ));
        shadow_paint.set_anti_alias(true);

        let shadow_rrect = RRect::new_rect_xy(
            Rect::from_xywh(
                layer.shadow_offset.x,
                layer.shadow_offset.y,
                layer.size.x,
                layer.size.y,
            ),
            layer.border_corner_radius.top_left,
            layer.border_corner_radius.top_right,
        );
        let save_count = canvas.save();
        canvas.clip_rrect(rrbounds, Some(ClipOp::Difference), Some(true));
        shadow_paint.set_alpha_f(layer.opacity);
        canvas.draw_rrect(shadow_rrect, &shadow_paint);
        canvas.restore_to_count(save_count);
    }

    // Draw border
    if layer.border_width > 0.0 {
        let mut border_color = match layer.border_color {
            PaintColor::Solid { color } => Color4f::from(color),
            _ => Color4f::new(1.0, 1.0, 1.0, layer.opacity),
        };
        border_color.a *= layer.opacity;
        let mut border_paint = Paint::new(border_color, None);
        border_paint.set_style(PaintStyle::Stroke);
        border_paint.set_stroke_width(layer.border_width);
        canvas.draw_rrect(rrbounds, &border_paint);
    }

    // Draw content if any
    if let Some(content) = &layer.content {
        canvas.clip_rrect(rrbounds, Some(ClipOp::Intersect), Some(true));
        content.playback(canvas);
    }
}
