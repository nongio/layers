use std::sync::Arc;

use crate::layers;
use crate::layers::text::Text;
use crate::types::PaintColor;
use skia_safe::textlayout::*;
use skia_safe::PaintStyle;
use skia_safe::*;

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
    // font.set_size(layer.font_size);
    let mut paint = Paint::new(Color4f::from(layer.text_color), None);
    paint.set_style(PaintStyle::Stroke);
    // DEBUG
    canvas.draw_rect(
        Rect::from_xywh(0.0, 0.0, layer.size.x, layer.size.y),
        &paint,
    );

    let mut background_paint = match layer.background_color {
        PaintColor::Solid { color } => Paint::new(Color4f::from(color), None),
        _ => Paint::new(Color4f::new(1.0, 1.0, 1.0, 0.0), None),
    };
    background_paint.set_anti_alias(true);
    background_paint.set_style(PaintStyle::Fill);

    // DEBUG
    // canvas.draw_rect(
    //     Rect::from_xywh(0.0, 0.0, layer.size.x, layer.size.y),
    //     &background_paint,
    // );
    paint.set_style(PaintStyle::Fill);

    let paragraph_style = ParagraphStyle::new();

    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::new(), None);
    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
    let mut ts = TextStyle::new();
    ts.set_font_families(&[layer.font_family.as_str()]);
    ts.set_font_size(layer.font_size);
    ts.set_foreground_color(&paint);

    paragraph_builder.push_style(&ts);

    paragraph_builder.add_text(layer.text.as_str());
    let mut paragraph = paragraph_builder.build();
    paragraph.layout(layer.size.x);
    paragraph.paint(canvas, Point { x: 0.0, y: 0.0 });
}

#[no_mangle]
pub extern "C" fn create_text() -> *const layers::text::ModelText {
    let text = layers::text::ModelText::create();
    Arc::into_raw(text)
}
