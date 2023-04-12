use layers::prelude::*;
use layers::types::Size;
use std::sync::Arc;

pub struct ToggleState {
    pub value: bool,
}
pub fn view_toggle(state: ToggleState) -> ViewLayerTree {
    const SIZE: f32 = 50.0;
    const PADDING: f32 = 5.0;
    let TOGGLE_SIZE = SIZE * 2.0;
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
    ViewLayerTreeBuilder::default()
        .root(Arc::new(
            ViewLayerBuilder::default()
                .position((Point { x: 30.0, y: 30.0 }, None))
                .size((
                    Point {
                        x: TOGGLE_SIZE,
                        y: SIZE + PADDING * 2.0,
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
                        timing: Easing::default(),
                    }),
                ))
                .border_corner_radius((
                    BorderRadius::new_single((SIZE + PADDING * 2.0) / 2.0),
                    None,
                ))
                .scale((
                    Point { x: 3.0, y: 3.0 },
                    Some(Transition {
                        delay: 0.0,
                        duration: 1.0,
                        timing: Easing::default(),
                    }),
                ))
                .build()
                .unwrap(),
        ))
        .children(vec![Arc::new(
            ViewLayerTreeBuilder::default()
                .root(Arc::new(
                    ViewLayerBuilder::default()
                        .position((
                            Point {
                                x: position,
                                y: PADDING,
                            },
                            Some(Transition {
                                delay: 0.0,
                                duration: 0.3,
                                timing: Easing::default(),
                            }),
                        ))
                        .size((Point { x: SIZE, y: SIZE }, None))
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
                        .unwrap(),
                ))
                .build()
                .unwrap(),
        )])
        .build()
        .unwrap()
}
