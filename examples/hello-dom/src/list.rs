use layers::prelude::*;
use layers::{prelude::timing::TimingFunction, types::Size};
use std::string::String;
use std::sync::Arc;

pub struct ListState {
    pub values: Vec<String>,
}
pub fn view_list(_state: ListState) -> LayerTree {
    const PADDING: f32 = 5.0;

    let background_color = Color::new_hex("#0075FF");

    LayerTreeBuilder::default()
        .position((Point { x: 30.0, y: 330.0 }, None))
        .size((
            Size {
                width: taffy::Dimension::Points(300.0),
                height: taffy::Dimension::Points(30.0),
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
        .border_corner_radius((BorderRadius::new_single(5.0), None))
        .scale((
            Point { x: 2.0, y: 2.0 },
            Some(Transition {
                delay: 0.0,
                duration: 1.0,
                timing: TimingFunction::default(),
            }),
        ))
        .children(vec![LayerTreeBuilder::default()
            .position((
                Point {
                    x: PADDING,
                    y: PADDING,
                },
                None,
            ))
            .size((
                Size {
                    width: taffy::Dimension::Points(290.0),
                    height: taffy::Dimension::Points(20.0),
                },
                None,
            ))
            .background_color((
                PaintColor::Solid {
                    color: Color::new_hex("#FFFFFF"),
                },
                None,
            ))
            // .shadow_color((Color::new_rgba(0.0, 0.0, 0.0, 0.5), None))
            // .shadow_offset((Point { x: 4.0, y: 4.0 }, None))
            // .shadow_radius((4.0, None))
            .build()
            .unwrap()])
        .build()
        .unwrap()
}
