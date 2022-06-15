use skia_safe::{Canvas, Color4f, Paint, Point, Rect, Size, Font, DeferredDisplayListRecorder, DeferredDisplayList, RRect};
use skia_safe::{Typeface, FontStyle, PaintStyle};


pub fn threadable_draw(canvas: &mut Canvas) -> Option<DeferredDisplayList> {
    unsafe {
        println!("got recording context");
        let canvas_surface = canvas.surface()?;
        
        if let Some(characterization) = canvas_surface.characterize() {
            let mut recorder = DeferredDisplayListRecorder::new(&characterization);
            
            let subcanvas = recorder.canvas();

            // threadable work
            subcanvas.clear(Color4f::new(1.0, 0.0, 0.0, 1.0));
            let canvas_size = Size::from(subcanvas.base_layer_size());
            let rect_size = canvas_size / 2.0;
            
            let rect = Rect::from_point_and_size(
                Point::new(
                    (canvas_size.width - rect_size.width) / 2.0,
                    (canvas_size.height - rect_size.height) / 2.0,
                ),
                rect_size,
            );
            let paint = Paint::new(Color4f::new(0.0, 0.6, 0.0, 1.0), None);
            subcanvas.draw_rect(rect, &paint);
            
            // end threadable
            if let Some(list) = recorder.detach() {

                
                return Some(list);
            }
            

        }
    }
    None
}