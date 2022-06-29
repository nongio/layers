


use skia_safe::{Canvas, Color4f, Paint, Point, Rect, Size, Font, Matrix, RRect};
use skia_safe::{Typeface, FontStyle, PaintStyle};

use crate::layer::{RenderLayer, PaintColor};
use crate::ecs::{State, Entities};

pub fn render_layer(canvas: &mut Canvas, layer: &RenderLayer) {

    let rect = Rect::from_point_and_size(
        (0.0, 0.0),
        // (layer.position.x as f32, layer.position.y as f32), 
    (layer.size.x as f32, layer.size.y as f32));
    let rrect = RRect::new_rect_radii(
        rect,
        &[
        Point::new(layer.border_corner_radius.top_left as f32, layer.border_corner_radius.top_left as f32),
        Point::new(layer.border_corner_radius.top_right as f32, layer.border_corner_radius.top_right as f32),
        Point::new(layer.border_corner_radius.bottom_left as f32,layer.border_corner_radius.bottom_left as f32),
        Point::new(layer.border_corner_radius.bottom_right as f32, layer.border_corner_radius.bottom_right as f32)
    ]);

    let mut paint = match layer.background_color {
        PaintColor::Solid { color } => Paint::new(Color4f::from(color), None),
        _ => Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
    };
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    canvas.draw_rrect(rrect, &paint);


    paint = match layer.border_color {
        PaintColor::Solid { color } => Paint::new(Color4f::from(color), None),
        _ => Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
    };
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(layer.border_width as f32);

    canvas.draw_rrect(rrect, &paint);
}

/// Renders a rectangle that occupies exactly half of the canvas
pub fn draw(canvas: &mut Canvas, state: &State) {
    let canvas_size = Size::from(canvas.base_layer_size());
    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));    
    
    let paint = Paint::new(Color4f::new(0.0, 0.0, 0.6, 1.0), None);
    let typeface = Typeface::new("HelveticaNeue", FontStyle::normal()).unwrap();
    let font = Font::new(typeface, 72.0);
    let fps = format!("{}", state.fps as u32);
    
    canvas.draw_str(fps, Point::new(10.0, 70.0), &font, &paint);

    for (id, entity) in state.get_entities().read().unwrap().iter() {
        match entity {
            Entities::Layer(layer, render, cache) => {
                if let Some(picture) = cache.picture.clone() {
                    canvas.draw_picture (picture, Some(&Matrix::translate((render.position.x as f32, render.position.y as f32))), None);
                } else {
                    render_layer(canvas, render);
                }
            },
            _ => {},
            
        }
    }
}