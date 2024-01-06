pub(crate) mod drawable;
pub(crate) mod model;
pub(crate) mod render_layer;
pub(crate) use self::model::ModelLayer;

use skia_safe::gpu::BackendTexture;

use skia_safe::gpu;
use skia_safe::gpu::gl::TextureInfo;
use std::sync::RwLock;
use std::{fmt, sync::Arc};
use taffy::prelude::Node;
use taffy::style::Style;

use crate::engine::animation::*;
use crate::engine::command::*;
use crate::engine::node::RenderableFlags;
use crate::engine::{Engine, NodeRef, TransactionRef};

use crate::types::*;

#[derive(Clone)]
pub struct Layer {
    pub(crate) engine: Arc<Engine>,
    pub id: Arc<RwLock<Option<NodeRef>>>,
    pub(crate) model: Arc<ModelLayer>,
    pub layout_node_id: Node,
}

impl Layer {
    pub fn with_engine(engine: Arc<Engine>) -> Self {
        let id = Arc::new(RwLock::new(None));
        let model = Arc::new(ModelLayer::default());

        let mut lt = engine.layout_tree.write().unwrap();

        let layout = lt
            .new_leaf(Style {
                ..Default::default()
            })
            .unwrap();

        Self {
            engine: engine.clone(),
            id,
            model,
            layout_node_id: layout,
        }
    }
    pub fn set_id(&self, id: NodeRef) {
        self.id.write().unwrap().replace(id);
    }
    pub fn id(&self) -> Option<NodeRef> {
        let id = *self.id.read().unwrap();
        id
    }
    change_model!(position, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(background_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_model!(scale, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(rotation, Point3d, RenderableFlags::NEEDS_LAYOUT);
    change_model!(anchor_point, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(
        border_corner_radius,
        BorderRadius,
        RenderableFlags::NEEDS_PAINT
    );

    change_model!(border_color, PaintColor, RenderableFlags::NEEDS_PAINT);
    change_model!(border_width, f32, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_offset, Point, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_radius, f32, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_spread, f32, RenderableFlags::NEEDS_PAINT);
    change_model!(shadow_color, Color, RenderableFlags::NEEDS_PAINT);
    change_model!(content, Option<Picture>, RenderableFlags::NEEDS_PAINT);
    change_model!(opacity, f32, RenderableFlags::NEEDS_PAINT);

    pub fn set_size(
        &self,
        value: impl Into<Size>,
        transition: Option<Transition>,
    ) -> TransactionRef {
        let value: Size = value.into();
        let flags = RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT;

        let change: Arc<ModelChange<Size>> = Arc::new(ModelChange {
            value_change: self.model.size.to(value, transition),
            flag: flags,
        });
        let id: Option<NodeRef> = *self.id.read().unwrap();
        let mut tr = TransactionRef(0);
        if let Some(id) = id {
            let animation = transition.map(|t| {
                self.engine.add_animation(
                    Animation {
                        duration: t.duration,
                        timing: t.timing,
                        start: t.delay + self.engine.now(),
                    },
                    true,
                )
            });

            tr = self.engine.schedule_change(id, change, animation);
        } else {
            self.model.size.set(value);
            // self.engine.set_node_layout_size(self.layout, value);
        }
        tr
    }

    pub fn set_layout_style(&self, style: Style) {
        self.engine
            .set_node_layout_style(self.layout_node_id, style);
    }

    pub fn set_node_layout_size(&self, size: Size) {
        self.engine.set_node_layout_size(self.layout_node_id, size);
    }

    pub fn node_layout_style(&self) -> Style {
        self.engine.get_node_layout_style(self.layout_node_id)
    }

    // // set content from gl texture
    pub fn set_content_from_texture(
        &self,
        context: &mut gpu::RecordingContext,
        texture_id: u32,
        target: skia_safe::gpu::gl::Enum,
        // _format: skia_safe::gpu::gl::Enum,
        size: impl Into<Point>,
    ) {
        let size = size.into();
        unsafe {
            let texture_info = TextureInfo {
                target,
                id: texture_id,
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
            };

            let texture = BackendTexture::new_gl(
                (size.x as i32, size.y as i32),
                skia_safe::gpu::MipMapped::No,
                texture_info,
            );

            let image = Image::from_texture(
                context,
                &texture,
                skia_safe::gpu::SurfaceOrigin::TopLeft,
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Premul,
                None,
            )
            .unwrap();

            let mut recorder = skia_safe::PictureRecorder::new();
            let canvas = recorder.begin_recording(
                skia_safe::Rect::from_wh(image.width() as f32, image.height() as f32),
                None,
            );
            canvas.draw_image(&image, (0.0, 0.0), None);
            let picture = recorder.finish_recording_as_picture(None);

            self.model.content.set(picture);
            if let Some(id) = self.id() {
                let change = Arc::new(NoopChange::new(id.0.into()));
                self.engine.schedule_change(id, change, None);
            }
        }
    }

    pub fn add_sublayer(&self, layer: Layer) -> NodeRef {
        self.engine.scene_add_layer(layer, self.id())
    }

    pub fn set_blend_mode(&self, blend_mode: BlendMode) {
        self.model.blend_mode.set(blend_mode);
    }
}

impl fmt::Debug for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layer")
            .field("id", &self.id())
            // .field("model", &self.model)
            .finish()
    }
}
