use criterion::{black_box, criterion_group, criterion_main, Criterion};

use hello::ecs::animations::{Easing, Transition};
use hello::ecs::{setup_ecs, Entities};
use hello::layer::{BorderRadius, Color, ModelChanges, PaintColor, Point};

pub struct Timestamp(f64);

fn criterion_benchmark(c: &mut Criterion) {
    let mut state = setup_ecs();

    let mut changes = Vec::<ModelChanges>::new();
    for (_, entity) in state.get_entities().read().unwrap().iter() {
        match entity {
            Entities::Layer(layer, _, _, _) => {
                changes.push(layer.position_to(
                    Point {
                        x: rand::random::<f64>() * 1000.0,
                        y: rand::random::<f64>() * 1000.0,
                    },
                    None,
                ));
                changes.push(layer.background_color_to(
                    PaintColor::Solid {
                        color: Color {
                            r: rand::random::<f64>(),
                            g: rand::random::<f64>(),
                            b: rand::random::<f64>(),
                            a: 1.0,
                        },
                    },
                    None,
                ));

                changes.push(layer.size_to(
                    Point {
                        x: rand::random::<f64>() * 200.0,
                        y: rand::random::<f64>() * 200.0,
                    },
                    None,
                ));
                changes.push(layer.border_corner_radius_to(
                    BorderRadius::new_single(rand::random::<f64>() * 200.0),
                    None,
                ));
            }
        }
    }

    state.add_changes(
        changes,
        Some(Transition {
            duration: 20000.0,
            delay: 0.0,
            timing: Easing::default(),
        }),
    );

    c.bench_function("update", |b| b.iter(|| state.update(black_box(0.001))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
