pub(crate) mod model;
pub(crate) mod render_layer;
pub(crate) mod state;
pub(crate) use self::model::ModelLayer;

use model::ContentDrawFunctionInternal;
use render_layer::RenderLayer;
use skia::{ColorFilter, Contains, ImageFilter};
use std::{fmt, sync::Arc};
use std::{
    hash::{Hash, Hasher},
    sync::RwLock,
};
use taffy::style::Display;
use taffy::style::Style;

use self::model::{ContentDrawFunction, PointerHandlerFunction};

use crate::engine::{animation::*, storage::TreeStorageId, AnimatedNodeChange};
use crate::engine::{command::*, PointerEventType};
use crate::engine::{node::RenderableFlags, TransactionCallback};
use crate::engine::{Engine, NodeRef, TransactionRef};
use crate::types::*;

#[cfg(feature = "layer_state")]
use state::LayerDataProps;

#[allow(private_interfaces)]
#[repr(C)]
#[derive(Clone)]
pub struct Layer {
    pub engine: Arc<Engine>,
    pub id: NodeRef,
    pub layout_id: taffy::tree::NodeId,
    pub(crate) model: Arc<ModelLayer>,
    pub(crate) effect: Arc<RwLock<Option<Arc<dyn Effect>>>>,

    #[cfg(feature = "layer_state")]
    pub(crate) state: Arc<RwLock<LayerDataProps>>,
}

pub trait Effect: Send + Sync {
    fn init(&self, layer: &Layer);
    fn start(&self, layer: &Layer);
    fn update(&self, layer: &Layer, time: f32);
    fn finish(&self, layer: &Layer);
}

impl Layer {
    pub(crate) fn with_engine(
        engine: Arc<Engine>,
        id: NodeRef,
        layout_id: taffy::tree::NodeId,
    ) -> Self {
        Self {
            id,
            layout_id,
            engine: engine.clone(),
            model: Arc::new(ModelLayer::default()),
            effect: Arc::new(RwLock::new(None)),
            #[cfg(feature = "layer_state")]
            state: Arc::new(RwLock::new(LayerDataProps::new())),
        }
    }
    pub fn id(&self) -> NodeRef {
        self.id
    }
    pub fn set_key(&self, key: impl Into<String>) {
        let key = key.into();
        *self.model.key.write().unwrap() = key;
    }
    pub fn key(&self) -> String {
        let key = self.model.key.read().unwrap();
        key.clone()
    }
    pub fn set_hidden(&self, hidden: bool) {
        // when hidden we set display to none so that the layout engine
        // doesn't layout the node
        let mut display = Display::None;

        if !hidden {
            display = self.model.display.value();
        }

        let mut style = self.engine.get_node_layout_style(self.layout_id);
        style.display = display;

        // self.engine.set_node_layout_style(self.layout_node_id, style);

        self.engine.scene.with_arena_mut(|arena| {
            let id = self.id.into();
            let node = arena.get_mut(id);
            if let Some(node) = node {
                let node = node.get_mut();
                node.set_hidden(hidden);
                node.insert_flags(RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT);
            }
            let mut iter = id.ancestors(arena);
            iter.next(); // skip self
            if let Some(parent_id) = iter.next() {
                if arena.get(parent_id).is_some() {
                    if let Some(parent_node) = arena.get_mut(parent_id) {
                        let parent = parent_node.get_mut();
                        parent.insert_flags(
                            RenderableFlags::NEEDS_LAYOUT | RenderableFlags::NEEDS_PAINT,
                        );
                    }
                }
            }
        });
    }
    pub fn hidden(&self) -> bool {
        self.engine.scene.with_arena(|a| {
            let node = a.get(self.id.into()).unwrap();
            let node = node.get();
            node.hidden
        })
    }
    pub fn set_pointer_events(&self, pointer_events: bool) {
        self.model
            .pointer_events
            .store(pointer_events, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn pointer_events(&self) -> bool {
        self.model
            .pointer_events
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
    change_model!(clip_content, bool, RenderableFlags::NEEDS_PAINT);
    change_model!(clip_children, bool, RenderableFlags::NEEDS_PAINT);

    pub fn change_size(&self, value: Size) -> AnimatedNodeChange {
        let flags = RenderableFlags::NEEDS_LAYOUT;
        let change: Arc<ModelChange<Size>> = Arc::new(ModelChange {
            value_change: self.model.size.to(value, None),
            flag: flags,
        });

        AnimatedNodeChange {
            node_id: self.id,
            animation_id: None,
            change,
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
        let value_id = self.model.size.id;

        let change: Arc<ModelChange<Size>> = Arc::new(ModelChange {
            value_change: self.model.size.to(value, transition),
            flag: flags,
        });

        let animation = transition.map(|t| {
            // if there is a transition
            let merged_timing = if let TimingFunction::Spring(mut spring) = t.timing {
                // and the transition is a spring, check if there is already a running transaction
                let velocity = self
                    .engine
                    .get_transaction_for_value(value_id)
                    .map(|running_transaction| {
                        if let Some(animation_id) = running_transaction.animation_id {
                            let animation_state = self.engine.get_animation(animation_id).unwrap();
                            let animation = animation_state.animation;
                            match animation.timing {
                                TimingFunction::Spring(s) => {
                                    let (_current_position, current_velocity) =
                                        s.update_pos_vel_at(animation_state.time);
                                    current_velocity
                                }
                                _ => 0.0,
                            }
                        } else {
                            0.0
                        }
                    })
                    .unwrap_or(0.0);
                spring.initial_velocity = velocity;
                TimingFunction::Spring(spring)
            } else {
                t.timing
            };
            self.engine.add_animation(
                Animation {
                    timing: merged_timing,
                    start: t.delay + self.engine.now(),
                },
                true,
            )
        });

        self.engine.schedule_change(self.id, change, animation)
    }
    pub fn size(&self) -> Size {
        self.model.size.value()
    }
    pub fn set_layout_style(&self, style: Style) {
        self.engine.set_node_layout_style(self.layout_id, style);
    }

    pub fn set_node_layout_size(&self, size: Size) {
        self.engine.set_node_layout_size(self.layout_id, size);
    }

    pub fn node_layout_style(&self) -> Style {
        self.engine.get_node_layout_style(self.layout_id)
    }

    pub fn set_draw_content<F: Into<ContentDrawFunction>>(&self, content_handler: F) {
        let mut model_content = self.model.draw_content.write().unwrap();
        let draw: ContentDrawFunction = content_handler.into();
        *model_content = Some(draw.into());

        self.engine
            .schedule_change(self.id, Arc::new(NoopChange::new(self.id.0.into())), None);
    }
    #[allow(unused)]
    pub(crate) fn set_draw_content_internal<F: Into<ContentDrawFunctionInternal>>(
        &self,
        content_handler: F,
    ) {
        let mut model_content = self.model.draw_content.write().unwrap();
        *model_content = Some(content_handler.into());

        self.engine
            .set_node_flags(self.id, RenderableFlags::NEEDS_PAINT);
    }
    pub fn remove_draw_content(&self) {
        let mut model_content = self.model.draw_content.write().unwrap();
        *model_content = None;
        self.engine
            .set_node_flags(self.id, RenderableFlags::NEEDS_PAINT);
    }
    pub fn add_sublayer<'a>(&self, layer: impl Into<&'a NodeRef>) {
        self.engine.append_layer(layer, self.id)
    }

    pub fn prepend_sublayer(&self, layer: Layer) {
        self.engine.prepend_layer(layer, self.id)
    }

    pub fn set_blend_mode(&self, blend_mode: BlendMode) {
        self.model.blend_mode.set(blend_mode);
        self.engine
            .schedule_change(self.id, Arc::new(NoopChange::new(self.id.0.into())), None);
    }
    pub fn set_display(&self, display: Display) {
        self.model.display.set(display);
    }

    pub fn add_on_pointer_move<F: Into<PointerHandlerFunction>>(&self, handler: F) -> usize {
        let handler = handler.into();

        self.engine
            .add_pointer_handler(self.id, PointerEventType::Move, handler)
    }
    pub fn remove_on_pointer_move(&self, handler_id: usize) {
        self.engine.remove_pointer_handler(self.id, handler_id);
    }
    pub fn add_on_pointer_in<F: Into<PointerHandlerFunction>>(&self, handler: F) -> usize {
        let handler = handler.into();

        self.engine
            .add_pointer_handler(self.id, PointerEventType::In, handler)
    }
    pub fn remove_on_pointer_in(&self, handler_id: Option<usize>) {
        let handler_id = handler_id.unwrap();
        self.engine.remove_pointer_handler(self.id, handler_id);
    }
    pub fn add_on_pointer_out<F: Into<PointerHandlerFunction>>(&self, handler: F) -> usize {
        let handler = handler.into();

        self.engine
            .add_pointer_handler(self.id, PointerEventType::Out, handler)
    }
    pub fn remove_on_pointer_out(&self, handler_id: Option<usize>) {
        let handler_id = handler_id.unwrap();
        self.engine.remove_pointer_handler(self.id, handler_id);
    }
    pub fn add_on_pointer_press<F: Into<PointerHandlerFunction>>(&self, handler: F) -> usize {
        let handler = handler.into();

        self.engine
            .add_pointer_handler(self.id, PointerEventType::Down, handler)
    }
    pub fn remove_on_pointer_press(&self, handler_id: Option<usize>) {
        let handler_id = handler_id.unwrap();
        self.engine.remove_pointer_handler(self.id, handler_id);
    }
    pub fn add_on_pointer_release<F: Into<PointerHandlerFunction>>(&self, handler: F) -> usize {
        let handler = handler.into();

        self.engine
            .add_pointer_handler(self.id, PointerEventType::Up, handler)
    }
    pub fn remove_on_pointer_release(&self, handler_id: Option<usize>) {
        let handler_id = handler_id.unwrap();
        self.engine.remove_pointer_handler(self.id, handler_id);
    }
    pub fn remove_all_pointer_handlers(&self) {
        self.engine.remove_all_pointer_handlers(self.id);
    }
    pub fn render_layer(&self) -> RenderLayer {
        self.engine.scene.with_arena(|arena| {
            let node = arena.get(self.id.0).unwrap();
            let node = node.get();
            node.render_layer.clone()
        })
    }
    pub fn render_position(&self) -> Point {
        self.engine.scene.with_arena(|arena| {
            let node = arena.get(self.id.into()).unwrap();
            let node = node.get();
            let render_layer = &node.render_layer;
            let rl = render_layer;
            Point {
                x: rl.global_transformed_bounds.x(),
                y: rl.global_transformed_bounds.y(),
            }
        })
    }
    pub fn render_size(&self) -> Point {
        self.engine.scene.with_arena(|arena| {
            let node = arena.get(self.id.into()).unwrap();
            let node = node.get();
            let render_layer = &node.render_layer;
            let rl = render_layer;
            Point {
                x: rl.global_transformed_bounds.width(),
                y: rl.global_transformed_bounds.height(),
            }
        })
    }
    pub fn render_bounds_transformed(&self) -> skia_safe::Rect {
        self.engine.scene.with_arena(|arena| {
            let node = arena.get(self.id.into()).unwrap();
            let node = node.get();
            let render_layer = &node.render_layer;
            let rl = render_layer;
            rl.global_transformed_bounds
        })
    }
    pub fn render_bounds_with_children_transformed(&self) -> skia_safe::Rect {
        self.engine.scene.with_arena(|arena| {
            let node = arena.get(self.id.into()).unwrap();
            let node = node.get();
            let render_layer = &node.render_layer;
            let rl = render_layer;
            rl.global_transformed_bounds_with_children
        })
    }
    pub fn render_bounds_with_children(&self) -> skia_safe::Rect {
        self.engine.scene.with_arena(|arena| {
            let node = arena.get(self.id.into()).unwrap();
            let node = node.get();
            let render_layer = &node.render_layer;
            let rl = render_layer;
            rl.bounds_with_children
        })
    }
    pub fn cointains_point(&self, point: impl Into<skia_safe::Point>) -> bool {
        let point = point.into();
        self.render_bounds_transformed().contains(point)
    }
    pub fn children_nodes(&self) -> Vec<NodeRef> {
        let node_id: TreeStorageId = self.id.into();
        return {
            self.engine
                .scene
                .with_arena(|arena| node_id.children(arena).map(NodeRef).collect())
        };
    }
    pub fn children(&self) -> Vec<Layer> {
        let node_id: TreeStorageId = self.id.into();
        return {
            self.engine.scene.with_arena(|arena| {
                node_id
                    .children(arena)
                    .filter_map(|cid| self.engine.get_layer(&NodeRef(cid)))
                    .collect()
            })
        };
    }
    #[cfg(feature = "layer_state")]
    pub fn with_state<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&LayerDataProps) -> T,
    {
        let data = self.state.read().unwrap();
        f(&data)
    }
    #[cfg(feature = "layer_state")]
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

        let change = Arc::new(NoopChange::new(self.id.into()));
        self.engine.schedule_change(self.id, change, None);
        let mut model_filter = self.model.color_filter.write().unwrap();
        *model_filter = filter;
    }
    pub fn set_filter_bounds(&self, bounds: impl Into<Option<skia::Rect>>) {
        let mut model_filter_bounds = self.model.filter_bounds.write().unwrap();
        let bounds = bounds.into();
        *model_filter_bounds = bounds;
    }
    pub fn remove(&self) {
        self.engine.mark_for_delete(self.id);
    }
    pub fn set_effect(&self, effect: impl Effect + 'static) {
        let effect = Arc::new(effect);
        effect.init(self);
        let effect_ref = effect.clone();
        let value_id = self.image_filter_progress_value_id();
        self.engine.on_update_value(
            value_id,
            move |l: &Layer, _p| {
                effect_ref.update(l, l.model.image_filter_progress.value());
            },
            false,
        );
        let effect_ref = effect.clone();
        self.engine.on_start_value(
            value_id,
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
        self.engine.on_update_value(size_id, f, once);
    }
    pub fn set_image_cached(&self, image_cached: bool) {
        self.engine.scene.with_arena_mut(|arena| {
            let id = self.id.0;
            let node = arena.get_mut(id).unwrap();
            let node = node.get_mut();
            node.set_image_cached(image_cached);
        });
    }

    pub fn set_picture_cached(&self, picture_cache: bool) {
        self.engine.scene.with_arena_mut(|arena| {
            let id = self.id.0;
            let node = arena.get_mut(id).unwrap();
            let node = node.get_mut();
            node.set_picture_cached(picture_cache);
        });
        if picture_cache == false {
            self.engine.scene.with_renderable_arena_mut(|arena| {
                let id: usize = self.id.into();
                let node = arena.get_mut(&id).unwrap();
                node.draw_cache = None;
            });
        }
    }

    pub(crate) fn set_follow_node(&self, follow_node: Option<NodeRef>) {
        self.engine.scene.with_arena_mut(|node_arena| {
            let node = node_arena.get_mut(self.id().into());
            if let Some(node) = node {
                let scene_node = node.get_mut();
                scene_node._follow_node = follow_node;
            }
        });
    }
}

impl fmt::Debug for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layer")
            .field("id", &self.id)
            // .field("model", &self.model)
            .finish()
    }
}

impl PartialEq for Layer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Layer {}

impl Hash for Layer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<Layer> for NodeRef {
    fn from(val: Layer) -> Self {
        val.id
    }
}

impl From<&Layer> for NodeRef {
    fn from(val: &Layer) -> Self {
        val.id
    }
}

impl<'a> From<&'a Layer> for &'a NodeRef {
    fn from(val: &'a Layer) -> Self {
        &val.id
    }
}

impl From<Layer> for Option<NodeRef> {
    fn from(val: Layer) -> Self {
        Some(val.id)
    }
}
