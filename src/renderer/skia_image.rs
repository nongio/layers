#![allow(dead_code)]

use skia_safe::Surface;

use std::fs::File;
use std::io::Write;

use std::cell::Cell;

use crate::drawing::scene::{draw_scene, DrawScene};
use crate::engine::scene::Scene;

/// A scene renderer that renders to an image file.
/// Image encoding is currently hard-coded to PNG.
pub struct SkiaImageRenderer {
    pub surface: Surface,
    pub filename: String,
    pub image_format: skia_safe::EncodedImageFormat,
}
impl SkiaImageRenderer {
    pub fn new(height: i32, width: i32, filename: String) -> Self {
        let surface = Surface::new_raster_n32_premul((width, height)).expect("no surface!");
        let image_format = skia_safe::EncodedImageFormat::PNG;
        Self {
            surface,
            filename,
            image_format,
        }
    }

    pub fn create<S: Into<String>>(width: i32, height: i32, filename: S) -> Cell<Self> {
        Cell::new(Self::new(width, height, filename.into()))
    }

    pub fn surface(&mut self) -> &mut Surface {
        &mut self.surface
    }

    pub fn save(&mut self) {
        let mut file = File::create(&self.filename).expect("no file!");
        let image = self.surface.image_snapshot();
        let data = image.encode_to_data(self.image_format).unwrap();
        file.write_all(&data).expect("no write!");
    }
}

impl DrawScene for SkiaImageRenderer {
    fn draw_scene(&mut self, scene: &Scene) {
        let surface = self.surface();
        let c = surface.canvas();
        draw_scene(c, scene);
        surface.flush_and_submit();
    }
}