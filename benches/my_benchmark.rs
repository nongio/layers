use criterion::{black_box, criterion_group, criterion_main, Criterion};

use hello::{
    engine::animations::{Easing, Transition},
    engine::node::RenderNode,
    engine::scene::Scene,
    engine::Engine,
    layers::layer::ModelLayer,
    types::Point,
};
use std::sync::Arc;
pub struct Timestamp(f64);

fn criterion_benchmark(c: &mut Criterion) {
    let engine = Engine::create();

    let mut models = Vec::<Arc<ModelLayer>>::new();
    for _ in 0..1000 {
        let model = ModelLayer::create();
        engine.scene.add(model.clone() as Arc<dyn RenderNode>);
        models.push(model);
    }

    for model in models.iter() {
        model.size(
            Point { x: 100.0, y: 100.0 },
            Some(Transition {
                duration: 10000.0,
                delay: 0.0,
                timing: Easing::default(),
            }),
        );
    }

    c.bench_function("update", |b| b.iter(|| engine.update(black_box(0.001))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
