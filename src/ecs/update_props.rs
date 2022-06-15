use legion::*;
use crate::ecs::*;

//#[resource] timestamp: &Timestamp

// #[system(for_each)]
// #[filter(maybe_changed::<Property>())]
// fn update_animations(
//     prop: &mut Property,
//     #[resource] timestamp: &Timestamp
// ) {
   
// }

#[system(for_each)]
// #[filter(maybe_changed::<AnimatedValue<f64>>())]
pub fn update_props(
    animation: &mut AnimatedValue<f64>,
    #[resource] timestamp: &Timestamp
) {
   animation.update_at(timestamp.0);
}


// let system = SystemBuilder::new("update animations")
//     .read_resource::<Timestamp>()
//     .with_query(<Write<Property>>::query());
//     .build(|_cmd, world, timestamp, query| {
//         for mut prop in query.iter_mut(world) {
//             match prop.target {
//                 Some((target, animation)) => {
//                     let t = timestamp.0;
//                     match animation {
//                         Some(animation) => {
//                             let value = animation.value(t);
//                             println!("animation value: {}", value);
//                             prop.current_value = value * target;
//                         }
//                         None => {
//                             prop.current_value = target;
//                         },
//                     };
//                 },
//                 None => continue,
//             };
//         }
//     });