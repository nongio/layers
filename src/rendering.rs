use skia_safe::{Canvas, Color4f, Font, Paint, Point, RRect, Rect, Size};
use skia_safe::{FontStyle, PaintStyle, Typeface};

use crate::ecs::entities::HasHierarchy;
use crate::ecs::{entities::Entities, State};
use crate::layer::{PaintColor, RenderLayer};

pub fn render_layer(canvas: &mut Canvas, layer: &RenderLayer) {
    let rect = Rect::from_point_and_size(
        (0.0, 0.0),
        // (layer.position.x as f32, layer.position.y as f32),
        (layer.size.x as f32, layer.size.y as f32),
    );
    let rrect = RRect::new_rect_radii(
        rect,
        &[
            Point::new(
                layer.border_corner_radius.top_left as f32,
                layer.border_corner_radius.top_left as f32,
            ),
            Point::new(
                layer.border_corner_radius.top_right as f32,
                layer.border_corner_radius.top_right as f32,
            ),
            Point::new(
                layer.border_corner_radius.bottom_left as f32,
                layer.border_corner_radius.bottom_left as f32,
            ),
            Point::new(
                layer.border_corner_radius.bottom_right as f32,
                layer.border_corner_radius.bottom_right as f32,
            ),
        ],
    );

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

pub fn draw_single_entity(canvas: &mut Canvas, entity: &Entities) {
    match entity {
        Entities::Layer { layer, cache, .. } => {
            if let Some(picture) = cache.write().unwrap().picture.clone() {
                let r = layer.read().unwrap();
                canvas.concat(&r.matrix);
                canvas.draw_picture(picture, None, None);
            } else {
                let render = layer.clone();
                let render = render.read().unwrap();
                render_layer(canvas, &render);
            }
        }
        _ => {}
    }
}
pub fn draw_entity(canvas: &mut Canvas, entity: &Entities) {
    canvas.save();
    draw_single_entity(canvas, entity);
    for child in entity.children().iter() {
        draw_entity(canvas, child);
    }
    canvas.restore();
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

    draw_entity(canvas, &state.root);
}
