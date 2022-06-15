use criterion::{black_box, criterion_group, criterion_main, Criterion};

use legion::*;
use hello::ecs::animations::{AnimatedValue, Transition, Easing};

pub struct Timestamp(f64);

struct State {
    ecs: World,
    resources: Resources,
    systems: Schedule,
}

#[system(for_each)]
pub fn update_props(
    animation: &mut AnimatedValue<f64>,
    #[resource] timestamp: &Timestamp
) {
   animation.update_at(timestamp.0);
}

impl State {
    fn new() -> Self {
        let ecs = World::default();
        let mut resources = Resources::default();

        resources.insert(Timestamp(0.));
        
        let systems = Schedule::builder()
        .add_system(update_props_system())
        .build();

        State {
            ecs,
            systems,
            resources,
        }
    }
    fn update(&mut self, dt: f64) {
        self.resources.get_mut::<Timestamp>().map(|mut d| d.0 += dt);
        self.systems.execute(&mut self.ecs, &mut self.resources);
    }
}


fn criterion_benchmark(c: &mut Criterion) {
    let mut state = State::new();
    let transition = Transition {
        duration: 1.0,
        delay: 0.0,
        timing: Easing{x1:0.0, y1:0.0, x2:1.0, y2:1.0},
    };
    for frame in 1..=5000 {
        state.ecs.push((AnimatedValue::new(frame as f64).to_animated(100.0, Some(transition)),));
    }

    c.bench_function("update", |b| b.iter(|| state.update(black_box(0.01))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);