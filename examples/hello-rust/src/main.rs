use std::time::Duration;

use gl_rs as gl;
use glutin::{
    event::{Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile,
};

use layers::prelude::*;

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

    gl::load_with(|s| windowed_context.get_proc_address(s));

    let pixel_format = windowed_context.get_pixel_format();

    let size = windowed_context.window().inner_size();
    let sample_count: usize = pixel_format
        .multisampling
        .map(|s| s.try_into().unwrap())
        .unwrap_or(0);
    let pixel_format: usize = pixel_format.stencil_bits.try_into().unwrap();

    let mut skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
        size.width.try_into().unwrap(),
        size.height.try_into().unwrap(),
        sample_count,
        pixel_format,
        0,
    );

    let mut _mouse_x = 0.0;
    let mut _mouse_y = 0.0;

    struct Env {
        windowed_context: WindowedContext,
    }
    let env = Env { windowed_context };
    let engine = LayersEngine::new();
    let root_layer = engine.new_layer();

    root_layer.set_size(
        layers::types::Size {
            x: window_width as f32 * 2.0,
            y: window_height as f32 * 2.0,
        },
        None,
    );
    root_layer.set_position(Point { x: 0.0, y: 0.0 }, None);

    root_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 180, 255),
        },
        None,
    );
    root_layer.set_border_corner_radius(10.0, None);

    engine.scene_add_layer(root_layer.clone());
    let wrap_layer = engine.new_layer();

    wrap_layer.set_position(layers::types::Point { x: 200.0, y: 100.0 }, None);
    wrap_layer.set_size(layers::types::Size { x: 450.0, y: 600.0 }, None);
    wrap_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 0, 0),
        },
        None,
    );
    wrap_layer.set_border_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 0, 255),
        },
        None,
    );
    wrap_layer.set_border_width(4.0, None);

    let container = engine.new_layer();
    container.set_position(layers::types::Point { x: 0.0, y: 0.0 }, None);
    container.set_size(
        layers::types::Size {
            x: 450.0,
            y: 50000.0,
        },
        None,
    );
    container.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 0, 100),
        },
        None,
    );
    container.set_layout_style(Style {
        display: Display::Flex,
        position: Position::Absolute,

        flex_direction: FlexDirection::Row,
        justify_content: Some(JustifyContent::Center),
        flex_wrap: FlexWrap::Wrap,
        align_items: Some(AlignItems::Baseline),
        align_content: Some(AlignContent::FlexStart),
        gap: points(2.0),

        size: layers::taffy::prelude::Size {
            width: points(450.0),
            height: points(50000.0),
        },
        ..Default::default()
    });
    engine.scene_add_layer(wrap_layer.clone());
    engine.scene_add_layer_to(container.clone(), wrap_layer.id());
    let mut layers: Vec<Layer> = Vec::with_capacity(5000);
    for n in 100..200 {
        let image_path = format!("./assets/img_{}.png", n + 1);
        let image = image::open(image_path).unwrap();
        let image = image.into_rgba8();
        let w = image.width() as i32;
        let h = image.height() as i32;
        let data = image.into_vec();

        let layer = engine.new_layer();
        // layer.set_content_from_data_raster_rgba8(&data, w, h);

        layer
            .set_anchor_point(Point { x: 0.5, y: 0.5 }, None)
            .set_size(Point { x: 100.0, y: 100.0 }, None)
            .set_border_corner_radius(20.0, None)
            .set_shadow_color(Color::new_rgba(0.0, 0.0, 0.0, 0.5), None)
            .set_background_color(
                Color::new_rgba(rand::random(), rand::random(), rand::random(), 1.0),
                None,
            )
            .set_shadow_offset(Point { x: 10.0, y: 10.0 }, None)
            .set_shadow_radius(10.0, None)
            .set_layout_style(Style {
                flex_grow: 0.0,
                size: layers::taffy::prelude::Size {
                    width: points(100.0),
                    height: points(100.0),
                },

                ..Default::default()
            });
        layers.push(layer.clone());

        engine.scene_add_layer_to(layer, container.id());
    }
    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;
    let mut scroll_acceleration = 0.0;
    events_loop.run(move |event, _, control_flow| {
        let now = std::time::Instant::now();
        let dt = (now - last_instant).as_secs_f32();
        let next = now.checked_add(Duration::new(0, 2 * 1000000)).unwrap();
        *control_flow = ControlFlow::WaitUntil(next);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    env.windowed_context.resize(physical_size);

                    let size = env.windowed_context.window().inner_size();
                    skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
                        size.width.try_into().unwrap(),
                        size.height.try_into().unwrap(),
                        sample_count,
                        pixel_format,
                        0,
                    );
                    let _transition = root_layer.set_size(
                        Point {
                            x: size.width as f32,
                            y: size.height as f32,
                        },
                        Some(Transition {
                            duration: 1.0,
                            delay: 0.0,
                            timing: Easing::default(),
                        }),
                    );
                    env.windowed_context.window().request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    _mouse_x = position.x;
                    _mouse_y = position.y;
                }
                WindowEvent::MouseWheel {
                    device_id: _,
                    delta,
                    phase: _,
                    modifiers: _,
                } => {
                    match delta {
                        MouseScrollDelta::LineDelta(_x, _y) => {}
                        MouseScrollDelta::PixelDelta(pos) => {
                            let mut y = pos.y as f32 * 500.0;
                            if y != 0.0 {
                                scroll_acceleration = (y / dt) / dt;
                            }

                            // Add momentum when scrolling stops
                            let friction = 0.95;
                            scroll_acceleration *= friction;
                            y = scroll_acceleration * dt * dt;

                            let y = container.position().y + y;
                            let p = Point { x: 0.0, y };

                            container.set_position(
                                p,
                                Some(Transition {
                                    duration: 1.0,
                                    delay: 0.0,
                                    timing: Easing::default(),
                                }),
                            );
                        }
                    };
                }
                WindowEvent::MouseInput {
                    state: button_state,
                    ..
                } => {
                    if button_state == winit::event::ElementState::Released {
                        let _i = 0;

                        layers.iter().for_each(|layer| {
                            let _transition = layer.set_size(
                                Point { x: 100.0, y: 100.0 },
                                Some(Transition {
                                    duration: 0.5,
                                    delay: 0.0,
                                    timing: Easing::default(),
                                }),
                            );
                        });
                    } else {
                        layers.iter().for_each(|layer| {
                            let _transition = layer.set_size(
                                Point { x: 400.0, y: 400.0 },
                                Some(Transition {
                                    duration: 1.5,
                                    delay: 0.0,
                                    timing: Easing::default(),
                                }),
                            );
                        });
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let now = instant.elapsed().as_secs_f64();
                let frame_number = (now / 0.016).floor() as i32;
                if update_frame != frame_number {
                    update_frame = frame_number;
                    let dt = 0.016;
                    let needs_redraw = engine.update(dt);
                    if needs_redraw {
                        env.windowed_context.window().request_redraw();
                        // draw_frame = -1;
                    }
                }
            }
            Event::RedrawRequested(_) => {
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
        // });
    });
}
