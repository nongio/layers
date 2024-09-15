use std::hash::Hash;

use layers::{
    prelude::*,
    skia::{Color4f, Font, FontStyle, Typeface},
};
use layers::{skia, types::Size};

#[derive(Clone, Hash, Debug)]
pub struct AppSwitcherState {
    pub current_app: usize,
    pub apps: Vec<String>,
    pub width: i32,
}

#[derive(Clone)]
pub struct AppIconState {
    pub name: String,
    pub is_selected: bool,
    pub index: usize,
    pub icon_width: f32,
    pub test: i32,
}
impl Hash for AppIconState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.is_selected.hash(state);
        self.index.hash(state);
        self.icon_width.to_bits().hash(state);
        self.test.hash(state);
    }
}
pub fn view_app_icon(state: &AppIconState, view: &View<AppIconState>) -> LayerTree {
    const PADDING: f32 = 35.0;
    let icon_width = state.icon_width;
    // let name_color = state.name.len() as f32 / 10.0;
    // let test = state.test;
    let layer = view.layer.read().unwrap().clone().unwrap();
    // let internal_state = view.layer.sta;
    let index = state.index;
    let val = layer.with_state(|state| state.get::<i32>("notification").unwrap_or_default());
    let id: usize = layer.id().unwrap().0.into();
    let draw_picture = move |canvas: &layers::skia::Canvas, w: f32, h: f32| -> layers::skia::Rect {
        let paint = skia::Paint::new(Color4f::new(1.0, 1.0, 0.0, 1.0), None);
        let width = (w - PADDING * 2.0).max(0.0);
        canvas.draw_rect(
            skia::Rect::from_xywh(PADDING, PADDING, width, width),
            &paint,
        );
        let typeface = Typeface::new("HelveticaNeue", FontStyle::normal()).unwrap();
        let font = Font::new(typeface, 24.0);
        let paint = skia::Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);

        let text = format!("i:{} l:{} v:{}", index, id, val);
        canvas.draw_str(text, (w / 2.0 - 20.0, h / 2.0 - 30.0), &font, &paint);
        skia::Rect::from_xywh(0.0, 0.0, width, width)
    };
    let index = state.index;
    let view1 = view.clone();
    let view2 = view.clone();
    LayerTreeBuilder::default()
        .key(format!("app_icon_view_{}", state.index))
        .size((
            Size {
                width: taffy::Dimension::Length(icon_width + PADDING * 2.0),
                height: taffy::Dimension::Length(icon_width + PADDING * 2.0),
            },
            None,
        ))
        // .background_color((
        //     PaintColor::Solid {
        //         color: Color::new_rgba(1.0, name_color, 0.0, 1.0),
        //     },
        //     Some(Transition::default()),
        // ))
        // .border_corner_radius((BorderRadius::new_single(20.0), None))
        .content(Some(draw_picture))
        .on_pointer_in(move |layer: Layer, x, _y| {
            println!("pointer in");
            let val = layer.with_state(|state| state.get::<i32>("notification").unwrap_or(0));
            layer.with_mut_state(|state| state.insert("notification", x as i32));
            view1.render(&layer);
        })
        .on_pointer_out(move |layer: Layer, x, _y| {
            println!("pointer out");
            let val = layer.with_state(|state| state.get::<i32>("notification").unwrap_or(0));
            layer.with_mut_state(|state| state.insert("notification", x as i32));
            // println!("({}) {}", index, val);
            view2.render(&layer);
        })
        .build()
        .unwrap()
}

struct AppIconView {
    view: View<AppIconState>,
}
impl AppIconView {
    pub fn new(state: AppIconState) -> Self {
        let view = View::new(
            &format!("app_icon_view_{}", state.index),
            state,
            view_app_icon,
        );
        // spawn a thread to call update every second using tokio
        let instance = Self { view: view.clone() };
        // task::spawn(async move {
        //     println!("starting tic toc");
        //     let mut interval = time::interval(Duration::from_millis(1000));
        //     loop {
        //         interval.tick().await;
        //         println!("tick");
        //         let state = view.get_state();
        //         view.update_state(&AppIconState {
        //             test: state.test + 1,
        //             ..state.clone()
        //         });
        //     }
        // });
        instance
    }
    pub fn update(&self) {
        let state = self.view.get_state();
        self.view.update_state(&AppIconState {
            test: state.test + 1,
            ..state.clone()
        });
    }
}
impl layers::prelude::RenderLayerTree for AppIconView {
    fn key(&self) -> String {
        self.view.key()
    }
    fn set_path(&mut self, path: String) {
        self.view.set_path(path);
    }
    fn mount_layer(&self, layer: Layer) {
        self.view.set_layer(layer);
    }
    fn render_layertree(&self) -> LayerTree {
        self.view.render_layertree()
    }
}

pub fn view_app_switcher(state: &AppSwitcherState, _view: &View<AppSwitcherState>) -> LayerTree {
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
    let component_height = 500.0; //icon_size + ICON_PADDING * 2.0 + COMPONENT_PADDING_V * 2.0;
    let background_color = Color::new_rgba(1.0, 1.0, 0.5, 0.4);
    let current_app = state.current_app as f32;
    // let mut app_name = "".to_string();
    // if !state.apps.is_empty() {
    //     app_name = state.apps[state.current_app].clone();
    // }
    let _draw_container = move |canvas: &mut skia::Canvas, _w: f32, h: f32| {
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
            // canvas.draw_str_align(
            //     &app_name,
            //     (
            //         selection_x + selection_width / 2.0,
            //         selection_y + selection_height + font_size,
            //     ),
            //     &font,
            //     &paint,
            //     skia::utils::text_utils::Align::Center,
            // );
        }
    };
    LayerTreeBuilder::default()
        .key("apps_switcher")
        .size((
            Size {
                width: taffy::Dimension::Length(component_width),
                height: taffy::Dimension::Length(component_height),
            },
            Some(Transition {
                duration: 1.0,
                ..Default::default()
            }),
            // None,
        ))
        .blend_mode(BlendMode::BackgroundBlur)
        .background_color((
            PaintColor::Solid {
                color: background_color,
            },
            None,
        ))
        // .content(Some(draw_container))
        .shadow_color(Color::new_rgba(0.0, 0.0, 0.0, 0.3))
        .shadow_offset(((0.0, -10.0).into(), None))
        .shadow_radius((20.0, None))
        .border_corner_radius((BorderRadius::new_single(50.0), None))
        .border_width((10.0, None))
        .layout_style(taffy::Style {
            position: taffy::Position::Relative,
            display: taffy::Display::Flex,
            justify_content: Some(taffy::JustifyContent::Center),
            align_items: Some(taffy::AlignItems::Center),
            justify_items: Some(taffy::JustifyItems::Center),
            ..Default::default()
        })
        .children(vec![LayerTreeBuilder::default()
            .key("apps_container")
            .size((
                Size {
                    width: taffy::Dimension::Auto,
                    height: taffy::Dimension::Length(component_height),
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
                        AppIconView::new(AppIconState {
                            name: app.clone(),
                            is_selected: i == state.current_app,
                            index: i,
                            icon_width: icon_size,
                            test: 0,
                        })
                    })
                    .collect(),
            )
            .build()
            .unwrap()])
        .build()
        .unwrap()
}
