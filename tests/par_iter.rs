#[cfg(test)]
mod tests {

    use glutin::{
        dpi::LogicalSize,
        event_loop::EventLoop,
        platform::{run_return::EventLoopExtRunReturn, unix::EventLoopBuilderExtUnix},
        window::WindowBuilder,
        GlProfile,
    };
    use layers::{
        engine::LayersEngine,
        prelude::{DrawScene, Layer, TimingFunction, Transition},
        types::*,
    };
    type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

    fn initialize_opengl() -> (WindowedContext, EventLoop<()>) {
        // use glutin::dpi::LogicalSize;
        let window_width = 900;
        let window_height = 800;

        let size: LogicalSize<i32> = LogicalSize::new(window_width, window_height);

        // let events_loop = EventLoop::;
        let mut events_loop = glutin::event_loop::EventLoopBuilder::new()
            .with_any_thread(true)
            .build();

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

        (windowed_context, events_loop)
    }
    #[test]
    pub fn update_multiple_children() {
        let engine = LayersEngine::new(1000.0, 1000.0);

        let root = engine.new_layer();
        engine.scene_set_root(root);
        let mut layers = Vec::<Layer>::new();
        for _ in 0..1000 {
            let layer = engine.new_layer();
            engine.scene_add_layer(layer.clone());
            layers.push(layer);
        }

        for layer in layers.iter() {
            layer.set_size(
                Size::points(1000.0, 1000.0),
                Some(Transition {
                    duration: 10000.0,
                    delay: 0.0,
                    timing: TimingFunction::default(),
                }),
            );
            layer.set_opacity(
                0.5,
                Some(Transition {
                    duration: 10000.0,
                    delay: 0.0,
                    timing: TimingFunction::default(),
                }),
            );
        }
        engine.update(0.0083);
    }

    #[test]
    pub fn draw_multiple_children() {
        let (windowed_context, mut events_loop) = initialize_opengl();
        let pixel_format = windowed_context.get_pixel_format();
        let size = windowed_context.window().inner_size();
        let sample_count: usize = pixel_format.multisampling.map(|s| s.into()).unwrap_or(0);
        let pixel_format: usize = pixel_format.stencil_bits.into();
        let engine = LayersEngine::new(1000.0, 1000.0);
        let mut skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
            size.width as i32,
            size.height as i32,
            sample_count,
            pixel_format,
            layers::skia::ColorType::RGBA8888,
            layers::skia::gpu::SurfaceOrigin::BottomLeft,
            0_u32,
        );
        let root = engine.new_layer();
        root.set_size(Size::points(1000.0, 1000.0), None);
        root.set_background_color(Color::new_hex("#ffffff"), None);
        engine.scene_set_root(root);
        let mut layers = Vec::<Layer>::new();
        for i in 0..1000 {
            let layer = engine.new_layer();
            layer.set_background_color(Color::new_rgba(1.0, 0.0, 0.0, 1.0), None);
            let i = i as f32;
            layer.set_position((i * 10.0, i * 10.0), None);
            engine.scene_add_layer(layer.clone());
            layers.push(layer);
        }

        for layer in layers.iter() {
            layer.set_size(
                Size::points(1000.0, 1000.0),
                Some(Transition {
                    duration: 10000.0,
                    delay: 0.0,
                    timing: TimingFunction::default(),
                }),
            );
            layer.set_opacity(
                0.5,
                Some(Transition {
                    duration: 10000.0,
                    delay: 0.0,
                    timing: TimingFunction::default(),
                }),
            );
        }

        engine.update(0.0083);

        // events_loop.run_return(|event, _, control_flow| {
        //     *control_flow = glutin::event_loop::ControlFlow::Poll;
        //     match event {
        //         glutin::event::Event::MainEventsCleared => {}
        //         glutin::event::Event::NewEvents(clause) => {
        //             println!("{:?}", clause);
        //         }
        //         glutin::event::Event::RedrawRequested(_) => {
        let root = engine.scene_root().unwrap();
        let skia_renderer = skia_renderer.get_mut();
        skia_renderer.draw_scene(engine.scene(), root, None);

        skia_renderer.gr_context.flush_submit_and_sync_cpu();
        //         }
        //         glutin::event::Event::RedrawEventsCleared => {
        //             windowed_context.swap_buffers().unwrap();
        //         }
        //         _ => {
        //             println!("{:?}", event);
        //         }
        //     }
        // });
        windowed_context.swap_buffers().unwrap();
    }
}
