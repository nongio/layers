use std::time::Duration;

use glutin::event::WindowEvent;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::GlProfile;
use layers::{drawing::scene::debug_scene, types::Size};
use layers::{prelude::*, skia::ColorType};

use crate::{
    app_switcher::view_app_switcher,
    app_switcher::AppSwitcherState,
    // popup_menu::{popup_menu_view, PopupMenuState},
    // list::{view_list, ListState},
    // toggle::{view_toggle, ToggleState},
};

mod app_switcher;
mod list;
mod popup_menu;
mod toggle;

#[tokio::main]
async fn main() {
    type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

    use glutin::dpi::LogicalSize;
    let window_width = 900;
    let window_height = 800;

    let size: LogicalSize<i32> = LogicalSize::new(window_width, window_height);

    let events_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Renderer".to_string());

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_gl_profile(GlProfile::Core)
        .with_vsync(false);
    let windowed_context = cb.build_windowed(window, &events_loop).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let pixel_format = windowed_context.get_pixel_format();

    println!(
        "Pixel format of the window's GL context: {:?}",
        pixel_format
    );

    gl_rs::load_with(|s| windowed_context.get_proc_address(s));

    let pixel_format = windowed_context.get_pixel_format();

    let size = windowed_context.window().inner_size();
    let sample_count: usize = pixel_format.multisampling.map(|s| s.into()).unwrap_or(0);
    let pixel_format: usize = pixel_format.stencil_bits.into();

    struct Env {
        pub windowed_context: WindowedContext,
    }
    let mut env = Env { windowed_context };

    env.windowed_context = unsafe { env.windowed_context.make_current().unwrap() };

    let mut skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
        size.width as i32,
        size.height as i32,
        sample_count,
        pixel_format,
        ColorType::RGBA8888,
        layers::skia::gpu::SurfaceOrigin::BottomLeft,
        0_u32,
    );

    let mut _mouse_x = 0.0;
    let mut _mouse_y = 0.0;

    let window_width = window_width as f32;
    let window_height = window_height as f32;
    let engine = LayersEngine::new(window_width * 2.0, window_height * 2.0);
    let root = engine.new_layer();
    root.set_size(
        Size {
            width: taffy::Dimension::Length(window_width * 2.0),
            height: taffy::Dimension::Length(window_height * 2.0),
        },
        None,
    );
    root.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 180, 255),
        },
        None,
    );
    root.set_border_corner_radius(10.0, None);
    root.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        display: taffy::Display::Flex,
        padding: taffy::Rect {
            left: taffy::LengthPercentage::Length(0.0),
            right: taffy::LengthPercentage::Length(0.0),
            top: taffy::LengthPercentage::Length(0.0),
            bottom: taffy::LengthPercentage::Length(0.0),
        },
        justify_content: Some(taffy::JustifyContent::Center),
        align_items: Some(taffy::AlignItems::Center),
        ..Default::default()
    });
    engine.scene_set_root(root.clone());
    let data = std::fs::read("./assets/bg.jpg").unwrap();
    let data = layers::skia::Data::new_copy(&data);
    let image = layers::skia::Image::from_encoded(data).unwrap();
    root.set_draw_content(Some(move |canvas: &layers::skia::Canvas, w, h| {
        let mut paint =
            layers::skia::Paint::new(layers::skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        // paint.set_stroke(true);
        // paint.set_stroke_width(10.0);
        // let rect = layers::skia::Rect::new(0.0, 0.0, 100.0, 100.0);
        // canvas.draw_rect(rect, &paint);
        canvas.draw_image(&image, (0.0, 0.0), Some(&paint));
        layers::skia::Rect::from_xywh(0.0, 0.0, w, h)
    }));
    let layer = engine.new_layer();
    engine.scene_add_layer_to(layer.clone(), root.id());

    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;

    let mut state = AppSwitcherState {
        width: 1000,
        current_app: 0,
        apps: vec![],
    };

    state.apps.push("App 1".to_string());

    let mut app_switcher = View::new(
        "app_switcher_view",
        state.clone(),
        Box::new(view_app_switcher),
    );
    app_switcher.mount_layer(layer);

    events_loop.run(move |event, _, control_flow| {
        let now = std::time::Instant::now();
        let _dt = (now - last_instant).as_secs_f32();
        let next = now.checked_add(Duration::new(0, 2 * 1000000)).unwrap();
        *control_flow = ControlFlow::WaitUntil(next);

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    env.windowed_context.resize(physical_size);

                    let size = env.windowed_context.window().inner_size();
                    skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
                        size.width as i32,
                        size.height as i32,
                        sample_count,
                        pixel_format,
                        ColorType::RGBA8888,
                        layers::skia::gpu::SurfaceOrigin::BottomLeft,
                        0_u32,
                    );
                    root.set_size(Size::points(size.width as f32, size.height as f32), None);

                    env.windowed_context.window().request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    _mouse_x = position.x;
                    _mouse_y = position.y;
                    engine.pointer_move(
                        (_mouse_x as f32, _mouse_y as f32),
                        engine.scene_root().unwrap(),
                    );
                }

                WindowEvent::MouseInput {
                    state: _button_state,
                    ..
                } => {
                    // app_switcher_view.render(&state);
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => {
                    #[allow(clippy::single_match)]
                    match input.virtual_keycode {
                        Some(keycode) => match keycode {
                            glutin::event::VirtualKeyCode::Space => {
                                println!("update");
                                let dt = 0.016;
                                let needs_redraw = engine.update(dt);
                                if needs_redraw {
                                    env.windowed_context.window().request_redraw();
                                    // draw_frame = -1;
                                }
                                println!("state {:?}", state);
                                app_switcher.update_state(&state);

                                // if let Some(root) = engine.scene_root() {
                                //     let skia_renderer = skia_renderer.get_mut();
                                //     // let damage_rect = engine.damage();

                                //     let mut surface = skia_renderer.surface();

                                //     let canvas = surface.canvas();

                                //     let save_point = canvas.save();
                                //     skia_renderer.draw_scene(engine.scene(), root, None);

                                //     canvas.restore_to_count(save_point);
                                // }
                            }
                            glutin::event::VirtualKeyCode::Tab => {
                                if input.state == glutin::event::ElementState::Released {
                                    state.current_app = (state.current_app + 1) % state.apps.len();

                                    app_switcher.update_state(&state);
                                }
                            }
                            glutin::event::VirtualKeyCode::A => {
                                if input.state == glutin::event::ElementState::Released {
                                    // let mut rng = rand::thread_rng();
                                    // let index = rng.gen_range(0..12000);
                                    state.apps.pop();
                                    app_switcher.update_state(&state);

                                    // state.apps.push(format!("{}", index));
                                    // app_switcher_view.render(&state);
                                }
                            }
                            glutin::event::VirtualKeyCode::E => {
                                if input.state == glutin::event::ElementState::Released {
                                    let state = app_switcher.get_state();
                                    println!("state {:?}", state);
                                }
                            }
                            glutin::event::VirtualKeyCode::S => {
                                if input.state == glutin::event::ElementState::Released {
                                    state.apps.push("Item".to_string());
                                    app_switcher.update_state(&state);

                                    // let mut rng = rand::thread_rng();
                                    // let index = rng.gen_range(0..state.apps.len());

                                    // app_switcher_view.render(&state);
                                }
                            }
                            glutin::event::VirtualKeyCode::D => {
                                if input.state == glutin::event::ElementState::Released {
                                    debug_scene(engine.scene(), engine.scene_root().unwrap());
                                }
                            }
                            glutin::event::VirtualKeyCode::Escape => {
                                if input.state == glutin::event::ElementState::Released {
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                            _ => (),
                        },
                        None => (),
                    }
                }
                _ => (),
            },
            glutin::event::Event::MainEventsCleared => {
                let now = instant.elapsed().as_secs_f64();
                let frame_number = (now / 0.016).floor() as i32;
                if update_frame != frame_number {
                    update_frame = frame_number;
                    let dt = 0.016;
                    let needs_redraw = engine.update(dt);
                    if needs_redraw {
                        env.windowed_context.window().request_redraw();
                    }
                }
            }
            glutin::event::Event::RedrawRequested(_) => {
                if draw_frame != update_frame {
                    if let Some(root) = engine.scene_root() {
                        let skia_renderer = skia_renderer.get_mut();
                        // let damage_rect = engine.damage();

                        let mut surface = skia_renderer.surface();

                        let canvas = surface.canvas();

                        let save_point = canvas.save();
                        skia_renderer.draw_scene(engine.scene(), root, None);

                        canvas.restore_to_count(save_point);

                        // let mut paint = layers::skia::Paint::new(
                        //     layers::skia::Color4f::new(1.0, 0.0, 0.0, 1.0),
                        //     None,
                        // );
                        // paint.set_stroke(true);
                        // paint.set_stroke_width(10.0);
                        // canvas.draw_rect(damage_rect, &paint);

                        // engine.clear_damage();
                        skia_renderer.gr_context.flush_submit_and_sync_cpu();
                    }
                    // this will be blocking until the GPU is done with the frame
                    env.windowed_context.swap_buffers().unwrap();
                    draw_frame = update_frame;
                }
            }
            _ => {}
        }
    });
}
