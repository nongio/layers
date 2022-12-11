use skia_safe::canvas::SaveLayerRec;
// use skia_safe::font_style::*;
use skia_safe::*;

use skia_safe::image_filters::{blur, CropRect};
use skia_safe::textlayout::*;
use skia_safe::PaintStyle;
use skia_safe::{BlurStyle, Canvas, ClipOp, MaskFilter, Point, RRect, Rect, TileMode};

use crate::models::layer::{BlendMode, Layer};
use crate::models::text::Text;
use crate::types::PaintColor;

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

pub fn draw_text(canvas: &mut Canvas, layer: &Text) {
    // let font_manager = FontMgr::default();
    // let mut font = Font::default();
    // let font_style = FontStyle::new(
    //     Weight::NORMAL,
    //     Width::NORMAL,
    //     skia_bindings::SkFontStyle_Slant::Upright,
    // );
    // if let Some(tf) = font_manager.match_family_style(&layer.font_family, font_style) {
    //     font.set_typeface(tf);
    // }

    // font.set_subpixel(true);
    // font.set_size(layer.font_size as f32);
    let mut paint = Paint::new(Color4f::from(layer.text_color), None);
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_rect(
        Rect::from_xywh(0.0, 0.0, layer.size.x as f32, layer.size.y as f32),
        &paint,
    );
    paint.set_style(PaintStyle::Fill);

    let paragraph_style = ParagraphStyle::new();

    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::new(), None);
    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
    let mut ts = TextStyle::new();
    ts.set_font_families(&[layer.font_family.as_str()]);
    ts.set_font_size(layer.font_size as f32);
    ts.set_foreground_color(Some(paint));

    paragraph_builder.push_style(&ts);

    paragraph_builder.add_text(layer.text.as_str());
    let mut paragraph = paragraph_builder.build();
    paragraph.layout(layer.size.x as f32);
    paragraph.paint(canvas, Point { x: 0.0, y: 0.0 });
}
