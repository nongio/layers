pub(crate) mod model;
pub(crate) mod render_layer;
pub(crate) mod state;
pub(crate) use self::model::ModelLayer;

use model::ContentDrawFunctionInternal;
use render_layer::RenderLayer;
use skia::{ColorFilter, Contains, ImageFilter};
use std::cell::RefCell;
use std::collections::HashSet;
use std::{fmt, sync::Arc};
use std::{
    hash::{Hash, Hasher},
    sync::RwLock,
};
use taffy::style::Display;
use taffy::style::Style;

use self::model::{ContentDrawFunction, PointerHandlerFunction};

use crate::engine::{command::*, PointerEventType};
use crate::engine::{node::RenderableFlags, TransactionCallback};
use crate::engine::{Engine, NodeRef, TransactionRef};
use crate::types::*;
use crate::{
    drawing::render_node_tree,
    engine::{animation::*, storage::TreeStorageId, AnimatedNodeChange},
};

#[cfg(feature = "layer_state")]
use state::LayerDataProps;

// Thread-local set to track layers currently being rendered via as_content
// to prevent infinite recursion when a follower is a descendant of its leader
thread_local! {
    static RENDERING_LAYERS: RefCell<HashSet<NodeRef>> = RefCell::new(HashSet::new());
}

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
        // Invalidate hit test node list since visibility affects hit-testing
        self.engine.invalidate_hit_test_node_list();
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
        // Invalidate hit test node list since pointer_events affects hit-testing
        self.engine.invalidate_hit_test_node_list();
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

    /// Sets the anchor point while compensating the `position` so the layer stays in the
    /// same place on screen. Returns the newly applied position.
    pub fn set_anchor_point_preserving_position(&self, anchor_point: impl Into<Point>) -> Point {
        let new_anchor = anchor_point.into();
        let current_anchor = self.anchor_point();

        if new_anchor.x == current_anchor.x && new_anchor.y == current_anchor.y {
            return self.position();
        }

        let layout_size = {
            let layout_tree = self.engine.layout_tree.read().unwrap();
            let layout = layout_tree
                .layout(self.layout_id)
                .expect("layout is available for every layer");
            layout.size
        };

        let scale = self.scale();
        let rotation = self.rotation();

        let mut linear = M44::scale(scale.x, scale.y, 1.0);
        let rotate_x = M44::rotate(
            V3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            rotation.x,
        );
        let rotate_y = M44::rotate(
            V3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            rotation.y,
        );
        let rotate_z = M44::rotate(
            V3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            rotation.z,
        );

        linear = M44::concat(&linear, &rotate_x);
        linear = M44::concat(&linear, &rotate_y);
        linear = M44::concat(&linear, &rotate_z);

        let mut new_position = self.position();
        let delta_local = (
            (new_anchor.x - current_anchor.x) * layout_size.width,
            (new_anchor.y - current_anchor.y) * layout_size.height,
        );
        let delta = linear.map(delta_local.0, delta_local.1, 0.0, 0.0);
        new_position.x += delta.x;
        new_position.y += delta.y;

        self.set_position(new_position, None);
        self.set_anchor_point(new_anchor, None);

        new_position
    }

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

        let attribute_id = self.model.blend_mode.id;
        self.engine
            .schedule_change(self.id, Arc::new(NoopChange::new(attribute_id)), None);
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
    pub fn render_size_transformed(&self) -> Point {
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
    pub fn render_size(&self) -> Point {
        self.engine.scene.with_arena(|arena| {
            let node = arena.get(self.id.into()).unwrap();
            let node = node.get();
            let render_layer = &node.render_layer;
            let rl = render_layer;
            Point {
                x: rl.bounds.width(),
                y: rl.bounds.height(),
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
        {
            self.engine
                .scene
                .with_arena(|arena| node_id.children(arena).map(NodeRef).collect())
        }
    }
    pub fn children(&self) -> Vec<Layer> {
        let node_id: TreeStorageId = self.id.into();
        {
            self.engine.scene.with_arena(|arena| {
                node_id
                    .children(arena)
                    .filter_map(|cid| self.engine.get_layer(&NodeRef(cid)))
                    .collect()
            })
        }
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

        // Clear previous handlers so we don’t accumulate duplicates
        self.engine.clear_value_handlers(value_id);

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
        let value_id = self.image_filter_progress_value_id();

        // Clear previous handlers so we don’t accumulate duplicates
        self.engine.clear_value_handlers(value_id);
        let mut effect = self.effect.write().unwrap();

        if let Some(effect) = &*effect {
            effect.finish(self);
        }

        *effect = None;
    }
    pub fn clear_on_change_size_handlers(&self) {
        let size_id = self.model.size.id;
        self.engine.clear_value_handlers(size_id);
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
        if !picture_cache {
            self.engine.scene.with_renderable_arena_mut(|arena| {
                let id: usize = self.id.into();
                let node = arena.get_mut(&id).unwrap();
                node.draw_cache = None;
            });
        }
    }

    pub fn add_follower_node(&self, follower: impl Into<NodeRef>) {
        let follower = follower.into();
        self.engine.scene.with_arena_mut(|node_arena| {
            let node = node_arena.get_mut(self.id.0);
            if let Some(node) = node {
                let scene_node = node.get_mut();
                scene_node.followers.insert(follower);
            }
        });
        let attribute_id = self.model.blend_mode.id;
        self.engine
            .schedule_change(self.id, Arc::new(NoopChange::new(attribute_id)), None);
    }
    pub fn remove_follower_node(&self, follower: impl Into<NodeRef>) {
        let follower = follower.into();
        self.engine.scene.with_arena_mut(|node_arena| {
            let node = node_arena.get_mut(self.id.0);
            if let Some(node) = node {
                let scene_node = node.get_mut();
                scene_node.followers.remove(&follower);
            }
        });
    }
    pub fn as_content(&self) -> ContentDrawFunction {
        let engine_ref = self.engine.clone();
        let layer_id = self.id();
        let draw_function = move |c: &skia::Canvas, w: f32, h: f32| {
            // Check if this layer is already being rendered to prevent infinite recursion
            // This can happen when a follower is a descendant of its leader
            let already_rendering = RENDERING_LAYERS.with(|set| set.borrow().contains(&layer_id));
            if already_rendering {
                // Return empty damage to break the recursion
                return skia::Rect::from_xywh(0.0, 0.0, w, h);
            }

            // Mark this layer as being rendered
            RENDERING_LAYERS.with(|set| {
                set.borrow_mut().insert(layer_id);
            });

            let scene = engine_ref.scene.clone();
            let damage = scene
                .try_with_arena(|arena| {
                    scene.with_renderable_arena(|renderable_arena| {
                        render_node_tree(layer_id, arena, renderable_arena, c, 1.0);
                    });
                    // the damage of a mirrored layer is the bounds with children
                    if let Some(scene_node) = arena.get(layer_id.0) {
                        if scene_node.is_removed() {
                            return skia::Rect::from_xywh(0.0, 0.0, w, h);
                        }
                        let scene_node = scene_node.get();
                        let render_layer = &scene_node.render_layer;
                        let damage = render_layer.bounds_with_children;
                        return damage;
                    }
                    skia::Rect::from_xywh(0.0, 0.0, w, h)
                })
                .unwrap_or(skia::Rect::from_xywh(0.0, 0.0, w, h));

            // Unmark this layer
            RENDERING_LAYERS.with(|set| {
                set.borrow_mut().remove(&layer_id);
            });

            damage
        };
        ContentDrawFunction::from(draw_function)
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
