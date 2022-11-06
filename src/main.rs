// #[allow(unreachable_code)]

use gl::types::*;
use gl_rs as gl;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile,
};

use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    Canvas, Color4f, ColorType, Paint, Rect, Surface,
};
use std::{sync::Arc, time::Instant};

use hello::{
    drawing::scene::draw_scene,
    engine::{
        animations::{Easing, Transition},
        node::RenderNode,
        scene::Scene,
        Engine,
    },
    layers::layer::ModelLayer,
    types::{Color, PaintColor, Point},
};

fn draw(canvas: &mut Canvas, _scene: &Scene) {
    let mut paint = Paint::new(Color4f::new(0.6, 0.6, 0.6, 1.0), None);
    paint.set_anti_alias(true);
    paint.set_style(skia_bindings::SkPaint_Style::Fill);
    let w = canvas.image_info().width() as f32;
    let h = canvas.image_info().height() as f32;
    canvas.draw_rect(Rect::from_xywh(0.0, 0.0, w, h), &paint);

    draw_scene(canvas, _scene);
}
fn main() {
    type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

    use winit::dpi::LogicalSize;

    let size: LogicalSize<i32> = LogicalSize::new(1000, 1000);

    let events_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Renderer".to_string());
    // .build(&events_loop)
    // .unwrap();

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_gl_profile(GlProfile::Core);

    // #[cfg(not(feature = "wayland"))]
    // let cb = cb.with_double_buffer(Some(true));
    let windowed_context = cb.build_windowed(window, &events_loop).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let pixel_format = windowed_context.get_pixel_format();

    println!(
        "Pixel format of the window's GL context: {:?}",
        pixel_format
    );

    gl::load_with(|s| windowed_context.get_proc_address(s));

    let mut gr_context = skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();

    let fb_info = {
        let mut fboid: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

        FramebufferInfo {
            fboid: fboid.try_into().unwrap(),
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
        }
    };

    fn create_surface(
        windowed_context: &WindowedContext,
        fb_info: &FramebufferInfo,
        gr_context: &mut skia_safe::gpu::DirectContext,
    ) -> skia_safe::Surface {
        let pixel_format = windowed_context.get_pixel_format();
        let size = windowed_context.window().inner_size();
        let backend_render_target = BackendRenderTarget::new_gl(
            (
                size.width.try_into().unwrap(),
                size.height.try_into().unwrap(),
            ),
            pixel_format.multisampling.map(|s| s.try_into().unwrap()),
            pixel_format.stencil_bits.try_into().unwrap(),
            *fb_info,
        );
        Surface::from_backend_render_target(
            gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap()
    }

    let mut _mouse_x = 0.0;
    let mut _mouse_y = 0.0;

    let surface = create_surface(&windowed_context, &fb_info, &mut gr_context);

    struct Env {
        surface: Option<Surface>,
        gr_context: skia_safe::gpu::DirectContext,
        windowed_context: WindowedContext,
    }

    let mut env = Env {
        surface: Some(surface),
        gr_context,
        windowed_context,
    };
    let engine = Engine::create();
    let layer = ModelLayer::create();
    let _id = engine.scene.add(layer.clone() as Arc<dyn RenderNode>);

    layer.size(Point { x: 100.0, y: 100.0 }, None);
    layer.position(Point { x: 100.0, y: 100.0 }, None);

    layer.background_color(
        PaintColor::Solid {
            color: Color::new(0.0, 0.8, 0.0, 1.0),
        },
        None,
    );

    let instant = std::time::Instant::now();
    let mut last_instant = 0.0;
    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    env.windowed_context.resize(physical_size);
                    env.surface = Some(create_surface(
                        &env.windowed_context,
                        &fb_info,
                        &mut env.gr_context,
                    ));
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
                        layer.position(
                            Point {
                                x: _mouse_x,
                                y: _mouse_y,
                            },
                            Some(Transition {
                                duration: 0.5,
                                delay: 0.0,
                                timing: Easing::default(),
                            }),
                        );
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let dt = instant.elapsed().as_secs_f64() - last_instant;
                let needs_redraw = engine.update(dt);
                last_instant = instant.elapsed().as_secs_f64();
                if needs_redraw {
                    env.windowed_context.window().request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if let Some(ref mut surface) = env.surface {
                    draw(surface.canvas(), &engine.scene);
                    surface.flush_and_submit();

                    env.windowed_context.swap_buffers().unwrap();
                }
            }
            _ => {}
        }
        // });
    });
}
