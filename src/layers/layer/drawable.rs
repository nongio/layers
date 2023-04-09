use skia_safe::Canvas;

use crate::{drawing::layer::draw_layer, engine::rendering::Drawable, types::*};

use super::{ModelLayer, RenderLayer};

impl Drawable for ModelLayer {
    fn draw(&self, canvas: &mut Canvas) {
        let layer: RenderLayer = RenderLayer::from(self);
        draw_layer(canvas, &layer);
    }
    fn bounds(&self) -> Rectangle {
        let s = self.size.value();
        let p = self.position.value();
        Rectangle {
            x: p.x,
            y: p.y,
            width: s.x,
            height: s.y,
        }
    }
    fn scaled_bounds(&self) -> Rectangle {
        let s = self.size.value();
        let scale = self.scale.value();

        Rectangle {
            x: 0.0,
            y: 0.0,
            width: s.x * scale.x,
            height: s.y * scale.y,
        }
    }
    fn transform(&self) -> Matrix {
        let s = self.scale.value();
        let p = self.position.value();
        let rotation = self.rotation.value();
        let anchor_point = self.anchor_point.value();
        let size = self.size.value();
        let anchor_translate =
            M44::translate(-anchor_point.x * size.x, -anchor_point.y * size.y, 0.0);
        let identity = M44::new_identity();
        let translate = M44::translate(p.x, p.y, 0.0);
        let _scale = M44::scale(s.x, s.y, 1.0);
        let rotate_x = M44::rotate(
            V3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            rotation.x,
        );
        let rotate_y = M44::rotate(
            V3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            rotation.y,
        );
        let rotate_z = M44::rotate(
            V3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            rotation.z,
        );
        // merge all transforms keeping into account the anchor point
        let transform = M44::concat(&translate, &identity);
        // let transform = M44::concat(&transform, &scale);
        let transform = M44::concat(&transform, &rotate_x);
        let transform = M44::concat(&transform, &rotate_y);
        let transform = M44::concat(&transform, &rotate_z);
        let transform = M44::concat(&transform, &anchor_translate);

        transform.to_m33()
    }
    fn scale(&self) -> (f32, f32) {
        let s = self.scale.value();
        (s.x, s.y)
    }
}
