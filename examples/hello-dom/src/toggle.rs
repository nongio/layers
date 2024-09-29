#![allow(dead_code)]
use layers::{
    prelude::{timing::TimingFunction, *},
    types::Size,
};

pub struct ToggleState {
    pub value: bool,
}
pub fn view_toggle(state: ToggleState) -> LayerTree {
    const SIZE: f32 = 50.0;
    const PADDING: f32 = 5.0;
    const TOGGLE_SIZE: f32 = SIZE * 2.0;
    let position = if state.value {
        PADDING
    } else {
        TOGGLE_SIZE - SIZE - PADDING
    };
    let background_color = if state.value {
        Color::new_hex("#0075FF")
    } else {
        Color::new_hex("#00B407")
    };

    LayerTreeBuilder::default()
        .position((Point { x: 30.0, y: 30.0 }, None))
        .size((
            Size {
                width: taffy::Dimension::Length(TOGGLE_SIZE),
                height: taffy::Dimension::Length(SIZE + PADDING * 2.0),
            },
            None,
        ))
        .background_color((
            PaintColor::Solid {
                color: background_color,
            },
            Some(Transition {
                delay: 0.0,
                duration: 0.3,
                timing: TimingFunction::default(),
            }),
        ))
        .border_corner_radius((BorderRadius::new_single((SIZE + PADDING * 2.0) / 2.0), None))
        .scale((
            Point { x: 3.0, y: 3.0 },
            Some(Transition {
                delay: 0.0,
                duration: 1.0,
                timing: TimingFunction::default(),
            }),
        ))
        .children(vec![LayerTreeBuilder::default()
            .position((
                Point {
                    x: position,
                    y: PADDING,
                },
                Some(Transition {
                    delay: 0.0,
                    duration: 0.3,
                    timing: TimingFunction::default(),
                }),
            ))
            .size((
                Size {
                    width: taffy::Dimension::Length(SIZE),
                    height: taffy::Dimension::Length(SIZE),
                },
                None,
            ))
            .background_color((
                PaintColor::Solid {
                    color: Color::new_hex("#FFFFFF"),
                },
                None,
            ))
            .border_corner_radius((BorderRadius::new_single(SIZE / 2.0), None))
            .shadow_color((Color::new_rgba(0.0, 0.0, 0.0, 0.5), None))
            .shadow_offset((Point { x: 4.0, y: 4.0 }, None))
            .shadow_radius((4.0, None))
            .build()
            .unwrap()])
        .build()
        .unwrap()
}
