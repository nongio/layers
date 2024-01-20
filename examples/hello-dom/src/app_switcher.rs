use layers::{prelude::*, skia::Color4f};
use layers::{skia, types::Size};

#[derive(Clone, Hash, Debug)]
pub struct AppSwitcherState {
    pub current_app: usize,
    pub apps: Vec<String>,
    pub width: i32,
}

pub struct AppIconState {
    pub name: String,
    pub is_selected: bool,
    pub index: usize,
}
pub fn view_app_icon(state: AppIconState, icon_width: f32) -> ViewLayer {
    const PADDING: f32 = 35.0;

    let draw_picture = move |canvas: &mut skia::Canvas, w: f32, _h: f32| {
        let paint = skia::Paint::new(Color4f::new(1.0, 0.0, 0.0, 1.0), None);

        let width = (w - PADDING * 2.0).max(0.0);

        canvas.draw_rect(
            skia::Rect::from_xywh(PADDING, PADDING, width, width),
            &paint,
        );
    };
    ViewLayerBuilder::default()
        .id(format!("item_{}", state.name))
        .size((
            Size {
                width: taffy::Dimension::Points(icon_width + PADDING * 2.0),
                height: taffy::Dimension::Points(icon_width + PADDING * 2.0),
            },
            None,
        ))
        .background_color((
            PaintColor::Solid {
                color: Color::new_rgba(1.0, 0.0, 0.0, 0.0),
            },
            None,
        ))
        .border_corner_radius((BorderRadius::new_single(20.0), None))
        .content(Some(draw_picture))
        .build()
        .unwrap()
}
pub fn view_app_switcher(state: &AppSwitcherState) -> ViewLayer {
    const COMPONENT_PADDING_H: f32 = 50.0;
    const COMPONENT_PADDING_V: f32 = 80.0;
    const ICON_PADDING: f32 = 35.0;
    const GAP: f32 = 0.0;
    const ICON_SIZE: f32 = 300.0;

    let available_width = state.width as f32;
    let apps_len = state.apps.len() as f32;
    let total_gaps = (apps_len - 1.0) * GAP; // gaps between items

    let total_padding = 2.0 * COMPONENT_PADDING_H + apps_len * ICON_PADDING * 2.0; // padding on both sides
    let available_icon_size =
        (available_width - total_padding - total_gaps) / state.apps.len() as f32;
    let icon_size = ICON_SIZE.min(available_icon_size);
    let component_width = apps_len * icon_size + total_gaps + total_padding;
    let component_height = icon_size + ICON_PADDING * 2.0 + COMPONENT_PADDING_V * 2.0;
    let background_color = Color::new_rgba(1.0, 1.0, 1.0, 0.4);
    let current_app = state.current_app as f32;
    let mut app_name = "".to_string();
    if !state.apps.is_empty() {
        app_name = state.apps[state.current_app].clone();
    }
    let draw_container = move |canvas: &mut skia::Canvas, _w, h| {
        let color = skia::Color4f::new(0.0, 0.0, 0.0, 0.4);
        let paint = skia::Paint::new(color, None);

        let available_icon_size = h - COMPONENT_PADDING_V * 2.0 - ICON_PADDING * 2.0;
        let icon_size = ICON_SIZE.min(available_icon_size);
        let selection_width = icon_size + ICON_PADDING * 2.0;
        let selection_height = selection_width;
        let selection_x = COMPONENT_PADDING_H
            + current_app * (icon_size + ICON_PADDING * 2.0)
            + GAP * current_app;
        let selection_y = h / 2.0 - selection_height / 2.0;
        let rrect = skia::RRect::new_rect_xy(
            skia::Rect::from_xywh(selection_x, selection_y, selection_width, selection_height),
            20.0,
            20.0,
        );
        if apps_len > 0.0 {
            canvas.draw_rrect(rrect, &paint);

            let mut font = skia::Font::default();
            let font_size = 40.0;
            font.set_size(font_size);
            canvas.draw_str_align(
                &app_name,
                (
                    selection_x + selection_width / 2.0,
                    selection_y + selection_height + font_size,
                ),
                &font,
                &paint,
                skia::utils::text_utils::Align::Center,
            );
        }
    };
    ViewLayerBuilder::default()
        .id("apps_switcher")
        .size((
            Size {
                width: taffy::Dimension::Points(component_width),
                height: taffy::Dimension::Points(component_height),
            },
            Some(Transition {
                duration: 1.0,
                ..Default::default()
            }),
        ))
        .blend_mode(BlendMode::BackgroundBlur)
        .background_color((
            PaintColor::Solid {
                color: background_color,
            },
            None,
        ))
        .content(Some(draw_container))
        .border_corner_radius((BorderRadius::new_single(50.0), None))
        .layout_style(taffy::Style {
            position: taffy::Position::Relative,
            display: taffy::Display::Flex,
            justify_content: Some(taffy::JustifyContent::Center),
            align_items: Some(taffy::AlignItems::Center),
            justify_items: Some(taffy::JustifyItems::Center),
            ..Default::default()
        })
        .children(vec![ViewLayerBuilder::default()
            .id("apps_container")
            .size((
                Size {
                    width: taffy::Dimension::Auto,
                    height: taffy::Dimension::Auto,
                },
                Some(Transition {
                    duration: 2.0,
                    ..Default::default()
                }),
            ))
            .layout_style(taffy::Style {
                position: taffy::Position::Absolute,
                display: taffy::Display::Flex,
                justify_content: Some(taffy::JustifyContent::Center),
                justify_items: Some(taffy::JustifyItems::Center),
                align_items: Some(taffy::AlignItems::Baseline),
                ..Default::default()
            })
            .children(
                state
                    .apps
                    .iter()
                    .enumerate()
                    .map(|(i, app)| {
                        view_app_icon(
                            AppIconState {
                                name: app.clone(),
                                is_selected: i == state.current_app,
                                index: i,
                            },
                            icon_size,
                        )
                    })
                    .collect::<Vec<ViewLayer>>(),
            )
            .build()
            .unwrap()])
        .build()
        .unwrap()
}
