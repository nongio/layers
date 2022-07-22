use std::{f64::consts::PI, sync::Arc};

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
    ColorType, Data, Image, Matrix, Surface,
};

use crate::ecs::{
    animations::{Easing, Transition},
    entities::HasHierarchy,
    AnimatedChange,
};
use crate::ecs::{setup_ecs, State};
use crate::types::{BorderRadius, BorderStyle, Color, PaintColor, Point};

use crate::layers::layer::{BlendMode, Layer, ModelLayer};
use crate::rendering::draw;

mod easing;
mod ecs;
mod layers;
mod rendering;
mod types;

fn iconLayer(x: f32) -> Layer {
    let mut matrix = Matrix::new_identity();
    matrix.set_translate_x(x);
    Layer {
        content: None,
        background_color: PaintColor::Solid {
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        },
        border_color: PaintColor::Solid {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        },
        border_width: 0.0,
        size: Point { x: 80.0, y: 80.0 },
        border_style: BorderStyle::Solid,
        border_corner_radius: BorderRadius::new_single(15.0),
        shadow_offset: Point { x: 0.0, y: 0.0 },
        shadow_color: Color::new(0.0, 0.0, 0.0, 0.3),
        shadow_radius: 10.0,
        shadow_spread: 0.0,
        matrix,
        blend_mode: BlendMode::Normal,
    }
}

fn magnify_for_x(x: f64, center: f64) -> (f64, f64) {
    let mut xx = ((x - center) / 200.0) / 2.0;
    xx = xx.max(-1.0);
    xx = xx.min(1.0);

    (xx, 1.0 + (PI * xx * 1.0).cos() / 4.0)
}
#[allow(unreachable_code)]
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

    #[cfg(not(feature = "wayland"))]
    let cb = cb.with_double_buffer(Some(true));

    let windowed_context = cb.build_windowed(window, &events_loop).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let pixel_format = windowed_context.get_pixel_format();

    println!(
        "Pixel format of the window's GL context: {:?}",
        pixel_format
    );

    gl::load_with(|s| windowed_context.get_proc_address(s));

    let gr_context = skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();

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

    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;

    let mut state: State = setup_ecs();
    let surface = None; //create_surface(&windowed_context, &fb_info, &mut gr_context);

    struct Env {
        surface: Option<Surface>,
        gr_context: skia_safe::gpu::DirectContext,
        windowed_context: WindowedContext,
    }

    let mut env = Env {
        surface,
        gr_context,
        windowed_context,
    };

    let mut background_layer = ModelLayer::new();

    // decode an image from a file path
    let data = std::fs::read("/Users/rcanalicchio/Pictures/ventura-resize-1.jpg").unwrap();
    unsafe {
        let data = Data::new_bytes(&data);
        let image = Image::from_encoded(data).unwrap();
        background_layer.content = Some(image);
    }

    state.add_layer(background_layer.clone());

    state.add_change(background_layer.size(
        Point {
            x: 2000.0,
            y: 2000.0,
        },
        None,
    ));

    let mut m = Matrix::new_identity();
    m.set_translate_y(-100.0);
    m.set_translate_x(200.0);
    let dock_layer = ModelLayer::from(Layer {
        content: None,
        background_color: PaintColor::Solid {
            color: Color::new(1.0, 1.0, 1.0, 0.5),
        },
        border_color: PaintColor::Solid {
            color: Color::new(1.0, 1.0, 1.0, 0.3),
        },
        border_width: 1.0,
        size: Point { x: 1000.0, y: 90.0 },
        border_style: BorderStyle::Solid,
        border_corner_radius: BorderRadius::new_single(25.0),
        shadow_offset: Point { x: 00.0, y: 0.0 },
        shadow_color: Color::new(0.0, 0.0, 0.0, 0.3),
        shadow_radius: 20.0,
        shadow_spread: 0.0,
        matrix: m,
        blend_mode: BlendMode::BackgroundBlur,
    });

    let mut dock = state.add_layer(dock_layer.clone());

    let icon_layer = ModelLayer::from(iconLayer(0.0));

    let mut entity = state.add_layer(icon_layer.clone());

    dock.add_child(&mut entity);

    let icon_layer2 = ModelLayer::from(Layer {
        background_color: PaintColor::Solid {
            color: Color::new(1.0, 1.0, 0.0, 1.0),
        },
        ..iconLayer(90.0)
    });

    let mut entity2 = state.add_layer(icon_layer2.clone());
    dock.add_child(&mut entity2);

    let icon_layer3 = ModelLayer::from(Layer {
        background_color: PaintColor::Solid {
            color: Color::new(0.0, 1.0, 0.0, 1.0),
        },
        ..iconLayer(180.0)
    });

    let mut entity3 = state.add_layer(icon_layer3.clone());
    dock.add_child(&mut entity3);

    let icon_layer4 = ModelLayer::from(Layer {
        background_color: PaintColor::Solid {
            color: Color::new(0.0, 1.0, 0.0, 1.0),
        },
        ..iconLayer(270.0)
    });

    let mut entity4 = state.add_layer(icon_layer4.clone());
    dock.add_child(&mut entity4);

    let mut dock_shown = false;
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
                    mouse_x = position.x;
                    mouse_y = position.y;

                    if mouse_y > 0.0 && mouse_y < 100.0 {
                        if !dock_shown {
                            dock_shown = true;
                            state.add_change(dock_layer.position(
                                Point { x: 200.0, y: 0.0 },
                                Some(Transition {
                                    duration: 2.0,
                                    delay: 0.0,
                                    timing: Easing {
                                        ..Default::default()
                                    },
                                }),
                            ));
                        }

                        let offset_x = dock_layer.position.value().x;

                        let (shift1, scale_1) = magnify_for_x(mouse_x - offset_x, 0.0);
                        let (shift2, scale_2) = magnify_for_x(mouse_x - offset_x, 90.0);
                        let (shift3, scale_3) = magnify_for_x(mouse_x - offset_x, 180.0);
                        let (shift4, scale_4) = magnify_for_x(mouse_x - offset_x, 270.0);

                        let changes: Vec<Arc<dyn AnimatedChange>> = vec![
                            icon_layer.scale(
                                Point {
                                    x: scale_1,
                                    y: scale_1,
                                },
                                None,
                            ),
                            icon_layer.position(
                                Point {
                                    x: 0.0 - shift1 * 20.0,
                                    y: icon_layer.position.value().y,
                                },
                                None,
                            ),
                            icon_layer2.scale(
                                Point {
                                    x: scale_2,
                                    y: scale_2,
                                },
                                None,
                            ),
                            icon_layer2.position(
                                Point {
                                    x: 90.0 - shift2 * 20.0,
                                    y: icon_layer2.position.value().y,
                                },
                                None,
                            ),
                            icon_layer3.scale(
                                Point {
                                    x: scale_3,
                                    y: scale_3,
                                },
                                None,
                            ),
                            icon_layer3.position(
                                Point {
                                    x: 180.0 - shift3 * 20.0,
                                    y: icon_layer3.position.value().y,
                                },
                                None,
                            ),
                            icon_layer4.scale(
                                Point {
                                    x: scale_4,
                                    y: scale_4,
                                },
                                None,
                            ),
                            icon_layer4.position(
                                Point {
                                    x: 270.0 - shift4 * 20.0,
                                    y: icon_layer4.position.value().y,
                                },
                                None,
                            ),
                        ];
                        state.add_changes(
                            changes,
                            Some(Transition {
                                duration: 0.5,
                                delay: 0.0,
                                timing: Easing {
                                    ..Default::default()
                                },
                            }),
                        );
                    } else if dock_shown {
                        dock_shown = false;
                        state.add_change(dock_layer.position(
                            Point {
                                x: 200.0,
                                y: -100.0,
                            },
                            Some(Transition {
                                duration: 0.5,
                                delay: 0.0,
                                timing: Easing {
                                    ..Default::default()
                                },
                            }),
                        ));
                    }
                }
                WindowEvent::MouseInput {
                    state: button_state,
                    ..
                } => {
                    if button_state == winit::event::ElementState::Released {
                        // println!("{:?}", mouse_x / 5.0);
                        // state.add_change(dock_layer.change(dock_layer.shadow_radius.to(
                        //     mouse_x / 5.0,
                        //     Some(Transition {
                        //         duration: 2.0,
                        //         delay: 0.0,
                        //         timing: Easing {
                        //             ..Default::default()
                        //         },
                        //     }),
                        // )));
                        // state.add_change(dock_layer.change(dock_layer.position.to(
                        //     Point {
                        //         x: mouse_x,
                        //         y: mouse_y,
                        //     },
                        //     Some(Transition {
                        //         duration: 2.0,
                        //         delay: 0.0,
                        //         timing: Easing {
                        //             ..Default::default()
                        //         },
                        //     }),
                        // )));

                        // state.add_change(layer.change(layer.size.to(
                        //     Point {
                        //         x: mouse_x,
                        //         y: mouse_y,
                        //     },
                        //     Some(Transition {
                        //         duration: 2.0,
                        //         delay: 0.0,
                        //         timing: Easing {
                        //             ..Default::default()
                        //         },
                        //     }),
                        // )));
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                if state.update(0.016) {
                    env.windowed_context.window().request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if let Some(ref mut surface) = env.surface {
                    // test_draw(surface.canvas());
                    draw(surface.canvas(), &state);
                    surface.flush_and_submit();

                    env.windowed_context.swap_buffers().unwrap();
                }
            }
            _ => {}
        }
        // });
    });
}
