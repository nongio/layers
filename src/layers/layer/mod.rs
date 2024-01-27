pub(crate) mod drawable;
pub(crate) mod model;
pub(crate) mod render_layer;
use self::model::ContentDrawFunction;
pub(crate) use self::model::ModelLayer;

use std::sync::{atomic::AtomicBool, RwLock};
use std::{fmt, sync::Arc};
use taffy::style::Style;
use taffy::{prelude::Node, style::Display};

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
    pub hidden: Arc<AtomicBool>,
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
            hidden: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn set_id(&self, id: NodeRef) {
        self.id.write().unwrap().replace(id);
    }
    pub fn id(&self) -> Option<NodeRef> {
        let id = *self.id.read().unwrap();
        id
    }
    pub fn set_hidden(&self, hidden: bool) {
        self.hidden
            .store(hidden, std::sync::atomic::Ordering::Relaxed);

        // when hidden we set display to none so that the layout engine
        // doesn't layout the node
        let mut display = Display::None;

        if !hidden {
            display = self.model.display.value();
        }
        let mut style = self.engine.get_node_layout_style(self.layout_node_id);
        style.display = display;
        self.engine
            .set_node_layout_style(self.layout_node_id, style);

        if let Some(id) = self.id() {
            // let node = self.engine.scene.get_node(id.0);
            let arena = self.engine.scene.nodes.data();
            let arena = arena.read().unwrap();
            let mut iter = id.ancestors(&arena);
            if let Some(node) = self.engine.scene.get_node(id) {
                let node = node.get();
                node.insert_flags(RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT);
            }

            iter.next(); // skip self
            if let Some(parent_id) = iter.next() {
                drop(arena);
                if let Some(parent) = self.engine.scene.get_node(NodeRef(parent_id)) {
                    let parent = parent.get();
                    parent
                        .insert_flags(RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT);
                }
            }
        }
    }
    pub fn hidden(&self) -> bool {
        self.hidden.load(std::sync::atomic::Ordering::Relaxed)
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
        }
        tr
    }
    pub fn size(&self) -> Size {
        self.model.size.value()
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

    pub fn set_draw_content<F: Into<ContentDrawFunction>>(&self, content_handler: Option<F>) {
        let mut model_content = self.model.draw_content.write().unwrap();
        if let Some(content_handler) = content_handler {
            *model_content = Some(content_handler.into());
        } else {
            *model_content = None;
        }
        if let Some(id) = self.id() {
            let mut node = self.engine.scene.get_node(id).unwrap();
            let node = node.get_mut();
            node.insert_flags(RenderableFlags::NEEDS_PAINT);
        }
    }
    pub fn add_sublayer(&self, layer: Layer) -> NodeRef {
        self.engine.scene_add_layer(layer, self.id())
    }

    pub fn set_blend_mode(&self, blend_mode: BlendMode) {
        self.model.blend_mode.set(blend_mode);
    }
    pub fn set_display(&self, display: Display) {
        self.model.display.set(display);
    }

    pub fn on_finish<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        handler: F,
    ) {
        self.engine.on_finish(transaction, handler);
    }
    pub fn on_update<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        transaction: TransactionRef,
        handler: F,
    ) {
        self.engine.on_update(transaction, handler);
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
