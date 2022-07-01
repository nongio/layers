
use skia_safe::{scalar, ColorType, Size, Surface};

use crate::layer::{ModelLayer, ModelChanges, Point, BorderRadius, Color};
use crate::rendering::draw;
use crate::ecs::{State, setup_ecs, Entities};
use crate::ecs::animations::{Transition, Easing};

mod rendering;
mod layer;
mod ecs;
mod easing;
mod skcache;

#[allow(unreachable_code)]
fn main() {
    use cocoa::{appkit::NSView, base::id as cocoa_id};

    use core_graphics_types::geometry::CGSize;
    use std::mem;

    use foreign_types_shared::{ForeignType, ForeignTypeRef};
    use metal_rs::{Device, MTLPixelFormat, MetalLayer};
    use objc::{rc::autoreleasepool, runtime::YES};

    use skia_safe::gpu::{mtl, BackendRenderTarget, DirectContext, SurfaceOrigin};

    use winit::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::macos::WindowExtMacOS,
        window::WindowBuilder,
    };

    
    

    let size = LogicalSize::new(1000, 1000);

    let events_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Renderer".to_string())
        .build(&events_loop)
        .unwrap();

    let device = Device::system_default().expect("no device found");

    let metal_layer = {
        let draw_size = window.inner_size();
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);
        layer.set_display_sync_enabled(false); // <-- vsync

        unsafe {
            let view = window.ns_view() as cocoa_id;
            view.setWantsLayer(YES);
            view.setLayer(mem::transmute(layer.as_ref()));
        }
        layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));
        layer
    };

    let command_queue = device.new_command_queue();

    let backend = unsafe {
        mtl::BackendContext::new(
            device.as_ptr() as mtl::Handle,
            command_queue.as_ptr() as mtl::Handle,
            std::ptr::null(),
        )
    };

    let mut context = DirectContext::new_metal(&backend, None).unwrap();

    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;

    let mut state: State = setup_ecs();
    
    events_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        metal_layer
                            .set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                        window.request_redraw()
                    },
                    WindowEvent::CursorMoved {position, .. } => {
                        mouse_x = position.x;
                        mouse_y = position.y;
                        
                    },
                    WindowEvent::MouseInput {state:button_state, ..} => {
                        if button_state == winit::event::ElementState::Released {

                            let mut changes = Vec::<ModelChanges>::new();
                            let t = 4.0;
                            for (id, entity) in state.get_entities().read().unwrap().iter() {
                                match entity {
                                    Entities::Layer(layer, _, _, _) => {
                                        
                                        changes.push(
                                            layer.position_to(
                                                Point{
                                                    x: mouse_x - 500.0 + rand::random::<f64>() * 1000.0,
                                                    y: mouse_y - 500.0 + rand::random::<f64>() * 1000.0,
                                                },
                                                Some(Transition {
                                                    duration: t*3.0,
                                                    delay: 0.0,
                                                    timing: Easing::default(),
                                                })
                                            )
                                        );
                                        // let s = rand::random::<f64>() * 200.0;
                                        // changes.push(
                                        //     layer.size_to(
                                        //         Point{
                                        //             x: s,
                                        //             y: s,
                                        //         },
                                        //         None
                                        //     )
                                        // );
                                        // changes.push(
                                        //     layer.border_corner_radius_to(
                                        //         BorderRadius::new_single(s/2.0),
                                        //         None
                                        //     )
                                        // );
                                        // changes.push(
                                        //     layer.background_color_to(
                                        //         layer::PaintColor::Solid { color: Color {r: rand::random::<f64>(), g: rand::random::<f64>(), b: rand::random::<f64>(), a: 1.0} },
                                        //         None
                                        //     )
                                        // );
                                        // changes.push(
                                        //     layer.border_width_to(
                                        //         s/10.0,
                                        //         None
                                        //     )
                                        // );
                                    },
                                }
                            }

                            state.add_changes(changes, Some(Transition {
                                duration: t,
                                delay: 0.0,
                                timing: Easing::default(),
                            }));
                        }
                    },
                    _ => (),
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                    state.update(0.016);
                },
                Event::RedrawRequested(_) => {
                    if let Some(drawable) = metal_layer.next_drawable() {
                        let drawable_size = {
                            let size = metal_layer.drawable_size();
                            Size::new(size.width as scalar, size.height as scalar)
                        };

                        let mut surface = unsafe {
                            let texture_info =
                                mtl::TextureInfo::new(drawable.texture().as_ptr() as mtl::Handle);

                            let backend_render_target = BackendRenderTarget::new_metal(
                                (drawable_size.width as i32, drawable_size.height as i32),
                                1,
                                &texture_info,
                            );

                            Surface::from_backend_render_target(
                                &mut context,
                                &backend_render_target,
                                SurfaceOrigin::TopLeft,
                                ColorType::BGRA8888,
                                None,
                                None,
                            )
                            .unwrap()
                        };
                        
                        draw(surface.canvas(), &state);
                        surface.flush_and_submit();
                        drop(surface);

                        let command_buffer = command_queue.new_command_buffer();
                        command_buffer.present_drawable(drawable);
                        command_buffer.commit();
                    }
                }
                _ => {}
            }
        });
    });
}


