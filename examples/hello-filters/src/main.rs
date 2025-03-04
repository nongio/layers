use std::{
    fs::read_to_string,
    time::{Duration, Instant},
};

use glutin::event::WindowEvent;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::GlProfile;
use lay_rs::types::Size;
use lay_rs::{prelude::*, skia::ColorType};

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
        ColorType::RGBA8888,
        lay_rs::skia::gpu::SurfaceOrigin::BottomLeft,
        0_u32,
    );

    let mut _mouse_x = 0.0;
    let mut _mouse_y = 0.0;

    let window_width = window_width as f32;
    let window_height = window_height as f32;
    let engine = Engine::create(window_width * 6.0, window_height * 6.0);
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

    engine.scene_set_root(root.clone());

    let layer = engine.new_layer();
    engine.append_layer(&layer, root.id);

    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;

    let test_layer = engine.new_layer();
    engine.append_layer(&test_layer, root.id);
    test_layer.set_anchor_point((0.0, 0.0), None);
    test_layer.set_size(Size::points(500.0, 900.0), None);
    test_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    test_layer.set_image_cached(true);
    test_layer.set_border_corner_radius(50.0, None);
    test_layer.set_border_width(100.0, None);
    test_layer.set_border_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 0, 0, 255),
        },
        None,
    );

    let test2 = engine.new_layer();
    engine.append_layer(&test2, test_layer.id);
    test2.set_anchor_point((0.0, 0.0), None);
    test2.set_size(Size::points(100.0, 100.0), None);
    test2.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(0, 255, 0, 255),
        },
        None,
    );

    let data = std::fs::read("./assets/grid.jpg").unwrap();
    let data = lay_rs::skia::Data::new_copy(&data);
    let image = lay_rs::skia::Image::from_encoded(data).unwrap();

    test_layer.set_draw_content(move |canvas: &lay_rs::skia::Canvas, w, h| {
        let paint = lay_rs::skia::Paint::new(lay_rs::skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);

        canvas.draw_image(&image, (6.0, 6.0), Some(&paint));
        lay_rs::skia::Rect::from_xywh(0.0, 0.0, w, h)
    });

    test_layer.set_border_width(5.0, None);
    test_layer.set_position((100.0, 100.0), None);

    engine.start_debugger();
    const SHADERFILE: &str = "./assets/genie.sksl";
    let sksl = read_to_string(SHADERFILE).expect("Failed to read SKSL file");

    let runtime_effect = lay_rs::skia::RuntimeEffect::make_for_shader(sksl, None).unwrap();
    let mut builder = lay_rs::skia::runtime_effect::RuntimeShaderBuilder::new(runtime_effect);

    let mut target_x = 300.0;
    let mut target_y = 1000.0;
    let target_w = 250.0;
    let target_h = 250.0;

    let mut progress = 0.0;

    engine.update(0.0);

    let _ = builder.set_uniform_float("dst_bounds", &[target_x, target_y, target_w, target_h]);
    let _ = builder.set_uniform_float("progress", &[progress]);

    let mut animation_start = Instant::now();
    let animation_duration = 1.0;

    let mut filter_shader = None;

    const ANIMATION_STEP: f32 = 0.01;

    let font_mgr = lay_rs::skia::FontMgr::default();
    let typeface = font_mgr
        .match_family_style("Inter", lay_rs::skia::FontStyle::default())
        .unwrap();
    let mut forward = true;
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
                            ColorType::RGBA8888,
                            lay_rs::skia::gpu::SurfaceOrigin::BottomLeft,
                            0_u32,
                        );
                        root.set_size(Size::points(size.width as f32, size.height as f32), None);
                        env.windowed_context.window().request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    _mouse_x = position.x;
                    _mouse_y = position.y;
                    engine.pointer_move((_mouse_x as f32, _mouse_y as f32), None);
                }

                WindowEvent::MouseInput {
                    state: button_state,
                    ..
                } => {
                    if button_state == glutin::event::ElementState::Released {
                        target_x = _mouse_x as f32;
                        target_y = _mouse_y as f32;

                        let _ = builder.set_uniform_float(
                            "dst_bounds",
                            &[target_x, target_y, target_w, target_h],
                        );
                        forward = !forward;
                        progress = 0.0;
                        if !forward {
                            progress = 1.0;
                        }
                        let _ = builder.set_uniform_float("progress", &[progress]);

                        // let render_layer = test_layer.render_bounds();

                        // let mut y = test_layer.position().y;
                        // if target_y - y < (render_layer.height() + 200.0) {
                        //     y = target_y - (render_layer.height() + 200.0);
                        // }
                        // test_layer.set_position((render_layer.x(), y), None);

                        animation_start = now;
                        let dt = 0.016;
                        engine.update(dt);

                        env.windowed_context.window().request_redraw();
                    }
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
                                if input.state == glutin::event::ElementState::Released {
                                    let sksl = read_to_string(SHADERFILE)
                                        .expect("Failed to read SKSL file");

                                    let runtime_effect =
                                        lay_rs::skia::RuntimeEffect::make_for_shader(sksl, None)
                                            .unwrap();
                                    builder =
                                        lay_rs::skia::runtime_effect::RuntimeShaderBuilder::new(
                                            runtime_effect,
                                        );
                                    progress = 0.0;
                                    let _ = builder.set_uniform_float("progress", &[progress]);
                                    forward = !forward;
                                    let dt = 0.016;
                                    engine.update(dt);
                                    env.windowed_context.window().request_redraw();
                                }
                            }

                            glutin::event::VirtualKeyCode::D => {
                                if input.state == glutin::event::ElementState::Released {
                                    progress = progress + ANIMATION_STEP;
                                    // println!("progress {:?}", progress);

                                    let _ = builder.set_uniform_float("progress", &[progress]);

                                    let dt = 0.016;
                                    engine.update(dt);
                                    env.windowed_context.window().request_redraw();
                                }
                            }
                            glutin::event::VirtualKeyCode::A => {
                                if input.state == glutin::event::ElementState::Released {
                                    progress = progress - ANIMATION_STEP;
                                    // println!("progress {:?}", progress);

                                    let _ = builder.set_uniform_float("progress", &[progress]);

                                    let dt = 0.016;
                                    engine.update(dt);
                                    env.windowed_context.window().request_redraw();
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
                let diff = animation_start.elapsed().as_secs_f64();
                if (diff) > 0.0 && (diff <= (animation_duration + 0.1)) {
                    let mut progress = diff / animation_duration;
                    if !forward {
                        progress = 1.0 - progress;
                    }
                    // println!("progress {:?}", progress);
                    let _ = builder.set_uniform_float("progress", &[progress as f32]);
                }
                let render_layer = test_layer.render_bounds_transformed();
                let _ = builder.set_uniform_float(
                    "src_bounds",
                    &[
                        (render_layer.x()),
                        (render_layer.y()),
                        render_layer.width(),
                        render_layer.height(),
                    ],
                );
                let _ = builder
                    .set_uniform_float("dst_bounds", &[target_x, target_y, target_w, target_h]);

                filter_shader = lay_rs::skia::image_filters::runtime_shader(&builder, "", None);

                test_layer.set_image_filter(filter_shader.clone());
                let bounds = lay_rs::skia::Rect::join2(
                    lay_rs::skia::Rect::from_xywh(
                        0.0,
                        0.0,
                        render_layer.width(),
                        render_layer.height(),
                    ),
                    lay_rs::skia::Rect::from_xywh(
                        target_x - render_layer.x(),
                        target_y - render_layer.y(),
                        target_w,
                        target_h,
                    ),
                );

                test_layer.set_filter_bounds(Some(bounds));
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

                        let bounds = lay_rs::skia::Rect::from_wh(
                            surface.width() as f32,
                            surface.height() as f32,
                        );
                        let canvas = surface.canvas();

                        let paint = lay_rs::skia::Paint::new(
                            lay_rs::skia::Color4f::new(1.0, 1.0, 1.0, 1.0),
                            None,
                        );
                        // // draw background white
                        canvas.draw_rect(bounds, &paint);

                        // render the scene
                        skia_renderer.draw_scene(engine.scene(), root, None);

                        // draw debug text
                        let mut paint = lay_rs::skia::Paint::new(
                            lay_rs::skia::Color4f::new(1.0, 1.0, 1.0, 1.0),
                            None,
                        );
                        paint.set_color4f(lay_rs::skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None);

                        let font = lay_rs::skia::Font::from_typeface_with_params(
                            typeface.clone(),
                            40.0,
                            1.0,
                            0.0,
                        );

                        canvas.draw_str(
                            format!("progress: {}", progress),
                            (60.0, 60.0),
                            &font,
                            &paint,
                        );
                        canvas.draw_str(format!("x:{}", target_x), (60.0, 100.0), &font, &paint);
                        canvas.draw_str(format!("y:{}", target_y), (60.0, 140.0), &font, &paint);

                        // // draw target position
                        let mut paint = lay_rs::skia::Paint::default();

                        paint.set_stroke(true);
                        paint.set_stroke_width(3.0);
                        paint.set_color4f(lay_rs::skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None);

                        canvas.draw_rect(
                            lay_rs::skia::Rect::from_xywh(target_x, target_y, target_w, target_h),
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
