use layers::skia;
use layers::{prelude::*, skia::Color4f};

#[derive(Clone)]
pub struct AppSwitcherState {
    pub current_app: usize,
    pub apps: Vec<String>,
}

pub struct AppIconState {
    pub name: String,
    pub is_selected: bool,
    pub index: usize,
}
pub fn view_app_icon(state: AppIconState) -> ViewLayer {
    let mut ICON_SIZE: f32 = 200.0;
    const PADDING: f32 = 20.0;

    let mut selection_background_color = Color::new_hex("#00000000");
    let mut text_opacity = 0.0;
    let mut transition = Transition {
        duration: 2.0,
        ..Default::default()
    };
    if state.is_selected {
        selection_background_color = Color::new_hex("#00000022");
        text_opacity = 1.0;
        ICON_SIZE = 250.0;
        transition = Transition {
            duration: 0.3,
            ..Default::default()
        };
    }
    let picture = {
        let mut recorder = skia::PictureRecorder::new();
        let canvas = recorder.begin_recording(skia::Rect::from_wh(500.0, 500.0), None);

        recorder.finish_recording_as_picture(None)
    };
    ViewLayerBuilder::default()
        // .size((
        //     Point {
        //         x: ICON_SIZE + PADDING * 2.0,
        //         y: ICON_SIZE + PADDING * 2.0 + 50.0,
        //     },
        //     Some(transition),
        // ))
        .background_color((
            PaintColor::Solid {
                color: Color::new_rgba(0.0, 0.0, 0.0, 0.3),
            },
            None,
        ))
        .layout_style(taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: taffy::FlexDirection::Column,
            justify_content: Some(taffy::JustifyContent::Center),
            align_items: Some(taffy::AlignItems::Center),
            gap: taffy::Size {
                width: taffy::LengthPercentage::Points(0.0),
                height: taffy::LengthPercentage::Points(0.0),
            },
            flex_grow: 1.0,
            flex_shrink: 1.0,
            max_size: taffy::Size {
                width: taffy::Dimension::Points(ICON_SIZE * ((state.index + 1) as f32)),
                height: taffy::Dimension::Points(ICON_SIZE * 2.0),
            },
            min_size: taffy::Size {
                width: taffy::Dimension::Points(60.0),
                height: taffy::Dimension::Points(ICON_SIZE),
            },
            ..Default::default()
        })
        .children(vec![ViewLayerBuilder::default()
            .layout_style(taffy::Style {
                display: taffy::Display::Flex,
                flex_direction: taffy::FlexDirection::Column,
                justify_content: Some(taffy::JustifyContent::Center),
                align_items: Some(taffy::AlignItems::Center),
                max_size: taffy::Size {
                    width: taffy::Dimension::Points(ICON_SIZE + PADDING * 2.0),
                    height: taffy::Dimension::Points(ICON_SIZE + PADDING * 2.0),
                },
                min_size: taffy::Size {
                    width: taffy::Dimension::Points(1.0),
                    height: taffy::Dimension::Points(1.0),
                },
                ..Default::default()
            })
            // .size((
            //     Point {
            //         x: ICON_SIZE + PADDING * 2.0,
            //         y: ICON_SIZE + PADDING * 2.0,
            //     },
            //     None,
            // ))
            .background_color((
                PaintColor::Solid {
                    color: selection_background_color,
                },
                None,
            ))
            .border_corner_radius((BorderRadius::new_single(20.0), None)) //     .children(vec![ViewLayerBuilder::default()
            .layout_style(taffy::Style {
                max_size: taffy::Size {
                    width: taffy::Dimension::Points(ICON_SIZE),
                    height: taffy::Dimension::Points(ICON_SIZE),
                },
                ..Default::default()
            })
            // .size((
            //     Point {
            //         x: ICON_SIZE,
            //         y: ICON_SIZE,
            //     },
            //     None,
            // ))
            .background_color((
                PaintColor::Solid {
                    color: Color::new_hex("#00ff00ff"),
                },
                None,
            ))
            .border_corner_radius((BorderRadius::new_single(100.0), None))
            .build()
            .unwrap()])
        .build()
        .unwrap()
}
pub fn view_app_switcher(state: AppSwitcherState) -> ViewLayer {
    const ICON_SIZE: f32 = 200.0;
    const PADDING: f32 = 20.0;

    let background_color = Color::new_rgba(1.0, 1.0, 1.0, 0.4);
    ViewLayerBuilder::default()
        .position((Point { x: 0.0, y: 10.0 }, None))
        .size((
            Point {
                x: 200.0,
                y: ICON_SIZE + PADDING * 2.0 + 100.0,
            },
            None,
        ))
        .background_color((
            PaintColor::Solid {
                color: background_color,
            },
            None,
        ))
        .blend_mode(BlendMode::BackgroundBlur)
        .border_corner_radius((BorderRadius::new_single(50.0), None))
        .layout_style(taffy::Style {
            display: taffy::Display::Flex,
            padding: taffy::Rect {
                left: taffy::LengthPercentage::Points(30.0),
                right: taffy::LengthPercentage::Points(30.0),
                top: taffy::LengthPercentage::Points(0.0),
                bottom: taffy::LengthPercentage::Points(0.0),
            },
            justify_content: Some(taffy::JustifyContent::Center),
            align_items: Some(taffy::AlignItems::Center),
            justify_items: Some(taffy::JustifyItems::Center),
            gap: taffy::Size {
                width: taffy::LengthPercentage::Points(20.0),
                height: taffy::LengthPercentage::Points(PADDING),
            },
            size: taffy::Size {
                width: taffy::auto(), //taffy::Dimension::Percent(1.0),
                height: taffy::Dimension::Points(300.0),
            },
            max_size: taffy::Size {
                width: taffy::Dimension::Percent(1.0),
                height: taffy::Dimension::Points(300.0),
            },
            min_size: taffy::Size {
                width: taffy::Dimension::Points(200.0),
                height: taffy::Dimension::Points(300.0),
            },
            ..Default::default()
        })
        .children(
            state
                .apps
                .iter()
                .enumerate()
                .map(|(i, app)| {
                    view_app_icon(AppIconState {
                        name: app.clone(),
                        is_selected: i == state.current_app,
                        index: i,
                    })
                })
                .collect(),
        )
        .build()
        .unwrap()
}
