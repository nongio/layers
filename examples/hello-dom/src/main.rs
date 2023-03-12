use gl_rs as gl;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile,
};
use mt_dom::{attr, diff, diff_with_key, element, leaf};
use std::{f64::consts::PI, sync::Arc};

use layers::{
    drawing::scene::DrawScene,
    engine::{
        animations::{Easing, Transition},
        LayersEngine,
    },
    layers::*,
    taffy::{
        style::{AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Style},
        style_helpers::points,
        Taffy,
    },
    types::*,
};

pub type MyNode =
    mt_dom::Node<&'static str, &'static str, &'static str, &'static str, &'static str>;

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
    root_layer.set_layout_style(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: Some(JustifyContent::FlexStart),
        flex_wrap: FlexWrap::Wrap,
        align_items: Some(AlignItems::Center),
        ..Default::default()
    });

    root_layer.set_size(
        Size {
            x: window_width as f64 * 2.0,
            y: window_height as f64 * 2.0,
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
    engine.scene_add_layer(root_layer.clone());

    let div1: MyNode = element("layer", [attr("key", "1"), attr("value", "1")], []);
    let div2: MyNode = element("layer", [attr("key", "1"), attr("value", "2")], []);
    let diff = diff_with_key(&div1, &div2, &"key");
    println!("{:#?}", diff);
    return;
    let instant = std::time::Instant::now();
    let mut last_instant = 0.0;

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let now = instant.elapsed().as_secs_f64();
        let dt = now - last_instant;
        engine.step_time(dt);
        last_instant = now;
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
                            x: size.width as f64,
                            y: size.height as f64,
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
                WindowEvent::MouseInput {
                    state: _button_state,
                    ..
                } => {
                    // if button_state == winit::event::ElementState::Released {
                    // } else {
                    // }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let now = instant.elapsed().as_secs_f64();
                let dt = now - last_instant;
                let needs_redraw = engine.update(dt);
                last_instant = now;
                if needs_redraw {
                    env.windowed_context.window().request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                let now = instant.elapsed().as_secs_f64();
                if let Some(root) = engine.scene_root() {
                    let skia_renderer = skia_renderer.get_mut();
                    skia_renderer.draw_scene(&engine.scene(), root);
                }

                let _delta = instant.elapsed().as_secs_f64() - now;
                // println!("draw time: {}ms", delta * 1000.0);
                // this will be blocking until the GPU is done with the frame
                env.windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
        // });
    });
}
