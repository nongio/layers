use crate::prelude::ContentDrawFunction;

#[allow(dead_code)]
pub fn save_image<'a>(
    context: impl Into<Option<&'a mut skia_safe::gpu::DirectContext>>,
    image: &skia_safe::Image,
    name: &str,
) {
    use std::fs::File;
    use std::io::Write;

    let data = image
        .encode(context.into(), skia_safe::EncodedImageFormat::PNG, None)
        .unwrap();
    let bytes = data.as_bytes();
    let filename = format!("{}.png", name);
    let mut file = File::create(filename).unwrap();
    file.write_all(bytes).unwrap();
}

// pub fn svg_dom(
//     image_path: &str,
//     size: impl Into<skia_safe::ISize>,
// ) -> Result<skia_safe::svg::Dom, String> {
//     let svg_data =
//         std::fs::read(image_path).map_err(|e| format!("Failed to read SVG file: {}", e))?;

//     let size: skia_safe::ISize = size.into();
//     let options = usvg::Options {
//         resources_dir: None,
//         dpi: 96.0,
//         // Default font is user-agent dependent so we can use whichever we like.
//         font_family: "Times New Roman".to_owned(),
//         font_size: 12.0,
//         languages: vec!["en".to_string()],
//         shape_rendering: usvg::ShapeRendering::default(),
//         text_rendering: usvg::TextRendering::default(),
//         image_rendering: usvg::ImageRendering::default(),
//         default_size: usvg::Size::from_wh(size.width as f32, size.height as f32).unwrap(),
//         image_href_resolver: usvg::ImageHrefResolver::default(),
//     };
//     let mut rtree = usvg::Tree::from_data(&svg_data, &options)
//         .map_err(|e| format!("Failed to parse SVG data: {}", e))?;
//     rtree.size = usvg::Size::from_wh(size.width as f32, size.height as f32).unwrap();
//     let xml_options = usvg::XmlOptions::default();
//     let xml = usvg::TreeWriting::to_string(&rtree, &xml_options);
//     let resource_provider = SvgResourceProvider {};
//     let mut svg = skia_safe::svg::Dom::from_bytes(xml.as_bytes(), resource_provider)
//         .map_err(|e| format!("Failed to create SVG DOM: {}", e))?;
//     svg.set_container_size((size.width as f32, size.height as f32));
//     // println!("SVG DOM created {} \n {:?}", image_path, svg.root());
//     Ok(svg)
// }
pub fn load_svg_image(
    image_path: &str,
    size: impl Into<skia_safe::ISize>,
) -> Result<skia_safe::Image, String> {
    let size: skia_safe::ISize = size.into();

    println!("Loading image from path: {}", image_path);
    println!("Loading image with size: {:?}", size);

    // let svg = svg_dom(image_path, size)?;

    let svg_data =
        std::fs::read(image_path).map_err(|e| format!("Failed to read SVG file: {}", e))?;

    let pixmap_size =
        resvg::tiny_skia::IntSize::from_wh(size.width as u32, size.height as u32).unwrap();

    let options = usvg::Options {
        languages: vec!["en".to_string()],
        dpi: 1.0,
        default_size: usvg::Size::from_wh(pixmap_size.width() as f32, pixmap_size.height() as f32)
            .unwrap(),
        ..Default::default()
    };
    let rtree = usvg::Tree::from_data(&svg_data, &options)
        .map_err(|e| format!("Failed to parse SVG data: {}", e))?;

    let size = rtree.size().to_int_size();

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    let transform = resvg::tiny_skia::Transform::from_scale(
        pixmap_size.width() as f32 / size.width() as f32,
        pixmap_size.height() as f32 / size.height() as f32,
    );
    resvg::render(&rtree, transform, &mut pixmap.as_mut());

    let info = skia_safe::ImageInfo::new(
        (pixmap_size.width() as i32, pixmap_size.height() as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );
    let image = skia_safe::images::raster_from_data(
        &info,
        &skia_safe::Data::new_copy(pixmap.data()),
        pixmap_size.width() as usize * 4,
    )
    .unwrap();

    Ok(image)
}

pub fn draw_image_content(image: &skia_safe::Image) -> ContentDrawFunction {
    let resampler = skia_safe::CubicResampler::catmull_rom();

    let img = image.clone();
    let draw_function = move |canvas: &skia_safe::Canvas, w: f32, h: f32| -> skia_safe::Rect {
        let paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        canvas.draw_image_rect_with_sampling_options(
            &img,
            None,
            skia_safe::Rect::from_xywh(0.0, 0.0, w, h),
            resampler,
            &paint,
        );
        skia_safe::Rect::from_xywh(0.0, 0.0, w, h)
    };
    draw_function.into()
}
