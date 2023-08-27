use crate::{
    engine::animations::SyncValue,
    types::{BlendMode, Color, Point, *},
};

pub(crate) struct ModelLayer {
    pub anchor_point: SyncValue<Point>,
    pub position: SyncValue<Point>,
    pub scale: SyncValue<Point>,
    pub rotation: SyncValue<Point3d>,
    pub size: SyncValue<Point>,
    pub background_color: SyncValue<PaintColor>,
    pub border_corner_radius: SyncValue<BorderRadius>,
    pub border_color: SyncValue<PaintColor>,
    pub border_width: SyncValue<f32>,
    pub shadow_offset: SyncValue<Point>,
    pub shadow_radius: SyncValue<f32>,
    pub shadow_spread: SyncValue<f32>,
    pub shadow_color: SyncValue<Color>,
    pub content: SyncValue<Option<Image>>,
    pub blend_mode: BlendMode,
    pub opacity: SyncValue<f32>,
}

impl Default for ModelLayer {
    fn default() -> Self {
        let position = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let size = SyncValue::new(Point { x: 100.0, y: 100.0 });
        let anchor_point = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let scale = SyncValue::new(Point { x: 1.0, y: 1.0 });
        let rotation = SyncValue::new(Point3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let background_color = SyncValue::new(PaintColor::Solid {
            color: Color::new_rgba(1.0, 1.0, 1.0, 0.0),
        });
        let border_corner_radius = SyncValue::new(BorderRadius::new_single(0.0));
        let border_color = SyncValue::new(PaintColor::Solid {
            color: Color::new_rgba(0.0, 0.0, 0.0, 1.0),
        });
        let border_width = SyncValue::new(0.0);
        let shadow_offset = SyncValue::new(Point { x: 0.0, y: 0.0 });
        let shadow_radius = SyncValue::new(0.0);
        let shadow_spread = SyncValue::new(0.0);
        let shadow_color = SyncValue::new(Color::new_rgba(0.0, 0.0, 0.0, 1.0));
        let content = SyncValue::new(None);
        let blend_mode = BlendMode::Normal;
        let opacity = SyncValue::new(1.0);
        Self {
            anchor_point,
            position,
            scale,
            rotation,
            size,
            background_color,
            border_corner_radius,
            border_color,
            border_width,
            shadow_offset,
            shadow_radius,
            shadow_spread,
            shadow_color,
            content,
            blend_mode,
            opacity,
        }
    }
}
