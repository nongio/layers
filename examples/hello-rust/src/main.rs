use std::{
    fs::read_to_string,
    time::{Duration, Instant},
};

use glutin::event::WindowEvent;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::GlProfile;
use layers::types::Size;
use layers::{prelude::*, skia};

#[allow(unused_assignments)]
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
        .with_title("".to_string());

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
        skia::ColorType::RGBA8888,
        skia::gpu::SurfaceOrigin::BottomLeft,
        0_u32,
    );

    let mut _mouse_x = 0.0;
    let mut _mouse_y = 0.0;

    let window_width = window_width as f32;
    let window_height = window_height as f32;
    let engine = LayersEngine::new(window_width * 6.0, window_height * 6.0);
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
            color: Color::new_rgba255(180, 180, 180, 0),
        },
        None,
    );
    root.set_border_corner_radius(10.0, None);
    root.set_layout_style(taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    });
    root.set_key("root");
    engine.scene_set_root(root.clone());

    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;

    let test_layer = engine.new_layer();
    engine.scene_add_layer_to(test_layer.clone(), root.id());
    test_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    test_layer.set_border_corner_radius(50.0, None);
    test_layer.set_border_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 0, 255),
        },
        None,
    );

    let data = std::fs::read("./assets/square-test.jpg").unwrap();
    let data = skia::Data::new_copy(&data);
    let image = skia::Image::from_encoded(data).unwrap();

    test_layer.set_key("test_layer");
    test_layer.set_position((200.0, 200.0), None);
    test_layer.set_size(Size::points(200.0, 200.0), None);
    // test_layer.set_scale((2.0, 2.0), None);

    test_layer.set_draw_content(Some(move |canvas: &skia::Canvas, w, h| {
        let paint = skia::Paint::new(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        canvas.draw_image_rect_with_sampling_options(
            &image,
            None,
            skia::Rect::from_xywh(0.0, 0.0, w, h),
            skia::SamplingOptions::default(),
            // skia_safe::SamplingOptions::from(resampler),
            &paint,
        );
        skia::Rect::from_xywh(0.0, 0.0, w, h)
    }));

    test_layer.set_border_width(5.0, None);
    // enable image cache
    test_layer.set_image_cache(true);

    let test2 = engine.new_layer();
    test2.set_key("test_child");
    test2.set_position((0.0, 200.0), None);
    test2.set_size(Size::points(300.0, 300.0), None);
    test2.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 100, 255),
        },
        None,
    );
    test2.set_border_width(5.0, None);
    test2.set_border_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 0, 255),
        },
        None,
    );
    // test2.set_image_cache(true);
    engine.scene_add_layer_to(test2.clone(), test_layer.id());

    let test3 = engine.new_layer();
    test3.set_key("test_child2");
    test3.set_position((100.0, 0.0), None);
    test3.set_size(Size::points(100.0, 100.0), None);
    test3.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    test3.set_border_width(5.0, None);
    test3.set_border_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 0, 255),
        },
        None,
    );

    engine.scene_add_layer_to(test3.clone(), test_layer);

    engine.start_debugger();

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
                    let current_surface = skia_renderer.get_mut().surface();
                    if current_surface.width() != size.width as i32
                        || current_surface.height() != size.height as i32
                    {
                        skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
                            (size.width) as i32,
                            (size.height) as i32,
                            sample_count,
                            pixel_format,
                            skia::ColorType::RGBA8888,
                            skia::gpu::SurfaceOrigin::BottomLeft,
                            0_u32,
                        );
                        root.set_size(Size::points(size.width as f32, size.height as f32), None);
                        env.windowed_context.window().request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    _mouse_x = position.x;
                    _mouse_y = position.y;
                    // engine.pointer_move((_mouse_x as f32, _mouse_y as f32), None);
                }

                WindowEvent::MouseInput {
                    state: button_state,
                    ..
                } => {
                    if button_state == glutin::event::ElementState::Released {
                        // test3.set_position((500.0, 500.0), Transition::default());
                    }
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } =>
                {
                    #[allow(clippy::single_match)]
                    match input.virtual_keycode {
                        Some(keycode) => match keycode {
                            glutin::event::VirtualKeyCode::Space => {
                                if input.state == glutin::event::ElementState::Released {
                                    let dt = 0.016;
                                    engine.update(dt);
                                    env.windowed_context.window().request_redraw();
                                }
                            }

                            glutin::event::VirtualKeyCode::D => {
                                if input.state == glutin::event::ElementState::Released {}
                            }
                            glutin::event::VirtualKeyCode::A => {
                                if input.state == glutin::event::ElementState::Released {}
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
                    if needs_redraw {}
                }
                env.windowed_context.window().request_redraw();
            }
            glutin::event::Event::RedrawRequested(_) => {
                if draw_frame != update_frame {
                    if let Some(root) = engine.scene_root() {
                        let skia_renderer = skia_renderer.get_mut();
                        // let damage_rect = engine.damage();

                        let mut surface = skia_renderer.surface();

                        let bounds =
                            skia::Rect::from_wh(surface.width() as f32, surface.height() as f32);
                        let canvas = surface.canvas();

                        let paint = skia::Paint::new(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                        // // draw background white
                        canvas.draw_rect(bounds, &paint);

                        // render the scene
                        canvas.save();
                        skia_renderer.draw_scene(engine.scene(), root, None);
                        canvas.restore();

                        let c: skia::Color4f = skia::Color::GRAY.into();
                        let paint = skia::Paint::new(&c, None);

                        // draw a grid of circles
                        for x in (0..3000).step_by(100) {
                            for y in (0..2000).step_by(100) {
                                canvas.draw_circle((x as f32, y as f32), 5.0, &paint);
                            }
                        }
                        engine.clear_damage();
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
