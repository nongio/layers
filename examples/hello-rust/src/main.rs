use std::{
    sync::atomic::{AtomicBool, AtomicI32},
    time::Duration,
};

use glutin::event::WindowEvent;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::GlProfile;
use lay_rs::types::Size;
use lay_rs::{prelude::*, skia};

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

    let mut skia_renderer = lay_rs::renderer::skia_fbo::SkiaFboRenderer::create(
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

    let layer = engine.new_layer();
    layer.set_anchor_point((0.5, 0.5), None);
    layer.set_key("test_layer");
    layer.set_position((100.0, 0.0), None);
    layer.set_size(Size::points(100.0, 100.0), None);
    layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    layer.set_border_width(5.0, None);
    layer.set_border_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 0, 255),
        },
        None,
    );
    layer.set_border_corner_radius(BorderRadius::new_single(50.0), None);

    engine.add_layer(layer.clone());

    // engine.start_debugger();
    let mut mass = 1.0;
    let mut stiffness = 0.5;
    let mut damping = 0.9;
    let animation_start = std::sync::Arc::new(AtomicBool::new(false));
    let animation_finished = std::sync::Arc::new(AtomicBool::new(false));
    let animation_progress = std::sync::Arc::new(AtomicI32::new(0));
    let font_mgr = lay_rs::skia::FontMgr::default();
    let typeface = font_mgr
        .match_family_style("Inter", lay_rs::skia::FontStyle::default())
        .unwrap();

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
                        skia_renderer = lay_rs::renderer::skia_fbo::SkiaFboRenderer::create(
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
                        {
                            animation_start.store(false, std::sync::atomic::Ordering::SeqCst);
                            animation_finished.store(false, std::sync::atomic::Ordering::SeqCst);
                            animation_progress.store(0, std::sync::atomic::Ordering::SeqCst);
                        }
                        let animation_start = animation_start.clone();
                        let animation_finished = animation_finished.clone();
                        let animation_progress = animation_progress.clone();

                        layer
                            .set_position(
                                (_mouse_x as f32, _mouse_y as f32),
                                Transition {
                                    delay: 0.0,
                                    timing: TimingFunction::Spring(
                                        Spring::with_duration_and_bounce(1.0, 0.4),
                                    ),
                                },
                            )
                            .on_start(
                                move |_l: &Layer, _p| {
                                    // println!("[{}] animation start", run);

                                    animation_start
                                        .store(true, std::sync::atomic::Ordering::SeqCst);
                                },
                                true,
                            )
                            .on_update(
                                move |_l: &Layer, p| {
                                    // println!("[{}] animation update: {}", run2, p);
                                    animation_progress.store(
                                        (p * 100.0) as i32,
                                        std::sync::atomic::Ordering::SeqCst,
                                    );
                                },
                                false,
                            )
                            .on_finish(
                                move |_l: &Layer, _p| {
                                    // println!("[{}], animation finished: {}", run3, p);
                                    animation_finished
                                        .store(true, std::sync::atomic::Ordering::SeqCst);
                                },
                                true,
                            );
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

                            glutin::event::VirtualKeyCode::M => {
                                if input.state == glutin::event::ElementState::Released {
                                    if input.modifiers == glutin::event::ModifiersState::SHIFT {
                                        mass -= 1.0;
                                    } else {
                                        mass += 1.0;
                                    }
                                }
                            }
                            glutin::event::VirtualKeyCode::S => {
                                if input.state == glutin::event::ElementState::Released {
                                    if input.modifiers == glutin::event::ModifiersState::SHIFT {
                                        stiffness -= 0.1;
                                    } else {
                                        stiffness += 0.1;
                                    }
                                }
                            }
                            glutin::event::VirtualKeyCode::D => {
                                if input.state == glutin::event::ElementState::Released {
                                    if input.modifiers == glutin::event::ModifiersState::SHIFT {
                                        damping -= 0.1;
                                    } else {
                                        damping += 0.1;
                                    }
                                }
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
                        let c: skia::Color4f = skia::Color::BLACK.into();
                        let paint = skia::Paint::new(&c, None);
                        let font = skia::Font::from_typeface(&typeface, 30.0);
                        canvas.draw_str(format!("mass: {}", mass), (20.0, 30.0), &font, &paint);
                        canvas.draw_str(
                            format!("stiffness: {}", stiffness),
                            (20.0, 60.0),
                            &font,
                            &paint,
                        );
                        canvas.draw_str(
                            format!("damping: {}", damping),
                            (20.0, 90.0),
                            &font,
                            &paint,
                        );
                        let animation_start =
                            animation_start.load(std::sync::atomic::Ordering::SeqCst);
                        let animation_finished =
                            animation_finished.load(std::sync::atomic::Ordering::SeqCst);
                        let animation_progress = animation_progress
                            .load(std::sync::atomic::Ordering::SeqCst)
                            .to_string();
                        canvas.draw_str(
                            format!("animation_start: {}", animation_start),
                            (20.0, 120.0),
                            &font,
                            &paint,
                        );
                        canvas.draw_str(
                            format!("animation_finish: {}", animation_finished),
                            (20.0, 150.0),
                            &font,
                            &paint,
                        );
                        canvas.draw_str(
                            format!("animation_progress: {}", animation_progress),
                            (20.0, 180.0),
                            &font,
                            &paint,
                        );

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
