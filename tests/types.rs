#[test]
pub fn colors() {
    let c = lay_rs::types::Color::new_hex("#ff0000");
    let cc: skia_safe::Color4f = c.into();
    assert_eq!(cc.r, 1.0);
    assert_eq!(cc.g, 0.0);
    assert_eq!(cc.b, 0.0);

    let c = lay_rs::types::Color::new_hex("#00ff00");
    let cc: skia_safe::Color4f = c.into();
    assert_eq!(cc.r, 0.0);
    assert_eq!(cc.g, 1.0);
    assert_eq!(cc.b, 0.0);

    let c = lay_rs::types::Color::new_hex("#0000ff");
    let cc: skia_safe::Color4f = c.into();
    assert_eq!(cc.r, 0.0);
    assert_eq!(cc.g, 0.0);
    assert_eq!(cc.b, 1.0);

    let c = lay_rs::types::Color::new_hex("#4043D1");
    let cc: skia_safe::Color4f = c.into();

    assert_eq!(cc.r, 64.0 / 255.0);
    assert_eq!(cc.g, 67.0 / 255.0);
    assert_eq!(cc.b, 209.0 / 255.0);
}
