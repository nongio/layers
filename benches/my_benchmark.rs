use criterion::{black_box, criterion_group, criterion_main, Criterion};

use hello::engine::animations::{Easing, Transition};
use hello::engine::scene::Scene;
use hello::layers::layer::{ModelLayer, ModelLayerRef};
use hello::types::Point;
// use hello::layers::layer::{BorderRadius, Color, ModelChanges, PaintColor, Point};
pub struct Timestamp(f64);

fn criterion_benchmark(c: &mut Criterion) {
    let scene = Scene::create();

    let mut models = Vec::<ModelLayerRef>::new();
    for _ in 0..1000 {
        let model = ModelLayer::create();
        scene.add_renderable(model.clone());
        models.push(model);
    }

    for model in models.iter() {
        model.position(
            Point { x: 100.0, y: 100.0 },
            Some(Transition {
                duration: 10000.0,
                delay: 0.0,
                timing: Easing::default(),
            }),
        );
    }

    c.bench_function("update", |b| b.iter(|| scene.update(black_box(0.001))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
