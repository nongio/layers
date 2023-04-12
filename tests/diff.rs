use std::{borrow::Borrow, sync::Arc, vec};

use diff::Differ;
use layers::{
    engine::{CommandWithTransition, LayersEngine},
    layers::layer::RenderLayer,
    types::{BorderRadius, BorderStyle, Color, PaintColor, Point, Size},
};

#[derive(Clone, Debug, PartialEq)]
struct Layer {
    pub position: Point,
    pub size: Point,
    // pub background_color: PaintColor,
    // pub border_color: PaintColor,
    pub border_width: f32,
    pub border_style: BorderStyle,
    pub border_corner_radius: BorderRadius,
}

#[derive(Clone, Debug)]
struct LayerNode {
    layer: Layer,
    children: Vec<Layer>,
}
#[test]
pub fn diff_layers() {
    let differ = LayerDiffer {};
    let engine = LayersEngine::new();
    let layer1 = engine.new_layer();
    layer1
        .set_size(Size { x: 100.0, y: 100.0 }, None)
        .set_position(Point { x: 50.0, y: 50.0 }, None)
        .set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#76d7c4"),
            },
            None,
        )
        .set_border_corner_radius(1.0, None);

    let layer2 = engine.new_layer();
    layer2
        .set_size(Size { x: 600.0, y: 600.0 }, None)
        .set_position(Point { x: 50.0, y: 50.0 }, None)
        .set_background_color(
            PaintColor::Solid {
                color: Color::new_hex("#76d7c4"),
            },
            None,
        )
        .set_border_corner_radius(50.0, None);

    let r1 = layer1.into_render_layer();
    let r2 = layer2.into_render_layer();

    let r3 = differ.diff(&r1, &r2);
    println!("{:?}", r3);
}

struct LayerDiffer {}
impl Differ<RenderLayer> for LayerDiffer {
    type Repr = Vec<Arc<dyn CommandWithTransition>>;
    fn diff(&self, a: &RenderLayer, b: &RenderLayer) -> Self::Repr {
        let commands = vec![];
        if a.size != b.size {
            
            commands.push(CommandWithTransition::SetSize {
                size: b.size,
                transition: None,
            });
        }

        commands
    }
    fn apply(&self, a: &mut RenderLayer, b: &Self::Repr) {}
}
// trait ToChanges {
//     fn to_changes(&self) -> Vec<Layer>;
// }
// impl ToChanges for LayerNodeDiff {
//     fn to_changes(&self) -> Vec<Layer> {
//         let mut changes = vec![];
//         if let Some(layer) = &self.layer {
//             changes.push(layer.clone());
//         }
//         if let Some(children) = &self.children {
//             children.0.iter().for_each(|l| {
//                 let l = l.borrow();
//                 changes.append(&mut l.to_changes());
//             });
//         }
//         changes
//     }
// }
