
use skia_safe::{Canvas, Color4f, Paint, Point, Rect, Size, Font, DeferredDisplayListRecorder, DeferredDisplayList, RRect};
use skia_safe::{Typeface, FontStyle, PaintStyle};

use crate::layer::{RenderLayer, PaintColor, Color, BorderStyle, BorderRadius};



fn render_layer(canvas: &mut Canvas, layer: RenderLayer) {

    let rect = Rect::from_point_and_size(
        (layer.position.x as f32, layer.position.y as f32), 
    (layer.size.x as f32, layer.size.y as f32));
    let rrect = RRect::new_rect_radii(
        rect,
        &[
        Point::new(layer.border_corner_radius.top_left, layer.border_corner_radius.top_left),
        Point::new(layer.border_corner_radius.top_right, layer.border_corner_radius.top_right),
        Point::new(layer.border_corner_radius.bottom_left,layer.border_corner_radius.bottom_left),
        Point::new(layer.border_corner_radius.bottom_right, layer.border_corner_radius.bottom_right)
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
pub fn draw(canvas: &mut Canvas) {
    let canvas_size = Size::from(canvas.base_layer_size());
    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));    
    
    let paint = Paint::new(Color4f::new(0.0, 0.0, 0.6, 1.0), None);
    let typeface = Typeface::new("HelveticaNeue", FontStyle::normal()).unwrap();
    let font = Font::new(typeface, 72.0);
    let size_str = format!("{}x{}", canvas_size.width, canvas_size.height);
    canvas.draw_str(size_str, Point::new(10.0, 70.0), &font, &paint);

    let layer = RenderLayer {
        background_color: PaintColor::Solid { color: Color::new(0.6, 0.0, 0.0, 1.0)},
        border_color: PaintColor::Solid { color: Color::new(0.0, 0.0, 0.0, 1.0)},
        border_corner_radius: BorderRadius{
            top_left: 20.0,
            top_right: 20.0,
            bottom_left: 10.0,
            bottom_right: 10.0,
        },
        border_style: BorderStyle::Solid,
        border_width: 4.0,
        position: crate::layer::Point{x:100.0, y:100.0},
        size: crate::layer::Point{x:200.0, y:200.0},
    };

    render_layer(canvas, layer);
}