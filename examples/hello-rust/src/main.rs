use gl::types::*;
use gl_rs as gl;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile,
};

use rand::*;
use std::sync::Arc;

use layers::{
    drawing::scene::DrawScene,
    engine::{
        animations::{Easing, Transition},
        node::RenderNode,
        scene::Scene,
        Engine, TransactionEventType,
    },
    models::{layer::ModelLayer, text::ModelText},
    types::*,
};

fn main() {
    type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

    use winit::dpi::LogicalSize;

    let size: LogicalSize<i32> = LogicalSize::new(1000, 1000);

    let events_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Renderer".to_string());

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_gl_profile(GlProfile::Core);

    let windowed_context = cb.build_windowed(window, &events_loop).unwrap();

    let mut windowed_context = unsafe { windowed_context.make_current().unwrap() };
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

    let mut skia_renderer = layers::engine::backend::SkiaRenderer::create(
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
    let mut env = Env { windowed_context };
    let engine = Engine::create();
    let root_layer = ModelLayer::create();
    let _text = ModelText::create();
    let layer_id = engine.scene.add(root_layer.clone() as Arc<dyn RenderNode>);

    root_layer.set_size(
        Point {
            x: 2000.0,
            y: 2000.0,
        },
        None,
    );
    root_layer.set_position(Point { x: 0.0, y: 0.0 }, None);

    root_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new(0.6, 0.6, 0.6, 1.0),
        },
        None,
    );

    let mut layers: Vec<Arc<ModelLayer>> = Vec::new();
    // for n in 0..10 {
    let layer = ModelLayer::create();
    layer.set_size(Point { x: 50.0, y: 50.0 }, None);
    layer.set_position(
        Point {
            x: rand::random::<f64>() * 2000.0,
            y: rand::random::<f64>() * 2000.0,
        },
        None,
    );
    layer.set_border_corner_radius(BorderRadius::new_single(15.0), None);
    layer.set_background_color(
        PaintColor::Solid {
            color: Color::new(rand::random(), rand::random(), rand::random(), 1.0),
        },
        None,
    );
    layers.push(layer.clone());
    engine.scene.add(layer as Arc<dyn RenderNode>);
    // }

    let text = ModelText::create();
    {
        *text.text.write().unwrap() = "Hello World".to_string();
    }
    text.set_position(Point { x: 10.0, y: 10.0 }, None);
    text.set_size(Point { x: 500.0, y: 200.0 }, None);
    text.set_font_size(42.0, None);
    println!("text id: {}", text.font_size.value());

    let text_id = engine.scene.add(text.clone() as Arc<dyn RenderNode>);
    engine.scene.append_node_to(text_id, layer_id);

    let instant = std::time::Instant::now();
    let mut last_instant = 0.0;
    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    env.windowed_context.resize(physical_size);

                    let size = env.windowed_context.window().inner_size();
                    skia_renderer = layers::engine::backend::SkiaRenderer::create(
                        size.width.try_into().unwrap(),
                        size.height.try_into().unwrap(),
                        sample_count,
                        pixel_format,
                        0,
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
                    if button_state == winit::event::ElementState::Released {
                        let i = 0;
                        layers.iter().for_each(|layer| {
                            let transition = layer.set_position(
                                Point {
                                    x: rand::random::<f64>() * 2000.0,
                                    y: rand::random::<f64>() * 2000.0,
                                },
                                Some(Transition {
                                    duration: 3.0,
                                    delay: 0.0,
                                    timing: Easing::default(),
                                }),
                            );

                            engine.on_update(transition, move |p| {
                                println!("({}): {}", transition.0, p);
                            });
                            engine.on_finish(transition, move |_p| {
                                println!("transition finished {}", transition.0);
                            });
                        });

                        text.set_size(
                            Point {
                                x: _mouse_x,
                                y: 100.0,
                            },
                            Some(Transition {
                                duration: 2.5,
                                delay: 0.0,
                                timing: Easing::default(),
                            }),
                        );
                    }
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
                skia_renderer.get_mut().draw_scene(&engine.scene);
                env.windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
        // });
    });
}
