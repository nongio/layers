use std::time::Duration;

use gl::types::{GLint, GLsizei, GLvoid};
use gl_rs as gl;
use glutin::{
    event::{Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile,
};

use layers::{
    prelude::{timing::TimingFunction, *},
    skia::Color4f,
};
use layers::{
    skia::{
        self,
        gpu::{
            self,
            gl::{FramebufferInfo, TextureInfo},
            BackendTexture,
        },
        ColorType, Paint, PixelGeometry, Surface, SurfaceProps, SurfacePropsFlags,
    },
    types::Size,
};
use winit::window::Icon;

pub fn draw(canvas: &mut skia::Canvas) {
    let paint = Paint::new(Color4f::new(1.0, 0.0, 0.0, 1.0), None);
    canvas.clear(Color4f::new(0.7, 0.7, 0.7, 1.0));

    let bounds = skia::Rect::from_xywh(100.0, 100.0, 400.0, 200.0);
    let rect_paint = skia::Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    canvas.draw_rect(bounds, &rect_paint);

    let mut text_style = skia::textlayout::TextStyle::new();
    text_style.set_font_size(60.0);
    let foreground_paint = skia::Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
    text_style.set_foreground_color(&foreground_paint);
    text_style.set_font_families(&["Inter"]);
    // let background_paint = skia::Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    // text_style.set_background_color(&background_paint);

    let font_mgr = skia::FontMgr::new();
    let type_face_font_provider = skia::textlayout::TypefaceFontProvider::new();
    let mut font_collection = skia::textlayout::FontCollection::new();
    font_collection.set_asset_font_manager(Some(type_face_font_provider.clone().into()));
    font_collection.set_dynamic_font_manager(font_mgr.clone());

    let mut paragraph_style = skia::textlayout::ParagraphStyle::new();
    paragraph_style.set_text_style(&text_style);
    paragraph_style.set_max_lines(1);
    paragraph_style.set_text_align(skia::textlayout::TextAlign::Center);
    paragraph_style.set_text_direction(skia::textlayout::TextDirection::LTR);
    paragraph_style.set_ellipsis("…");
    let mut paragraph = skia::textlayout::ParagraphBuilder::new(&paragraph_style, font_collection)
        .add_text("Hello World! 👋🌍")
        .build();
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
        size.width as i32,
        size.height as i32,
        sample_count,
        pixel_format,
        ColorType::RGBA8888,
        layers::skia::gpu::SurfaceOrigin::BottomLeft,
        0_u32,
    );

    let mut _mouse_x = 0.0;
    let mut _mouse_y = 0.0;

    struct Env {
        windowed_context: WindowedContext,
    }
    let env = Env { windowed_context };
    let engine = LayersEngine::new(window_width as f32 * 2.0, window_height as f32 * 2.0);
    let root_layer = engine.new_layer();

    // root_layer.set_size(
    //     layers::types::Size {
    //         x: window_width as f32 * 2.0,
    //         y: window_height as f32 * 2.0,
    //     },
    //     None,
    // );
    root_layer.set_position(Point { x: 0.0, y: 0.0 }, None);

    root_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 180, 255),
        },
        None,
    );
    root_layer.set_border_corner_radius(10.0, None);

    engine.scene_add_layer(root_layer.clone());

    let container = engine.new_layer();
    container.set_position(layers::types::Point { x: 0.0, y: 0.0 }, None);
    container.set_size(layers::types::Size::points(450.0, 500.0), None);
    container.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(255, 255, 255, 255),
        },
        None,
    );
    container.set_layout_style(taffy::Style {
        display: taffy::Display::Flex,
        position: taffy::Position::Absolute,

        flex_direction: taffy::FlexDirection::Row,
        justify_content: Some(taffy::JustifyContent::Center),
        flex_wrap: taffy::FlexWrap::Wrap,
        align_items: Some(taffy::AlignItems::Baseline),
        align_content: Some(taffy::AlignContent::FlexStart),
        gap: taffy::points(2.0),

        size: layers::taffy::prelude::Size {
            width: taffy::points(450.0),
            height: taffy::points(500.0),
        },
        ..Default::default()
    });

    engine.scene_add_layer(container.clone());

    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;

    // load an image
    let img = image::open("assets/fill.png").unwrap();

    // Get the image data as a byte slice
    let img_data = img.to_rgba8().into_raw();

    // Create a new GL texture
    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
    }

    // Upload the image data to the texture
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            img.width() as GLsizei,
            img.height() as GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img_data.as_ptr() as *const GLvoid,
        );
    }

    // Set texture parameters
    unsafe {
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as GLint,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    }

    let texture_info = TextureInfo {
        target: gl::TEXTURE_2D as u32,
        id: texture,
        format: gpu::gl::Format::RGBA8.into(),
    };

    // Create a Skia framebuffer info
    let framebuffer_info = FramebufferInfo {
        format: gpu::gl::Format::RGBA8.into(),
        fboid: 0,
    };

    let backend_texture = unsafe {
        BackendTexture::new_gl(
            (img.width() as i32, img.height() as i32),
            gpu::MipMapped::No,
            texture_info,
        )
    };
    let mut _gr_context: gpu::DirectContext = gpu::DirectContext::new_gl(None, None).unwrap();
    let skr = skia_renderer.get_mut();
    let mut surface = skr.surface();

    let image = Image::from_texture(
        &mut surface.canvas().recording_context().unwrap(),
        &backend_texture,
        layers::skia::gpu::SurfaceOrigin::TopLeft,
        ColorType::RGBA8888,
        layers::skia::AlphaType::Unpremul,
        None,
    )
    .unwrap();

    let picture = {
        let mut recorder = skia::PictureRecorder::new();
        let canvas = recorder.begin_recording(skia::Rect::from_wh(500.0, 500.0), None);
        let mut paint = Paint::new(Color4f::new(1.0, 0.0, 0.0, 1.0), None);

        paint.set_anti_alias(true);

        let mut font = skia::Font::default();
        font.set_size(50.0);
        let text = "Hello text";
        // canvas.draw_str(text, (0, 50), &font, &paint);

        let mut paint = skia::Paint::new(skia::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(skia::paint::Style::Fill);

        let mut font = skia::Font::default();
        font.set_size(30.0);
        // let text = state.name.as_bytes();
        let mut text_style = skia::textlayout::TextStyle::new();
        text_style.set_font_size(100.0);
        let background_paint = skia::Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        let foreground_paint = skia::Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
        text_style.set_background_color(&background_paint);
        text_style.set_foreground_color(&foreground_paint);
        text_style.set_font_families(&["Arial"]);
        let c = text_style.color();
        println!("{} {} {} {}", c.r(), c.g(), c.b(), c.a());
        let mut paragraph_style = skia::textlayout::ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);
        // paragraph_style.set_max_lines(1);
        // paragraph_style.set_text_align(skia::textlayout::TextAlign::Center);
        // paragraph_style.set_text_direction(skia::textlayout::TextDirection::LTR);

        let font_collection = skia::textlayout::FontCollection::new();
        let mut paragraph =
            skia::textlayout::ParagraphBuilder::new(&paragraph_style, font_collection)
                .add_text("Hello text")
                .build();
        paragraph.layout(200.0);
        paragraph.paint(canvas, (0.0, 50.0));

        recorder.finish_recording_as_picture(None)
    };
    container.set_content(picture, None);

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
                    let _transition = root_layer.set_size(
                        Size::points(size.width as f32, size.height as f32),
                        Some(Transition {
                            duration: 1.0,
                            delay: 0.0,
                            timing: TimingFunction::default(),
                        }),
                    );
                    env.windowed_context.window().request_redraw();
                }
                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    match input.virtual_keycode {
                        Some(keycode) => match keycode {
                            winit::event::VirtualKeyCode::Space => {
                                let dt = 0.016;
                                let needs_redraw = engine.update(dt);
                                if needs_redraw {
                                    env.windowed_context.window().request_redraw();
                                    // draw_frame = -1;
                                }
                            }
                            _ => (),
                        },
                        None => (),
                    }
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
                        let _i = 0;
                    } else {
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let now = instant.elapsed().as_secs_f64();
                let frame_number = (now / 0.016).floor() as i32;
                if update_frame != frame_number {
                    update_frame = frame_number;
                    let dt = 0.016;
                    // let needs_redraw = engine.update(dt);
                    // if needs_redraw {
                    env.windowed_context.window().request_redraw();
                    // draw_frame = -1;
                    // }
                }
            }
            Event::RedrawRequested(_) => {
                if draw_frame != update_frame {
                    if let Some(root) = engine.scene_root() {
                        let skia_renderer = skia_renderer.get_mut();
                        // skia_renderer.draw_scene(engine.scene(), root);
                        let mut surface = skia_renderer.surface();
                        let canvas = surface.canvas();
                        draw(canvas);
                        // let ii = image.image_info();
                        // // println!("image info: {:?}", ii);
                        // let paint = Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
                        // canvas.draw_image(image, (10.0, 10.0), Some(&paint));
                        surface.flush_and_submit();
                    }

                    // this will be blocking until the GPU is done with the frame
                    env.windowed_context.swap_buffers().unwrap();
                    draw_frame = update_frame;
                } else {
                    println!("skipping draw");
                }
            }
            _ => {}
        }
        // });
    });
}
