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
