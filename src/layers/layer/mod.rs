pub(crate) mod drawable;
pub(crate) mod model;
pub(crate) mod render_layer;
pub(crate) mod state;

pub(crate) use self::model::ModelLayer;
use self::model::{ContentDrawFunction, PointerHandlerFunction};

use skia::{ColorFilter, Contains, ImageFilter};
use state::LayerDataProps;
use std::{fmt, sync::Arc};
use std::{
    hash::{Hash, Hasher},
    sync::{atomic::AtomicBool, RwLock},
};
use taffy::style::Style;
use taffy::{prelude::NodeId, style::Display};

use crate::engine::{animation::*, storage::TreeStorageId, AnimatedNodeChange};
use crate::engine::{command::*, PointerEventType};
use crate::engine::{node::RenderableFlags, TransactionCallback};
use crate::engine::{Engine, NodeRef, TransactionRef};

use crate::types::*;
#[allow(private_interfaces)]
#[derive(Clone)]
pub struct Layer {
    pub engine: Arc<Engine>,
    pub(crate) id: Arc<RwLock<Option<NodeRef>>>,
    pub(crate) key: Arc<RwLock<String>>,
    pub(crate) hidden: Arc<AtomicBool>,
    pub(crate) pointer_events: Arc<AtomicBool>,
    pub(crate) layout_node_id: NodeId,
    pub(crate) model: Arc<ModelLayer>,
    pub(crate) image_cache: Arc<AtomicBool>,
    pub(crate) state: Arc<RwLock<LayerDataProps>>,
    pub(crate) effect: Arc<RwLock<Option<Arc<dyn Effect>>>>,
}

pub trait Effect: Send + Sync {
    fn init(&self, layer: &Layer);
    fn start(&self, layer: &Layer);
    fn update(&self, layer: &Layer, time: f32);
    fn finish(&self, layer: &Layer);
}
impl Layer {
    pub(crate) fn with_engine(engine: Arc<Engine>) -> Self {
        let id = Arc::new(RwLock::new(None));
        let key = Arc::new(RwLock::new(String::new()));
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
            key,
            model,
            layout_node_id: layout,
            hidden: Arc::new(AtomicBool::new(false)),
            pointer_events: Arc::new(AtomicBool::new(true)),
            image_cache: Arc::new(AtomicBool::new(false)),
            state: Arc::new(RwLock::new(LayerDataProps::new())),
            effect: Arc::new(RwLock::new(None)),
        }
    }
    pub fn set_id(&self, id: NodeRef) {
        self.id.write().unwrap().replace(id);
    }
    pub fn id(&self) -> Option<NodeRef> {
        let id = *self.id.read().unwrap();
        id
    }
    pub fn set_key(&self, key: impl Into<String>) {
        let key = key.into();
        *self.key.write().unwrap() = key.clone();
        *self.model.key.write().unwrap() = key;
    }
    pub fn key(&self) -> String {
        let key = self.key.read().unwrap();
        key.clone()
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
    pub fn set_pointer_events(&self, pointer_events: bool) {
        self.pointer_events
            .store(pointer_events, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn pointer_events(&self) -> bool {
        self.pointer_events
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    change_model!(position, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(scale, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(rotation, Point3d, RenderableFlags::NEEDS_LAYOUT);
    change_model!(anchor_point, Point, RenderableFlags::NEEDS_LAYOUT);
    change_model!(opacity, f32, RenderableFlags::NEEDS_LAYOUT);

    change_model!(background_color, PaintColor, RenderableFlags::NEEDS_PAINT);
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
    change_model!(image_filter_progress, f32, RenderableFlags::NEEDS_PAINT);

    pub fn change_size(&self, value: Size) -> AnimatedNodeChange {
        let flags = RenderableFlags::NEEDS_LAYOUT;
        let change: Arc<ModelChange<Size>> = Arc::new(ModelChange {
            value_change: self.model.size.to(value, None),
            flag: flags,
        });
        let node_id = self.id().unwrap();
        AnimatedNodeChange {
            animation_id: None,
            change,
            node_id,
        }
    }
    pub fn set_size(
        &self,
        value: impl Into<Size>,
        transition: impl Into<Option<Transition>>,
    ) -> TransactionRef {
        let transition = transition.into();
        let value: Size = value.into();
        let flags = RenderableFlags::NEEDS_LAYOUT;

        let change: Arc<ModelChange<Size>> = Arc::new(ModelChange {
            value_change: self.model.size.to(value, transition),
            flag: flags,
        });
        let id: Option<NodeRef> = *self.id.read().unwrap();
        let mut tr = TransactionRef {
            id: 0,
            engine_id: self.engine.id,
        };
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

    pub fn set_draw_content<F: Into<ContentDrawFunction>>(&self, content_handler: F) {
        let mut model_content = self.model.draw_content.write().unwrap();
        *model_content = Some(content_handler.into());
        if let Some(id) = self.id() {
            let mut node = self.engine.scene.get_node(id).unwrap();
            let node = node.get_mut();
            node.insert_flags(RenderableFlags::NEEDS_PAINT);
        }
    }
    pub fn remove_draw_content(&self) {
        let mut model_content = self.model.draw_content.write().unwrap();
        *model_content = None;
        if let Some(id) = self.id() {
            let mut node = self.engine.scene.get_node(id).unwrap();
            let node = node.get_mut();
            node.insert_flags(RenderableFlags::NEEDS_PAINT);
        }
    }
    pub fn set_image_cache(&self, value: bool) {
        self.image_cache
            .store(value, std::sync::atomic::Ordering::Relaxed);
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

    pub fn add_on_pointer_move<F: Into<PointerHandlerFunction>>(
        &self,
        handler: F,
    ) -> Option<usize> {
        let handler = handler.into();
        let id = self.id();
        if let Some(id) = id {
            let handler_id = self
                .engine
                .add_pointer_handler(id, PointerEventType::Move, handler);
            return Some(handler_id);
        }
        None
    }
    pub fn remove_on_pointer_move(&self, handler_id: Option<usize>) {
        if let Some(id) = self.id() {
            let handler_id = handler_id.unwrap();
            self.engine.remove_pointer_handler(id, handler_id);
        }
    }
    pub fn add_on_pointer_in<F: Into<PointerHandlerFunction>>(&self, handler: F) -> Option<usize> {
        let handler = handler.into();
        let id = self.id();
        if let Some(id) = id {
            let handler_id = self
                .engine
                .add_pointer_handler(id, PointerEventType::In, handler);
            return Some(handler_id);
        }
        None
    }
    pub fn remove_on_pointer_in(&self, handler_id: Option<usize>) {
        if let Some(id) = self.id() {
            let handler_id = handler_id.unwrap();
            self.engine.remove_pointer_handler(id, handler_id);
        }
    }
    pub fn add_on_pointer_out<F: Into<PointerHandlerFunction>>(&self, handler: F) -> Option<usize> {
        let handler = handler.into();
        let id = self.id();
        if let Some(id) = id {
            let handler_id = self
                .engine
                .add_pointer_handler(id, PointerEventType::Out, handler);
            return Some(handler_id);
        }
        None
    }
    pub fn remove_on_pointer_out(&self, handler_id: Option<usize>) {
        if let Some(id) = self.id() {
            let handler_id = handler_id.unwrap();
            self.engine.remove_pointer_handler(id, handler_id);
        }
    }
    pub fn add_on_pointer_press<F: Into<PointerHandlerFunction>>(
        &self,
        handler: F,
    ) -> Option<usize> {
        let handler = handler.into();
        let id = self.id();
        if let Some(id) = id {
            let handler_id = self
                .engine
                .add_pointer_handler(id, PointerEventType::Down, handler);
            return Some(handler_id);
        }
        None
    }
    pub fn remove_on_pointer_press(&self, handler_id: Option<usize>) {
        if let Some(id) = self.id() {
            let handler_id = handler_id.unwrap();
            self.engine.remove_pointer_handler(id, handler_id);
        }
    }
    pub fn add_on_pointer_release<F: Into<PointerHandlerFunction>>(
        &self,
        handler: F,
    ) -> Option<usize> {
        let handler = handler.into();
        let id = self.id();
        if let Some(id) = id {
            let handler_id = self
                .engine
                .add_pointer_handler(id, PointerEventType::Up, handler);
            return Some(handler_id);
        }
        None
    }
    pub fn remove_on_pointer_release(&self, handler_id: Option<usize>) {
        if let Some(id) = self.id() {
            let handler_id = handler_id.unwrap();
            self.engine.remove_pointer_handler(id, handler_id);
        }
    }
    pub fn remove_all_pointer_handlers(&self) {
        if let Some(id) = self.id() {
            self.engine.remove_all_pointer_handlers(id);
        }
    }

    pub fn render_position(&self) -> Point {
        let id = self.id();
        if let Some(id) = id {
            if let Some(node) = self.engine.scene.get_node(id) {
                let render_layer = node.get().render_layer.clone();
                let rl = render_layer.read().unwrap();

                return Point {
                    x: rl.global_transformed_bounds.x(),
                    y: rl.global_transformed_bounds.y(),
                };
            }
        }
        Point { x: 0.0, y: 0.0 }
    }
    pub fn render_size(&self) -> Point {
        let id = self.id();
        if let Some(id) = id {
            if let Some(node) = self.engine.scene.get_node(id) {
                let render_layer = node.get().render_layer.clone();
                let rl = render_layer.read().unwrap();

                return Point {
                    x: rl.global_transformed_bounds.width(),
                    y: rl.global_transformed_bounds.height(),
                };
            }
        }
        Point { x: 0.0, y: 0.0 }
    }
    pub fn render_bounds_transformed(&self) -> skia_safe::Rect {
        let id = self.id();
        if let Some(id) = id {
            if let Some(node) = self.engine.scene.get_node(id) {
                let render_layer = node.get().render_layer.clone();
                let rl = render_layer.read().unwrap();
                return rl.global_transformed_bounds;
            }
        }
        skia_safe::Rect::default()
    }
    pub fn render_bounds_with_children_transformed(&self) -> skia_safe::Rect {
        let id = self.id();
        if let Some(id) = id {
            if let Some(node) = self.engine.scene.get_node(id) {
                let render_layer = node.get().render_layer.clone();
                let rl = render_layer.read().unwrap();

                return rl.global_transformed_bounds_with_children;
            }
        }
        skia_safe::Rect::default()
    }
    pub fn render_bounds_with_children(&self) -> skia_safe::Rect {
        let id = self.id();
        if let Some(id) = id {
            if let Some(node) = self.engine.scene.get_node(id) {
                let render_layer = node.get().render_layer.clone();
                let rl = render_layer.read().unwrap();

                return rl.bounds_with_children;
            }
        }
        skia_safe::Rect::default()
    }
    pub fn cointains_point(&self, point: impl Into<skia_safe::Point>) -> bool {
        let point = point.into();
        self.render_bounds_transformed().contains(point)
    }
    pub fn children_nodes(&self) -> Vec<NodeRef> {
        if let Some(node_ref) = self.id() {
            let node_id: TreeStorageId = node_ref.into();
            return {
                let arena = self.engine.scene.nodes.data();
                let arena = arena.read().unwrap();
                node_id.children(&arena).map(NodeRef).collect()
            };
        }
        vec![]
    }
    pub fn children(&self) -> Vec<Layer> {
        if let Some(node_ref) = self.id() {
            let node_id: TreeStorageId = node_ref.into();
            return {
                let arena = self.engine.scene.nodes.data();
                let arena = arena.read().unwrap();
                node_id
                    .children(&arena)
                    .map(|cid| {
                        let c = arena.get(cid).unwrap();
                        c.get().layer.clone()
                    })
                    .collect()
            };
        }
        vec![]
    }
    pub fn with_state<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&LayerDataProps) -> T,
    {
        let data = self.state.read().unwrap();
        f(&data)
    }
    pub fn with_mut_state<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut LayerDataProps) -> T,
    {
        let mut data = self.state.write().unwrap();
        f(&mut data)
    }

    pub fn set_image_filter(&self, filter: impl Into<Option<ImageFilter>>) {
        let filter = filter.into();
        let mut model_filter = self.model.image_filter.write().unwrap();
        *model_filter = filter;
    }
    pub fn set_color_filter(&self, filter: impl Into<Option<ColorFilter>>) {
        let filter = filter.into();

        let id = self.id().unwrap();
        let change = Arc::new(NoopChange::new(id.0.into()));
        self.engine.schedule_change(id, change, None);
        let mut model_filter = self.model.color_filter.write().unwrap();
        *model_filter = filter;
    }
    pub fn set_filter_bounds(&self, bounds: impl Into<Option<skia::Rect>>) {
        let mut model_filter_bounds = self.model.filter_bounds.write().unwrap();
        let bounds = bounds.into();
        *model_filter_bounds = bounds;
    }
    pub fn remove(&self) {
        if let Some(id) = self.id() {
            self.engine.mark_for_delete(id);
        }
    }

    pub fn set_effect(&self, effect: impl Effect + 'static) {
        let effect = Arc::new(effect);
        effect.init(self);
        let filter_model_id = self.model.image_filter_progress.id;
        let tr = TransactionRef {
            id: filter_model_id,
            engine_id: self.engine.id,
        };
        let effect_ref = effect.clone();
        self.engine.on_update(
            tr,
            move |l: &Layer, _p| {
                effect_ref.update(l, l.model.image_filter_progress.value());
            },
            false,
        );
        let effect_ref = effect.clone();
        self.engine.on_start(
            tr,
            move |l: &Layer, _| {
                effect_ref.start(l);
            },
            false,
        );
        *self.effect.write().unwrap() = Some(effect.clone());
    }
    pub fn remove_effect(&self) {
        let mut effect = self.effect.write().unwrap();
        if let Some(effect) = &*effect {
            effect.finish(self);
        }

        *effect = None;
    }
    pub fn on_change_size<F: Into<TransactionCallback>>(&self, f: F, once: bool) {
        let size_id = self.model.size.id;
        self.engine.on_update(
            TransactionRef {
                id: size_id,
                engine_id: self.engine.id,
            },
            f,
            once,
        );
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

impl PartialEq for Layer {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for Layer {}
impl Hash for Layer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}
impl From<Layer> for Option<NodeRef> {
    fn from(val: Layer) -> Self {
        val.id()
    }
}
