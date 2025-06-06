use skia_safe::Contains;
use skia_safe::Point as SkiaPoint;

use crate::types::*;

use super::SceneNode;

pub trait ContainsPoint {
    fn contains(&self, point: Point) -> bool;
}

impl ContainsPoint for SceneNode {
    fn contains(&self, point: Point) -> bool {
        let matrix = self.render_layer.transform_33;
        let inverse = matrix.invert().unwrap();
        let point = inverse.map_point(SkiaPoint::new(point.x, point.y));

        self.render_layer.bounds.contains(point)
    }
}

impl ContainsPoint for Rectangle {
    fn contains(&self, point: Point) -> bool {
        self.x <= point.x
            && self.y <= point.y
            && self.x + self.width >= point.x
            && self.y + self.height >= point.y
    }
}
