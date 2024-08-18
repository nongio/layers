use skia_safe::*;

use crate::layers::layer::render_layer::RenderLayer;
use crate::types::PaintColor;

pub(crate) fn draw_layer(canvas: &Canvas, layer: &RenderLayer) -> skia_safe::Rect {
    let mut draw_damage = skia_safe::Rect::default();
    let bounds = Rect::from_xywh(0.0, 0.0, layer.size.width, layer.size.height);
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
    let background_color = match layer.background_color {
        PaintColor::Solid { color } => Color4f::from(color),
        _ => Color4f::new(1.0, 1.0, 1.0, layer.opacity),
    };
    {
        if (background_color.a * layer.opacity) > 0.0 {
            let save_count = canvas.save();
            canvas.clip_rrect(rrbounds, None, None);

            // Draw the background color.

            let mut background_paint = Paint::new(background_color, None);
            // background_paint.set_anti_alias(true);
            background_paint.set_style(PaintStyle::Fill);
            if layer.blend_mode == crate::types::BlendMode::BackgroundBlur {
                background_paint.set_blend_mode(skia_safe::BlendMode::Luminosity);
            }
            if background_color.a > 0.0 {
                canvas.draw_paint(&background_paint);
            }
            canvas.restore_to_count(save_count);

            draw_damage.join(bounds);
        }
    }
    // Draw shadow
    if layer.shadow_color.alpha > 0.0 {
        let mut shadow_paint = Paint::new(Color4f::from(layer.shadow_color), None);

        shadow_paint.set_mask_filter(MaskFilter::blur(
            BlurStyle::Normal,
            layer.shadow_radius,
            false,
        ));
        // shadow_paint.set_anti_alias(true);

        let shadow_rect = Rect::from_xywh(
            layer.shadow_offset.x,
            layer.shadow_offset.y,
            layer.size.width,
            layer.size.height,
        )
        .with_outset((layer.shadow_spread, layer.shadow_spread));
        let shadow_rrect = RRect::new_rect_radii(shadow_rect, &layer.border_corner_radius.into());
        let save_count = canvas.save();
        canvas.clip_rrect(rrbounds, Some(ClipOp::Difference), Some(true));
        shadow_paint.set_alpha_f(layer.opacity * layer.shadow_color.alpha);
        canvas.draw_rrect(shadow_rrect, &shadow_paint);
        canvas.restore_to_count(save_count);
        draw_damage.join(bounds);
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
        draw_damage.join(bounds.with_outset((layer.border_width / 2.0, layer.border_width / 2.0)));
    }

    // Draw content if any
    if let Some(content) = &layer.content {
        canvas.clip_rrect(rrbounds, Some(ClipOp::Intersect), Some(true));
        content.playback(canvas);
        draw_damage.join(layer.content_damage);
    }

    draw_damage
}
