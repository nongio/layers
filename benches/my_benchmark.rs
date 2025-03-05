use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use ::taffy::Style;
use lay_rs::{engine::LayersEngine, prelude::*, types::*};

#[allow(dead_code)]
fn criterion_benchmark_update(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("engine::update");
    group.measurement_time(Duration::from_secs(10));
    // Define different numbers of children to test
    let child_counts = [1, 10, 100, 1000];

    for &count in &child_counts {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            rt.block_on(async {
                let engine = LayersEngine::new(2048.0, 2048.0);
                let root = engine.new_layer();
                engine.scene_set_root(root);
                let mut layers = Vec::<Layer>::new();
                for _ in 0..count {
                    let layer = engine.new_layer();
                    engine.add_layer(layer.clone());
                    layers.push(layer);
                }
                for layer in layers.iter() {
                    layer.set_size(
                        Size::points(1000.0, 1000.0),
                        Some(Transition::ease_out_quad(10000.0)),
                    );
                    layer.set_opacity(0.5, Some(Transition::ease_in_quad(10000.0)));
                }
                b.iter(|| engine.update(black_box(0.016)));
            });
        });
    }
}

#[allow(dead_code)]
fn criterion_benchmark_append(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("engine::append");
    group.measurement_time(Duration::from_secs(10));
    // Define different numbers of children to test
    let child_counts = [1, 10, 100, 1000];

    for &count in &child_counts {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &_count| {
            rt.block_on(async {
                let engine = LayersEngine::new(2048.0, 2048.0);
                let root = engine.new_layer();
                engine.scene_set_root(root);

                engine.update(black_box(0.016));

                b.iter(|| {
                    let layer = engine.new_layer();
                    engine.add_layer(layer.clone());
                    engine.update(black_box(0.016));
                });
            });
        });
    }
}

#[allow(dead_code)]
fn criterion_benchmark_remove(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("engine::remove");
    group.measurement_time(Duration::from_secs(10));
    // Define different numbers of children to test
    let child_counts = [1, 10, 100, 1000];

    for &count in &child_counts {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            rt.block_on(async {
                let engine = LayersEngine::new(2048.0, 2048.0);
                let root = engine.new_layer();
                engine.scene_set_root(root);
                let mut layers = Vec::<Layer>::new();
                for _ in 0..count {
                    let layer = engine.new_layer();
                    engine.add_layer(layer.clone());
                    layers.push(layer);
                }
                engine.update(black_box(0.016));
                b.iter(|| {
                    if let Some(layer) = layers.pop() {
                        layer.remove();
                        engine.update(black_box(0.016));
                    }
                });
            });
        });
    }
}

#[allow(dead_code)]
fn criterion_benchmark_pointer_move(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("engine::pointer_move");
    group.measurement_time(Duration::from_secs(10));
    // Define different numbers of children to test
    let child_counts = [1, 10, 100, 1000];

    for &count in &child_counts {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            rt.block_on(async {
                let engine = LayersEngine::new(2048.0, 2048.0);
                let root = engine.new_layer();
                engine.scene_set_root(root);
                let mut layers = Vec::<Layer>::new();
                for _ in 0..count {
                    let layer = engine.new_layer();
                    layer.set_layout_style(Style {
                        position: taffy::Position::Absolute,
                        ..Default::default()
                    });
                    layer.set_size(Size::points(100.0, 100.0), None);
                    layer.set_position(
                        (
                            rand::random::<f32>() * 2048.0,
                            rand::random::<f32>() * 2048.0,
                        ),
                        None,
                    );
                    engine.add_layer(layer.clone());
                    layers.push(layer);
                }
                engine.update(black_box(0.016));

                b.iter(|| {
                    let x = rand::random::<f32>() * 2048.0;
                    let y = rand::random::<f32>() * 2048.0;
                    let p: Point = (x, y).into();
                    engine.pointer_move(p, None);
                });
            });
        });
    }
}

criterion_group!(
    benches,
    criterion_benchmark_update,
    criterion_benchmark_append,
    criterion_benchmark_remove
);
criterion_main!(benches);
