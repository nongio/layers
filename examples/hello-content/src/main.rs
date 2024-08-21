use std::time::Duration;

use gl_rs as gl;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile,
};

use layers::{
    prelude::{timing::TimingFunction, *},
    skia::{self, Color4f, ColorType},
    types::Size,
};

pub fn draw(canvas: &skia::Canvas, width: f32, _height: f32) {
    let mut text_style = skia::textlayout::TextStyle::new();
    text_style.set_font_size(60.0);
    let foreground_paint = skia::Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
    text_style.set_foreground_color(&foreground_paint);
    text_style.set_font_families(&["Inter"]);

    let font_mgr = skia::FontMgr::new();
    let type_face_font_provider = skia::textlayout::TypefaceFontProvider::new();
    let mut font_collection = skia::textlayout::FontCollection::new();
    font_collection.set_asset_font_manager(Some(type_face_font_provider.clone().into()));
    font_collection.set_dynamic_font_manager(font_mgr.clone());

    let mut paragraph_style = skia::textlayout::ParagraphStyle::new();

    paragraph_style.set_text_style(&text_style);
    paragraph_style.set_max_lines(2);
    paragraph_style.set_text_align(skia::textlayout::TextAlign::Center);
    paragraph_style.set_text_direction(skia::textlayout::TextDirection::LTR);
    paragraph_style.set_ellipsis("…");
    let mut paragraph = skia::textlayout::ParagraphBuilder::new(&paragraph_style, font_collection)
        .add_text("Hello, world! 👋🌍")
        .build();

    paragraph.layout(width);
    paragraph.paint(canvas, (0.0, 0.0));
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
    gl::load_with(|s| windowed_context.get_proc_address(s));

    let pixel_format = windowed_context.get_pixel_format();

    let window_size = windowed_context.window().inner_size();
    let sample_count: usize = pixel_format
        .multisampling
        .map(|s| s.try_into().unwrap())
        .unwrap_or(0);
    let pixel_format: usize = pixel_format.stencil_bits.try_into().unwrap();

    let mut skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
        window_size.width as i32,
        window_size.height as i32,
        sample_count,
        pixel_format,
        ColorType::RGBA8888,
        layers::skia::gpu::SurfaceOrigin::BottomLeft,
        0_u32,
    );

    struct Env {
        windowed_context: WindowedContext,
    }
    let env = Env { windowed_context };
    let engine = LayersEngine::new(window_width as f32 * 2.0, window_height as f32 * 2.0);
    let root_layer = engine.new_layer();

    root_layer.set_size(
        layers::types::Size::points(window_width as f32 * 2.0, window_height as f32 * 2.0),
        None,
    );
    root_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 180, 255),
        },
        None,
    );
    root_layer.set_border_corner_radius(10.0, None);
    root_layer.set_layout_style(taffy::Style {
        display: taffy::Display::Flex,
        align_content: Some(taffy::AlignContent::Center),
        align_items: Some(taffy::AlignItems::Center),
        justify_content: Some(taffy::JustifyContent::Center),
        ..Default::default()
    });
    engine.scene_add_layer(root_layer.clone());

    let other = engine.new_layer();
    other.set_size(layers::types::Size::points(100.0, 100.0), None);
    other.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 0, 0, 255),
        },
        None,
    );
    other.set_border_corner_radius(10.0, None);
    other.set_layout_style(taffy::Style {
        // position: taffy::Position::Absolute,
        ..Default::default()
    });
    engine.scene_add_layer(other.clone());
    let content_layer = engine.new_layer();
    let inner_content_layer = engine.new_layer();
    inner_content_layer.set_position(
        Point {
            x: -100.0,
            y: -100.0,
        },
        None,
    );
    inner_content_layer.set_size(layers::types::Size::points(600.0, 600.0), None);
    inner_content_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 255, 0, 100),
        },
        None,
    );
    inner_content_layer.set_border_corner_radius(BorderRadius::new_single(1.0), None);
    content_layer.set_size(layers::types::Size::points(620.0, 620.0), None);
    content_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 255, 255, 255),
        },
        None,
    );
    content_layer.set_border_corner_radius(50.0, None);
    content_layer.set_layout_style(taffy::Style {
        // position: taffy::Position::Absolute,
        ..Default::default()
    });

    engine.scene_add_layer(content_layer.clone());
    engine.scene_add_layer_to(inner_content_layer.clone(), content_layer.id());
    inner_content_layer.set_draw_content(Some(
        |canvas: &layers::skia::Canvas, width, height| -> layers::skia::Rect {
            draw(canvas, width, height);
            layers::skia::Rect::from_wh(width, height)
        },
    ));

    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;

    let mut w = 1.0;
    let mut h = 1.0;
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
                        size.width as i32,
                        size.height as i32,
                        sample_count,
                        pixel_format,
                        ColorType::RGBA8888,
                        layers::skia::gpu::SurfaceOrigin::BottomLeft,
                        0_u32,
                    );
                    let _transition = root_layer
                        .set_size(Size::points(size.width as f32, size.height as f32), None);
                    env.windowed_context.window().request_redraw();
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => {
                    #[allow(clippy::single_match)]
                    match input.virtual_keycode {
                        Some(keycode) => match keycode {
                            winit::event::VirtualKeyCode::Space => {
                                if input.state == winit::event::ElementState::Released {
                                    let dt = 0.016;
                                    let needs_redraw = engine.update(dt);
                                    if needs_redraw {
                                        env.windowed_context.window().request_redraw();
                                        // draw_frame = -1;
                                    }
                                }
                            }
                            winit::event::VirtualKeyCode::A => {
                                content_layer.set_position(
                                    Point { x: 0.0, y: 0.0 },
                                    Some(Transition {
                                        duration: 2.0,
                                        ..Default::default()
                                    }),
                                );
                            }
                            winit::event::VirtualKeyCode::W => {
                                content_layer.set_scale(
                                    Point { x: 0.5, y: 0.5 },
                                    Some(Transition {
                                        duration: 2.0,
                                        ..Default::default()
                                    }),
                                );
                            }
                            winit::event::VirtualKeyCode::S => {
                                content_layer.set_scale(
                                    Point { x: 2.0, y: 2.0 },
                                    Some(Transition {
                                        duration: 2.0,
                                        ..Default::default()
                                    }),
                                );
                            }

                            winit::event::VirtualKeyCode::D => {
                                content_layer.set_position(
                                    Point { x: 600.0, y: 600.0 },
                                    Some(Transition {
                                        duration: 2.0,
                                        ..Default::default()
                                    }),
                                );
                            }
                            winit::event::VirtualKeyCode::E => {
                                w += 10.0;
                                h += 10.0;
                                content_layer.set_position(
                                    Point { x: 0.0, y: 0.0 },
                                    Some(Transition {
                                        duration: 2.0,
                                        ..Default::default()
                                    }),
                                );
                                inner_content_layer.set_draw_content(Some(
                                    move |canvas: &layers::skia::Canvas, width, height| -> layers::skia::Rect {
                                        draw(canvas, width, height);
                                        layers::skia::Rect::from_wh(w, h)
                                    },
                                ));
                            }
                            winit::event::VirtualKeyCode::Escape => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => (),
                        },
                        None => (),
                    }
                }
                WindowEvent::CursorMoved { position: _, .. } => {
                    // _mouse_x = position.x;
                    // _mouse_y = position.y;
                }

                WindowEvent::MouseInput { state: _, .. } => {}
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
                        draw_frame = -1;
                    }
                }
            }
            Event::RedrawRequested(_) => {
                if draw_frame != update_frame {
                    if let Some(root) = engine.scene_root() {
                        let skia_renderer = skia_renderer.get_mut();
                        let damage_rect = engine.damage();

                        skia_renderer.draw_scene(engine.scene(), root, None);

                        let mut surface = skia_renderer.surface();
                        let canvas = surface.canvas();
                        let mut paint = skia::Paint::new(Color4f::new(1.0, 0.0, 0.0, 1.0), None);
                        paint.set_stroke(true);
                        paint.set_stroke_width(10.0);
                        canvas.draw_rect(damage_rect, &paint);
                        skia_renderer.gr_context.flush_and_submit();
                    }
                    engine.clear_damage();
                    // this will be blocking until the GPU is done with the frame
                    env.windowed_context.swap_buffers().unwrap();
                    draw_frame = update_frame;
                } else {
                    // println!("skipping draw");
                }
            }
            _ => {}
        }
        // });
    });
}
