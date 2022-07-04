use hello::ecs::{entities::*, State};
use hello::layer::*;
use skia_safe::{Color4f, Matrix};
use std::sync::{Arc, RwLock};

#[test]
pub fn test_entites() {
    let mut state = State::new();

    let model = ModelLayer::new();
    let entity = Entities::new_layer(model.clone());

    state.add_entity(entity.clone());
    assert_eq!(entity.parent().is_none(), false);

    // println!("{:?}", entity.parent());

    let ee = entity.clone();

    match ee {
        Entities::Layer { layer, .. } => {
            let r = RenderLayer {
                position: Point { x: 0.0, y: 0.0 },
                background_color: PaintColor::Solid {
                    color: Color::new(0.0, 0.0, 0.0, 1.0),
                },
                border_color: PaintColor::Solid {
                    color: Color::new(0.0, 0.0, 0.0, 1.0),
                },
                border_style: BorderStyle::Solid,
                border_width: 33.0,
                border_corner_radius: BorderRadius::new_single(0.0),
                size: Point { x: 100.0, y: 100.0 },
                matrix: Matrix::new_identity(),
            };
            *layer.write().unwrap() = r;
        }
        _ => panic!("Wrong type"),
    }

    let child_entity = Entities::new_layer(ModelLayer::new());

    // entity.add_child(child_entity.clone());

    // state.add_entity(child_entity);

    let mut e = state.model_storage.get(entity.id()).unwrap();

    e.add_child(&mut child_entity.clone());

    state.add_entity(child_entity.clone());
    // let prop = &model.properties["border_width"];

    // state.update(0.016);
    assert!(entity.children().contains(&child_entity));

    assert_eq!(state.root.children().contains(&child_entity), false);

    assert_eq!(state.root.children().len(), 1);

    match e {
        Entities::Layer { layer, .. } => {
            assert_eq!(layer.read().unwrap().border_width, 33.0);
        }
        _ => panic!("Wrong type"),
    }
}
