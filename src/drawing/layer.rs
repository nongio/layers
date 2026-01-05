use skia_safe::*;

use crate::{engine::draw_to_picture::DrawDebugInfo, layers::layer::render_layer::RenderLayer};
use crate::{engine::node::SceneNodeRenderable, types::PaintColor};

use super::scene::BACKGROUND_BLUR_SIGMA;

/// Draw a layer into a skia::Canvas.
/// Returns the damage rect in the layer's coordinate space.
#[profiling::function]
pub fn draw_layer(
    canvas: &Canvas,
    layer: &RenderLayer,
    context_opacity: f32,
    renderable: &SceneNodeRenderable,
) -> skia_safe::Rect {
    let mut draw_damage = skia_safe::Rect::default();
    let opacity = layer.opacity * context_opacity;
    // if the layer is completely transparent, we don't need to draw anything
    if layer.premultiplied_opacity <= 0.0 {
        return draw_damage;
    }

    // let bounds = Rect::from_xywh(0.0, 0.0, layer.size.width, layer.size.height);
    let bounds = layer.bounds;
    let rrbounds = &layer.rbounds;
    let background_color = match layer.background_color {
        PaintColor::Solid { color } => Color4f::from(color),
        _ => Color4f::new(1.0, 1.0, 1.0, opacity),
    };
    {
        if (background_color.a * opacity) > 0.0 {
            let save_count = canvas.save();
            canvas.clip_rrect(rrbounds, None, None);

            // Draw the background color.

            let mut background_paint = Paint::new(background_color, None);
            background_paint.set_anti_alias(true);
            background_paint.set_style(PaintStyle::Fill);
            if layer.blend_mode == crate::types::BlendMode::BackgroundBlur {
                background_paint.set_blend_mode(skia_safe::BlendMode::Luminosity);
            }
            if background_color.a > 0.0 {
                canvas.draw_rrect(rrbounds, &background_paint);
            }

            if layer.blend_mode == crate::types::BlendMode::BackgroundBlur {
                let mut paint = skia_safe::Paint::default();
                let noise = skia_safe::shaders::fractal_noise((0.7, 0.7), 3, 0.0, None)
                    .expect("noise shader");

                paint.set_shader(noise);
                paint.set_blend_mode(skia_safe::BlendMode::SoftLight);
                paint.set_alpha(100);

                canvas.draw_paint(&paint);
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
        shadow_paint.set_alpha_f(opacity * layer.shadow_color.alpha);
        canvas.draw_rrect(shadow_rrect, &shadow_paint);
        canvas.restore_to_count(save_count);
        let damage_rect = shadow_rect.with_outset((layer.shadow_radius, layer.shadow_radius));

        draw_damage.join(damage_rect);
    }

    // Draw content if any
    if let Some(content) = renderable.content_cache.as_ref() {
        let save_count = canvas.save();
        if layer.clip_content {
            canvas.clip_rrect(rrbounds, Some(ClipOp::Intersect), Some(true));
        }
        content.playback(canvas);
        canvas.restore_to_count(save_count);
        // draw_damage.join(renderable.repaint_damage);
    } else if let Some(draw_func) = layer.content_draw_func.as_ref() {
        let save_count = canvas.save();
        if layer.clip_content {
            canvas.clip_rrect(rrbounds, Some(ClipOp::Intersect), Some(true));
        }
        let caller = draw_func.0.as_ref();
        let content_damage = caller(canvas, layer.size.width, layer.size.height);
        draw_damage.join(content_damage);

        canvas.restore_to_count(save_count);
    }

    // Draw border
    if layer.border_width > 0.0 {
        let mut border_color = match layer.border_color {
            PaintColor::Solid { color } => Color4f::from(color),
            _ => Color4f::new(1.0, 1.0, 1.0, opacity),
        };
        border_color.a *= opacity;
        let mut border_paint = Paint::new(border_color, None);
        border_paint.set_anti_alias(true);
        border_paint.set_style(PaintStyle::Stroke);
        border_paint.set_stroke_width(layer.border_width);
        canvas.draw_rrect(rrbounds, &border_paint);
        draw_damage.join(bounds.with_outset((layer.border_width / 2.0, layer.border_width / 2.0)));
    }

    if layer.blend_mode == crate::types::BlendMode::BackgroundBlur
        && (background_color.a * opacity) > 0.0
    {
        draw_damage.outset((BACKGROUND_BLUR_SIGMA, BACKGROUND_BLUR_SIGMA));
    }

    draw_damage
}

pub(crate) fn draw_debug(
    canvas: &skia_safe::Canvas,
    dbg_info: &DrawDebugInfo,
    render_layer: &RenderLayer,
) {
    let font_mgr = skia_safe::FontMgr::new();
    let font = font_mgr
        .match_family_style("Inter", skia_safe::FontStyle::default())
        .map(|t| skia_safe::Font::from_typeface_with_params(t, 22.0, 1.0, 0.0))
        .unwrap_or_default();

    let mut paint = skia_safe::Paint::default();
    paint.set_color4f(crate::types::Color::new_hex("#8ABFFF70").c4f(), None);
    canvas.draw_rect(render_layer.bounds_with_children, &paint);

    paint.set_stroke(true);
    paint.set_stroke_width(2.0);
    paint.set_color4f(crate::types::Color::new_hex("#00000070").c4f(), None);
    canvas.draw_rect(render_layer.bounds_with_children, &paint);
    // println!("bounds_with_children: {:?}", bounds_with_children);
    let mut paint = skia_safe::Paint::default();
    paint.set_color4f(crate::types::Color::new_hex("#D1FF8790").c4f(), None);
    canvas.draw_rect(render_layer.bounds, &paint);
    paint.set_stroke(true);
    paint.set_stroke_width(2.0);
    paint.set_color4f(crate::types::Color::new_hex("#00000070").c4f(), None);
    canvas.draw_rect(render_layer.bounds, &paint);

    // paint.set_stroke(false);
    // paint.set_color4f(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.2), None);
    // canvas.draw_rect(bounds, &paint);

    // paint.set_color4f(skia_safe::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    let balloon = skia::RRect::new_rect_xy(
        skia::Rect::from_xywh(
            render_layer.bounds_with_children.x(),
            render_layer.bounds_with_children.y(),
            100.0,
            30.0,
        ),
        10.0,
        10.0,
    );
    paint.set_color4f(crate::types::Color::new_hex("#ffffffff").c4f(), None);
    paint.set_stroke(false);
    canvas.draw_rrect(balloon, &paint);
    paint.set_color4f(crate::types::Color::new_hex("#000000ff").c4f(), None);
    paint.set_stroke(true);
    paint.set_stroke_width(2.0);
    canvas.draw_rrect(balloon, &paint);
    paint.set_stroke(false);
    canvas.draw_str(
        format!(
            "{} | {} | {}",
            &dbg_info.info, dbg_info.frame, render_layer.opacity
        ),
        (
            render_layer.bounds_with_children.x() + 20.0,
            render_layer.bounds_with_children.y() + 20.0,
        ),
        &font,
        &paint,
    );
    // canvas.draw_str(format!("{}", dbg.frame), (0.0, 25.0), &font, &paint);
}
