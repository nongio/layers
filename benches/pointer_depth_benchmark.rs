use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use lay_rs::{engine::Engine, prelude::*, types::*};

/// Creates a tree of layers with specified depth
/// Each layer has a single child, creating a straight vertical line
fn create_layer_tree(engine: &Engine, depth: usize) -> (Layer, Vec<Layer>) {
    let root = engine.new_layer();
    root.set_size(Size::points(500.0, 500.0), None);
    root.set_position((0.0, 0.0), None);

    let mut current = root.clone();
    let mut layers = vec![root.clone()];

    for i in 0..depth {
        let child = engine.new_layer();
        child.set_size(Size::points(400.0, 400.0), None);
        child.set_position((50.0, 50.0), None); // Offset from parent

        // Add pointer handlers to the layer
        let layer_clone = child.clone();
        child.add_on_pointer_move(move |_: &Layer, _, _| {
            black_box(&layer_clone);
        });

        engine.append_layer(&child.id, current.id);
        current = child.clone();
        layers.push(child);
    }

    engine.update(0.016); // Initial update to apply layout

    (root, layers)
}

/// Creates a tree with multiple siblings at each level
fn create_wide_layer_tree(engine: &Engine, depth: usize, siblings: usize) -> (Layer, Vec<Layer>) {
    let root = engine.new_layer();
    root.set_size(Size::points(1000.0, 1000.0), None);
    root.set_position((0.0, 0.0), None);

    let mut layers = vec![root.clone()];
    let mut current_level = vec![root.clone()];
    let mut next_level = Vec::new();

    for _ in 0..depth {
        for parent in current_level.iter() {
            for i in 0..siblings {
                let child = engine.new_layer();
                child.set_size(Size::points(100.0, 100.0), None);

                // Position siblings next to each other
                let x = (i as f32 * 120.0) % 900.0;
                let y = ((i as f32 * 120.0) / 900.0).floor() * 120.0;
                child.set_position((x, y), None);

                // Add pointer handlers to the layer
                let layer_clone = child.clone();
                child.add_on_pointer_move(move |_: &Layer, _, _| {
                    black_box(&layer_clone);
                });

                engine.append_layer(&child.id, parent.id);
                next_level.push(child.clone());
                layers.push(child);
            }
        }

        std::mem::swap(&mut current_level, &mut next_level);
        next_level.clear();
    }

    engine.update(0.016); // Initial update to apply layout

    (root, layers)
}

fn criterion_benchmark_pointer_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("pointer_handler::depth");
    group.measurement_time(Duration::from_secs(5));

    // Test with different tree depths
    let depths = [1, 5, 10, 20, 50, 100];

    for &depth in &depths {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            let engine = Engine::create(2048.0, 2048.0);
            let (root, _layers) = create_layer_tree(&engine, depth);
            engine.scene_set_root(root);
            engine.update(black_box(0.016));

            // Test pointer events at different tree depths
            b.iter(|| {
                // Simulate pointer over the deepest layer
                let p: skia_safe::Point = (100.0, 100.0).into();
                engine.pointer_move(black_box(&p), None);
            });
        });
    }

    group.finish();
}

fn criterion_benchmark_pointer_siblings(c: &mut Criterion) {
    let mut group = c.benchmark_group("pointer_handler::siblings");
    group.measurement_time(Duration::from_secs(5));

    // Test with different numbers of siblings at each level
    let siblings_counts = [1, 3, 5, 10];

    for &siblings in &siblings_counts {
        group.bench_with_input(
            BenchmarkId::new("depth_5", siblings),
            &siblings,
            |b, &siblings| {
                let engine = Engine::create(2048.0, 2048.0);
                let (root, _layers) = create_wide_layer_tree(&engine, 5, siblings);
                engine.scene_set_root(root);
                engine.update(black_box(0.016));

                // Test pointer events with many siblings
                b.iter(|| {
                    // Simulate pointer movement over various parts of the tree
                    for x in [100.0, 300.0, 500.0, 700.0].iter() {
                        for y in [100.0, 300.0, 500.0].iter() {
                            let p: skia_safe::Point = (*x, *y).into();
                            engine.pointer_move(black_box(&p), None);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

fn criterion_benchmark_pointer_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("pointer_handler::events");
    group.measurement_time(Duration::from_secs(5));

    // Test with different tree depths for in/out events
    let depths = [5, 20, 50];

    for &depth in &depths {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            let engine = Engine::create(2048.0, 2048.0);
            let (root, layers) = create_layer_tree(&engine, depth);
            engine.scene_set_root(root);

            // Add in/out handlers to all layers
            for layer in &layers {
                let l = layer.clone();
                layer.add_on_pointer_in(move |_: &Layer, _, _| {
                    black_box(&l);
                });

                let l = layer.clone();
                layer.add_on_pointer_out(move |_: &Layer, _, _| {
                    black_box(&l);
                });
            }

            engine.update(black_box(0.016));

            // Benchmark moving pointer between inside and outside positions
            b.iter(|| {
                // Move pointer inside deepest layer
                let inside_p: skia_safe::Point = (100.0, 100.0).into();
                engine.pointer_move(black_box(&inside_p), None);

                // Move pointer outside all layers
                let outside_p: skia_safe::Point = (1000.0, 1000.0).into();
                engine.pointer_move(black_box(&outside_p), None);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    criterion_benchmark_pointer_depth,
    criterion_benchmark_pointer_siblings,
    criterion_benchmark_pointer_events
);
criterion_main!(benches);
