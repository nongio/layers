use std::{
    sync::{Mutex, Once},
    time::Duration,
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use glutin::{
    dpi::LogicalSize, event_loop::EventLoop, platform::unix::EventLoopBuilderExtUnix,
    window::WindowBuilder, GlProfile,
};
use layers::{
    engine::LayersEngine,
    prelude::{timing::TimingFunction, DrawScene, Layer, Transition},
    types::*,
};
fn criterion_benchmark_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_children");
    group.measurement_time(Duration::from_secs(10));
    // Define different numbers of children to test
    let child_counts = [1, 10, 100, 1000, 2000];
    // println!("child_counts: {:?}", child_counts);
    for &count in &child_counts {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let engine = LayersEngine::new(1000.0, 1000.0);

            let root = engine.new_layer();
            engine.scene_set_root(root);
            let mut layers = Vec::<Layer>::new();
            for _ in 0..count {
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
            b.iter(|| engine.update(black_box(0.0)));
        });
    }
}

type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

static INIT: Once = Once::new();
static mut WINDOW_CONTEXT: Option<Mutex<WindowedContext>> = None;
static mut EVENT_LOOP: Option<Mutex<EventLoop<()>>> = None;

fn with_gl_context(f: impl FnOnce(&WindowedContext, &EventLoop<()>)) {
    unsafe {
        INIT.call_once(|| {
            let window_width = 900;
            let window_height = 800;

            let size: LogicalSize<i32> = LogicalSize::new(window_width, window_height);

            // let events_loop = EventLoop::;
            let events_loop = glutin::event_loop::EventLoopBuilder::new()
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
            WINDOW_CONTEXT = Some(Mutex::new(windowed_context));
            EVENT_LOOP = Some(Mutex::new(events_loop));
        });
        let win = &*WINDOW_CONTEXT.as_ref().unwrap().lock().unwrap();
        let el = &*EVENT_LOOP.as_ref().unwrap().lock().unwrap();
        f(win, el);
    }
}

fn criterion_benchmark_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_and_draw_children");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    // Define different numbers of children to test
    let child_counts = [10, 50, 100];
    with_gl_context(|windowed_context, _| {
        let pixel_format = windowed_context.get_pixel_format();
        let size = windowed_context.window().inner_size();
        let sample_count: usize = pixel_format.multisampling.map(|s| s.into()).unwrap_or(0);
        let pixel_format: usize = pixel_format.stencil_bits.into();

        let mut skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
            size.width as i32,
            size.height as i32,
            sample_count,
            pixel_format,
            layers::skia::ColorType::RGBA8888,
            layers::skia::gpu::SurfaceOrigin::BottomLeft,
            0_u32,
        );
        for &count in &child_counts {
            group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
                let engine = LayersEngine::new(2000.0, 2000.0);

                let root = engine.new_layer();
                root.set_size(Size::points(1000.0, 1000.0), None);
                root.set_background_color(Color::new_hex("#ffffff"), None);

                engine.scene_set_root(root.clone());
                let mut layers = Vec::<Layer>::new();
                for i in 0..count {
                    let layer = engine.new_layer();
                    layer.set_layout_style(layers::taffy::Style {
                        position: taffy::Position::Absolute,
                        ..Default::default()
                    });
                    layer.set_size(Size::points(500.0, 500.0), None);
                    layer.set_border_corner_radius(BorderRadius::new_single(10.0), None);
                    layer.set_background_color(Color::new_hex("#ff0000"), None);
                    layer.set_border_color(Color::new_hex("#000000"), None);
                    layer.set_border_width(1.0, None);
                    layer.set_image_cache(true);
                    // let i = i as f32;
                    // layer.set_position((i, i), None);
                    engine.scene_add_layer(layer.clone());
                    layers.push(layer);
                }

                for (i, layer) in layers.iter().enumerate() {
                    let i = i as f32;

                    layer.set_position(
                        (i * 10.0, i * 20.0),
                        Some(Transition {
                            duration: 10.0,
                            delay: 0.0,
                            timing: TimingFunction::default(),
                        }),
                    );
                    layer.set_size(
                        Size::points(500.0 / (i + 1.0), 500.0 / (i + 1.0)),
                        Some(Transition {
                            duration: 20.0,
                            delay: 0.0,
                            timing: TimingFunction::default(),
                        }),
                    );
                }

                b.iter(|| {
                    engine.update(0.016);

                    let root = engine.scene_root().unwrap();
                    let skia_renderer = skia_renderer.get_mut();
                    skia_renderer.draw_scene(engine.scene(), root, None);

                    skia_renderer.gr_context.flush_and_submit();

                    windowed_context.swap_buffers().unwrap();
                })
            });
        }
    });

    group.finish();
}

fn criterion_benchmark_draw_shadow(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_and_draw_children_shadow");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    // Define different numbers of children to test
    let child_counts = [100, 200, 300];
    with_gl_context(|windowed_context, _| {
        let pixel_format = windowed_context.get_pixel_format();
        let size = windowed_context.window().inner_size();
        let sample_count: usize = pixel_format.multisampling.map(|s| s.into()).unwrap_or(0);
        let pixel_format: usize = pixel_format.stencil_bits.into();

        let mut skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
            size.width as i32,
            size.height as i32,
            sample_count,
            pixel_format,
            layers::skia::ColorType::RGBA8888,
            layers::skia::gpu::SurfaceOrigin::BottomLeft,
            0_u32,
        );
        for &count in &child_counts {
            group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
                let engine = LayersEngine::new(2000.0, 2000.0);

                let root = engine.new_layer();
                root.set_size(Size::points(1000.0, 1000.0), None);

                root.set_background_color(Color::new_hex("#ffffff"), None);

                engine.scene_set_root(root.clone());
                let mut layers = Vec::<Layer>::new();
                for i in 0..count {
                    let layer = engine.new_layer();
                    layer.set_layout_style(layers::taffy::Style {
                        position: taffy::Position::Absolute,
                        ..Default::default()
                    });
                    layer.set_size(Size::points(500.0, 500.0), None);
                    layer.set_border_corner_radius(BorderRadius::new_single(10.0), None);
                    layer.set_background_color(Color::new_hex("#ff0000"), None);
                    layer.set_border_color(Color::new_hex("#000000"), None);
                    layer.set_border_width(1.0, None);
                    layer.set_shadow_color(Color::new_rgba(0.0, 0.0, 0.0, 0.2), None);
                    layer.set_shadow_radius(10.0, None);
                    layer.set_shadow_spread(5.0, None);
                    layer.set_shadow_offset((0.0, 0.0), None);
                    layer.set_image_cache(true);
                    let i = i as f32;
                    layer.set_position((0.0, 0.0), None);
                    engine.scene_add_layer(layer.clone());
                    layers.push(layer);
                }

                for (i, layer) in layers.iter().enumerate() {
                    let i = i as f32;

                    layer.set_position(
                        (i * 10.0, i * 10.0),
                        Some(Transition {
                            duration: 100.0,
                            delay: 0.0,
                            timing: TimingFunction::default(),
                        }),
                    );
                    layer.set_size(
                        Size::points(500.0 / (i + 1.0), 500.0 / (i + 1.0)),
                        Some(Transition {
                            duration: 20.0,
                            delay: 0.0,
                            timing: TimingFunction::default(),
                        }),
                    );
                }
                // println!("running bench...");

                b.iter(|| {
                    engine.update(0.016);

                    let root = engine.scene_root().unwrap();
                    let skia_renderer = skia_renderer.get_mut();
                    skia_renderer.draw_scene(engine.scene(), root, None);

                    skia_renderer.gr_context.flush_and_submit();

                    windowed_context.swap_buffers().unwrap();
                })
            });
        }
    });

    group.finish();
}
criterion_group!(
    benches,
    // criterion_benchmark_update,
    // criterion_benchmark_draw,
    criterion_benchmark_draw_shadow
);
criterion_main!(benches);
