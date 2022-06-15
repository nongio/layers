pub mod animations;
pub mod update_props;

use legion::*;

use self::{animations::*};
use crate::layer::*;

pub struct ModelLayer {
    pub border_width: AnimatedValue<f64>,
    pub position: AnimatedValue<Point>,
}

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
pub fn setup_ecs() {
    let mut state = State::new();

    let layer = RenderLayer {
        background_color: PaintColor::Solid { color: Color::new(0.6, 0.0, 0.0, 1.0)},
        border_color: PaintColor::Solid { color: Color::new(0.0, 0.0, 0.0, 1.0)},
        border_corner_radius: BorderRadius{
            top_left: 20.0,
            top_right: 20.0,
            bottom_left: 10.0,
            bottom_right: 10.0,
        },
        border_style: BorderStyle::Solid,
        border_width: 4.0,
        position: Point{x:100.0, y:100.0},
        size: Point{x:200.0, y:200.0},
    };

    let mut model = ModelLayer {
        border_width: AnimatedValue::new(1.0),
        position: AnimatedValue::new(Point{x:100.0, y:100.0}),
    };
    
    // let mut x = model.border_width.clone();
    
    // let entity = state.ecs.push((model.border_width.clone(),));
    let transition = Transition {
        duration: 1.0,
        delay: 0.0,
        timing: Easing{x1:0.0, y1:0.0, x2:1.0, y2:1.0},
    };
    for frame in 1..=10 {
        state.ecs.push((AnimatedValue::new(frame as f64).to_animated(100.0, Some(transition)),));
    }

    for frame in 1..=10 {
        state.resources.get::<Timestamp>().map(|d| {
            let t = d.0;
            println!("frame {}:{}", frame, t);
        });
        

        // Update.
        state.update(0.01);

        // Render.
        let mut query = <&AnimatedValue<f64>>::query();
        for prop in query.iter(&state.ecs) {
            println!(
                "v: {:?}",
                prop.value()
            );
        }

        println!();
    }
}