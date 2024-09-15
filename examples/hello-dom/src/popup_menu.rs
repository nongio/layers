use std::cell::RefCell;

pub use layers::taffy;
use layers::{
    prelude::{LayerTree, LayerTreeBuilder},
    skia::{self, BlurStyle, ClipOp, MaskFilter},
};

#[derive(Default, Debug, Hash)]
pub struct PopupMenuState {
    pub items: Vec<String>,
}

struct FontCache {
    font_collection: skia::textlayout::FontCollection,
    font_mgr: skia::FontMgr,
    type_face_font_provider: RefCell<skia::textlayout::TypefaceFontProvider>,
}

// source: slint ui
// https://github.com/slint-ui/slint/blob/64e7bb27d12dd8f884275292c2333d37f4e224d5/internal/renderers/skia/textlayout.rs#L31
thread_local! {
    static FONT_CACHE: FontCache = {
        let font_mgr = skia::FontMgr::new();
        let type_face_font_provider = skia::textlayout::TypefaceFontProvider::new();
        let mut font_collection = skia::textlayout::FontCollection::new();
        font_collection.set_asset_font_manager(Some(type_face_font_provider.clone().into()));
        font_collection.set_dynamic_font_manager(font_mgr.clone());
        FontCache { font_collection, font_mgr, type_face_font_provider: RefCell::new(type_face_font_provider) }
    };
}

pub fn popup_menu_item_view(state: &String, selected: bool) -> LayerTree {
    let mut bg_color = layers::types::Color::new_rgba(0.8, 0.8, 0.8, 0.0);
    if selected {
        bg_color = layers::types::Color::new_hex("#0A82FF");
    }
    let text = state.clone();
    let font_size = 26.0;
    let height = 44.0;
    let text_padding_left = 44.0;
    let draw_text = move |canvas: &layers::skia::Canvas, w: f32, h: f32| -> layers::skia::Rect {
        let mut text_style = skia::textlayout::TextStyle::new();

        text_style.set_font_size(font_size);
        // text_style.set_height(h);
        let font_style = skia::FontStyle::new(
            skia::font_style::Weight::MEDIUM,
            skia::font_style::Width::NORMAL,
            skia::font_style::Slant::Upright,
        );
        text_style.set_font_style(font_style);

        // text_style.set_letter_spacing(0.0);
        let font_families = &["Inter"];
        let mut foreground_color = skia::Color4f::new(0.0, 0.0, 0.0, 1.0);
        if selected {
            foreground_color = skia::Color4f::new(1.0, 1.0, 1.0, 1.0);
        }
        let foreground_paint = skia::Paint::new(foreground_color, None);
        text_style.set_foreground_color(&foreground_paint);
        text_style.set_font_families(font_families);

        let mut paragraph_style = skia::textlayout::ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);

        paragraph_style.set_max_lines(1);
        paragraph_style.set_text_align(skia::textlayout::TextAlign::Left);
        paragraph_style.set_text_direction(skia::textlayout::TextDirection::LTR);
        paragraph_style.set_ellipsis("…");

        let mut builder = FONT_CACHE.with(|font_cache| {
            skia::textlayout::ParagraphBuilder::new(
                &paragraph_style,
                font_cache.font_collection.clone(),
            )
        });
        let mut paragraph = builder.add_text(&text).build();
        paragraph.layout(w - text_padding_left);

        let text_y = h / 2.0 - paragraph.height() / 2.0;
        let text_x = text_padding_left;
        let _bounding_box =
            skia::Rect::from_xywh(text_x, text_y, paragraph.max_width(), paragraph.height());
        let mut paint =
            layers::skia::Paint::new(layers::skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
        paint.set_stroke(true);
        // canvas.draw_rect(bounding_box, &paint);
        paragraph.paint(canvas, (text_x, text_y));

        skia::Rect::from_xywh(0.0, 0.0, w, h)
    };
    LayerTreeBuilder::default()
        .key(format!("popup_menu_item_{}", state))
        .size(layers::types::Size {
            width: taffy::style::Dimension::Length(350.0),
            height: taffy::style::Dimension::Length(height),
        })
        .background_color(layers::prelude::PaintColor::Solid { color: bg_color })
        .border_corner_radius(layers::types::BorderRadius::new_single(10.0))
        .content(Some(draw_text))
        .build()
        .unwrap()
}
pub fn popup_menu_view(state: &PopupMenuState) -> LayerTree {
    LayerTreeBuilder::default()
        .key("popup_menu")
        .size(layers::types::Size {
            width: taffy::style::Dimension::Auto,
            height: taffy::style::Dimension::Auto,
        })
        // .blend_mode(layers::types::BlendMode::BackgroundBlur)
        .background_color(layers::prelude::PaintColor::Solid {
            color: layers::types::Color::new_rgba255(246, 246, 246, 153),
        })
        .scale(layers::prelude::Point { x: 1.5, y: 1.5 })
        .border_corner_radius(layers::types::BorderRadius::new_single(12.0))
        .shadow_color((layers::types::Color::new_rgba(0.0, 0.0, 0.0, 0.25), None))
        .shadow_offset((layers::types::Point { x: 0.0, y: 7.0 }, None))
        .shadow_radius((22.0, None))
        .layout_style(taffy::style::Style {
            display: taffy::style::Display::Flex,
            justify_content: Some(taffy::style::JustifyContent::FlexStart),
            align_items: Some(taffy::style::AlignItems::Center),
            flex_direction: taffy::style::FlexDirection::Column,
            padding: taffy::geometry::Rect {
                left: taffy::style::LengthPercentage::Length(10.0),
                right: taffy::style::LengthPercentage::Length(10.0),
                top: taffy::style::LengthPercentage::Length(10.0),
                bottom: taffy::style::LengthPercentage::Length(10.0),
            },
            ..Default::default()
        })
        .content(Some(
            |canvas: &layers::skia::Canvas, w: f32, h: f32| -> layers::skia::Rect {
                let mut shadow_rrect =
                    skia::RRect::new_rect_xy(skia::Rect::from_xywh(0.0, 0.0, w, h), 12.0, 12.0);
                let mut shadow_paint =
                    layers::skia::Paint::new(layers::skia::Color4f::new(0.0, 0.0, 0.0, 0.25), None);
                shadow_paint.set_mask_filter(MaskFilter::blur(BlurStyle::Normal, 3.0, false));
                canvas.clip_rrect(shadow_rrect, Some(ClipOp::Difference), Some(true));
                canvas.draw_rrect(shadow_rrect, &shadow_paint);

                shadow_rrect =
                    skia::RRect::new_rect_xy(skia::Rect::from_xywh(0.0, 36.0, w, h), 12.0, 12.0);
                shadow_paint.set_mask_filter(MaskFilter::blur(BlurStyle::Normal, 100.0, false));
                shadow_paint.set_color4f(layers::skia::Color4f::new(0.0, 0.0, 0.0, 0.4), None);
                canvas.draw_rrect(shadow_rrect, &shadow_paint);

                skia::Rect::from_xywh(0.0, 0.0, w, h)
            },
        ))
        .children(
            state
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| popup_menu_item_view(item, index == 1))
                .collect::<Vec<LayerTree>>(),
        )
        .build()
        .unwrap()
}
