use crate::{
    engine::command::Attribute,
    types::{BlendMode, Color, Point, *},
};

pub(crate) struct ModelLayer {
    pub anchor_point: Attribute<Point>,
    pub position: Attribute<Point>,
    pub scale: Attribute<Point>,
    pub rotation: Attribute<Point3d>,
    pub size: Attribute<Point>,
    pub background_color: Attribute<PaintColor>,
    pub border_corner_radius: Attribute<BorderRadius>,
    pub border_color: Attribute<PaintColor>,
    pub border_width: Attribute<f32>,
    pub shadow_offset: Attribute<Point>,
    pub shadow_radius: Attribute<f32>,
    pub shadow_spread: Attribute<f32>,
    pub shadow_color: Attribute<Color>,
    pub content: Attribute<Option<Picture>>,
    pub blend_mode: Attribute<BlendMode>,
    pub opacity: Attribute<f32>,
}

impl Default for ModelLayer {
    fn default() -> Self {
        let position = Attribute::new(Point { x: 0.0, y: 0.0 });
        let size = Attribute::new(Point { x: 100.0, y: 100.0 });
        let anchor_point = Attribute::new(Point { x: 0.0, y: 0.0 });
        let scale = Attribute::new(Point { x: 1.0, y: 1.0 });
        let rotation = Attribute::new(Point3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let background_color = Attribute::new(PaintColor::Solid {
            color: Color::new_rgba(1.0, 1.0, 1.0, 0.0),
        });
        let border_corner_radius = Attribute::new(BorderRadius::new_single(0.0));
        let border_color = Attribute::new(PaintColor::Solid {
            color: Color::new_rgba(0.0, 0.0, 0.0, 1.0),
        });
        let border_width = Attribute::new(0.0);
        let shadow_offset = Attribute::new(Point { x: 0.0, y: 0.0 });
        let shadow_radius = Attribute::new(0.0);
        let shadow_spread = Attribute::new(0.0);
        let shadow_color = Attribute::new(Color::new_rgba(0.0, 0.0, 0.0, 1.0));
        let content = Attribute::new(None);
        let blend_mode = Attribute::new(BlendMode::Normal);
        let opacity = Attribute::new(1.0);
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
