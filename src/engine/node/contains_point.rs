use skia_safe::Point as SkiaPoint;

use crate::types::*;

use super::SceneNode;

pub trait ContainsPoint {
    fn contains(&self, point: Point) -> bool;
}

impl ContainsPoint for SceneNode {
    fn contains(&self, point: Point) -> bool {
        let matrix = self.transformation.read().unwrap();
        let inverse = matrix.invert().unwrap();
        let point = inverse.map_point(SkiaPoint::new(point.x, point.y));
        let point = Point {
            x: point.x,
            y: point.y,
        };
        self.model.bounds().contains(point)
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
