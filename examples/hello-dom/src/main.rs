use std::time::Duration;

use glutin::event::WindowEvent;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::GlProfile;
use layers::types::Size;
use layers::{prelude::*, skia::ColorType};

use crate::{
    app_switcher::view_app_switcher,
    toggle::{view_toggle, ToggleState},
};
use crate::{
    app_switcher::AppSwitcherState,
    list::{view_list, ListState},
};

mod app_switcher;
mod list;
mod toggle;
trait View<S> {
    fn view(&self, state: S) -> ViewLayer;
    fn on_press(&self, state: S) {}
    fn on_release(&self, state: S) {}
    fn on_move(&self, state: S) {}
}
// impl View for a function that accept an argument
impl<F, T> View<T> for F
where
    F: Fn(T) -> ViewLayer,
{
    fn view(&self, state: T) -> ViewLayer {
        (*self)(state)
    }
}

fn main() {
    type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

    use winit::dpi::LogicalSize;
    let window_width = 800;
    let window_height = 600;

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
    let sample_count: usize = pixel_format
        .multisampling
        .map(|s| s.try_into().unwrap())
        .unwrap_or(0);
    let pixel_format: usize = pixel_format.stencil_bits.try_into().unwrap();

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

    struct Env {
        windowed_context: WindowedContext,
    }
    let env = Env { windowed_context };
    let engine = LayersEngine::new(window_width as f32 * 2.0, window_height as f32 * 2.0);
    let root = engine.new_layer();
    root.set_size(
        Size {
            width: taffy::Dimension::Points(window_width as f32 * 2.0),
            height: taffy::Dimension::Points(window_height as f32 * 2.0),
        },
        None,
    );
    root.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 180, 255),
        },
        None,
    );
    root.set_position(Point { x: 0.0, y: 0.0 }, None);
    root.set_border_corner_radius(10.0, None);
    root.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        padding: taffy::Rect {
            left: taffy::LengthPercentage::Points(0.0),
            right: taffy::LengthPercentage::Points(0.0),
            top: taffy::LengthPercentage::Points(0.0),
            bottom: taffy::LengthPercentage::Points(0.0),
        },
        size: taffy::Size {
            width: taffy::Dimension::Percent(1.0),
            height: taffy::Dimension::Percent(1.0),
        },
        justify_content: Some(taffy::JustifyContent::Center),
        align_items: Some(taffy::AlignItems::Center),
        ..Default::default()
    });
    engine.scene_set_root(root.clone());
    let layer = engine.new_layer();
    engine.scene_add_layer_to(layer.clone(), root.id());

    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;

    let mut state = AppSwitcherState {
        current_app: 0,
        apps: vec![
            "firefox".into(),
            "code".into(),
            "terminal".into(),
            // "spotify".into(),
            // "discord".into(),
            // "steam".into(),
            // "obs".into(),
            // "blender".into(),
        ],
    };
    let layer_tree = view_app_switcher.view(state.clone());
    layer.build_layer_tree(&layer_tree);

    engine.update(0.0);
    events_loop.run(move |event, _, control_flow| {
        let now = std::time::Instant::now();
        let dt = (now - last_instant).as_secs_f32();
        let next = now.checked_add(Duration::new(0, 2 * 1000000)).unwrap();
        *control_flow = ControlFlow::WaitUntil(next);

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
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

                    env.windowed_context.window().request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    _mouse_x = position.x;
                    _mouse_y = position.y;
                }

                WindowEvent::MouseInput {
                    state: button_state,
                    ..
                } => {
                    let layer_tree = view_app_switcher.view(state.clone());
                    layer.build_layer_tree(&layer_tree);
                }
                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    match input.virtual_keycode {
                        Some(keycode) => match keycode {
                            winit::event::VirtualKeyCode::Space => {
                                println!("update");
                                let dt = 0.016;
                                let needs_redraw = engine.update(dt);
                                if needs_redraw {
                                    env.windowed_context.window().request_redraw();
                                    // draw_frame = -1;
                                }
                            }
                            winit::event::VirtualKeyCode::Tab => {
                                if input.state == winit::event::ElementState::Released {
                                    state.current_app = (state.current_app + 1) % state.apps.len();
                                    println!(
                                        "current app {} {}",
                                        state.current_app,
                                        state.apps.len()
                                    );
                                    let layer_tree = view_app_switcher.view(state.clone());
                                    layer.build_layer_tree(&layer_tree);
                                }
                            }
                            winit::event::VirtualKeyCode::A => {
                                if input.state == winit::event::ElementState::Released {
                                    state.apps.push("test".into());
                                    let layer_tree = view_app_switcher.view(state.clone());
                                    layer.build_layer_tree(&layer_tree);
                                }
                            }
                            winit::event::VirtualKeyCode::S => {
                                if input.state == winit::event::ElementState::Released {
                                    state.apps.pop();
                                    let layer_tree = view_app_switcher.view(state.clone());
                                    layer.build_layer_tree(&layer_tree);
                                }
                            }
                            winit::event::VirtualKeyCode::Escape => {
                                if input.state == winit::event::ElementState::Released {
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
            winit::event::Event::MainEventsCleared => {
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
            winit::event::Event::RedrawRequested(_) => {
                if draw_frame != update_frame {
                    if let Some(root) = engine.scene_root() {
                        let skia_renderer = skia_renderer.get_mut();
                        skia_renderer.draw_scene(engine.scene(), root);
                    }
                    // this will be blocking until the GPU is done with the frame
                    env.windowed_context.swap_buffers().unwrap();
                    draw_frame = update_frame;
                } else {
                    println!("skipping draw");
                }
            }
            _ => {}
        }
    });
}
