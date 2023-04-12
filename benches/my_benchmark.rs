use criterion::{black_box, criterion_group, criterion_main, Criterion};

use layers::{
    engine::animations::{Easing, Transition},
    engine::LayersEngine,
    prelude::*,
    types::*,
};

pub struct Timestamp(f32);

fn criterion_benchmark(c: &mut Criterion) {
    let engine = LayersEngine::new();

    let root = engine.new_layer();
    engine.scene_set_root(root);
    let mut models = Vec::<Layer>::new();
    for _ in 0..1000 {
        let model = engine.new_layer();
        engine.scene_add_layer(model.clone());
        models.push(model);
    }

    for model in models.iter() {
        model.set_size(
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
