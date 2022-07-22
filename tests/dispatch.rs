use enum_dispatch::enum_dispatch;
use hello::{
    easing::Interpolable,
    ecs::animations::{AnimatedValue, ValueChange},
    layer::{BorderRadius, PaintColor, Point},
};

pub struct MC<V>(pub usize, V, pub bool);

impl<V> HVC for MC<V> {}

#[enum_dispatch]
trait HVC {}

#[enum_dispatch(HVC)]
enum MCC {
    A(MC<i32>),
    B(MC<f64>),
    // BorderCornerRadius(ModelChange<BorderRadius>),
    // PaintColor(ModelChange<PaintColor>),
}
#[test]
fn test_works() {
    let mc = MC(0, 1, false);
    let mcc: MCC = mc.into();

    let mc2 = MC(0, 1.0, false);
    let mcc2: MCC = mc2.into();

    let mc: MC<f64> = mcc2.try_into().unwrap();
    // let v: Vec<MCC<V>> = vec![mcc, mcc2];
}
