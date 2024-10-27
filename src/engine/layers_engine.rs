use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc, Once, RwLock,
    },
};

use indextree::NodeId;
use taffy::Style;

use crate::{
    layers::layer::{state::LayerDataProps, ModelLayer},
    prelude::{Layer, Transition},
    types::Point,
};

use super::{
    node::SceneNode, scene::Scene, storage::TreeStorageNode, AnimatedNodeChange, AnimationRef,
    AnimationState, Engine, NodeRef, TransactionCallback, TransactionRef,
};

/// Public API for the Layers Engine
/// ## Usage: Setup a basic scene with a root layer
/// ```rust
/// use layers::prelude::*;
///
/// let engine = LayersEngine::new(800.0, 600.0);
/// let layer = engine.new_layer();
/// let engine = LayersEngine::new(1024.0, 768.0);
/// let root_layer = engine.new_layer();
/// root_layer.set_position(Point { x: 0.0, y: 0.0 }, None);

/// root_layer.set_background_color(
///     PaintColor::Solid {
///         color: Color::new_rgba255(180, 180, 180, 255),
///     },
///    None,
/// );
/// root_layer.set_border_corner_radius(10.0, None);
/// root_layer.set_layout_style(taffy::Style {
///     position: taffy::Position::Absolute,
///     display: taffy::Display::Flex,
///     flex_direction: taffy::FlexDirection::Column,
///     justify_content: Some(taffy::JustifyContent::Center),
///     align_items: Some(taffy::AlignItems::Center),
///     ..Default::default()
/// });
/// engine.scene_add_layer(root_layer.clone());
/// ```
/// ## Usage: Update the engine
/// ```rust
/// use layers::prelude::*;
///
/// let engine = LayersEngine::new(800.0, 600.0);
/// // setup the scene...
/// engine.update(0.016);
/// ```
#[derive(Clone)]
pub struct LayersEngine {
    pub(crate) engine: Arc<Engine>,
}
impl std::fmt::Debug for LayersEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayersEngine").finish()
    }
}
pub(crate) static INIT: Once = Once::new();
pub(crate) static ENGINE_ID: AtomicUsize = AtomicUsize::new(0);
pub(crate) static mut ENGINES: Option<RwLock<HashMap<usize, Arc<Engine>>>> = None;

pub(crate) fn initialize_engines() {
    unsafe {
        ENGINES = Some(RwLock::new(HashMap::new()));
    }
}
impl LayersEngine {
    /// Create a new engine with a scene initialized with the given width and height
    pub fn new(width: f32, height: f32) -> Self {
        let engines = unsafe {
            INIT.call_once(initialize_engines);
            ENGINES.as_ref().unwrap()
        };
        let id = ENGINE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let new_engine = Engine::create(id, width, height);
        engines.write().unwrap().insert(id, new_engine.clone());
        Self { engine: new_engine }
    }
    /// Set the size of the scene
    pub fn set_scene_size(&self, width: f32, height: f32) {
        self.engine.scene.set_size(width, height);
    }
    /// Create a new layer associated with the engine
    pub fn new_layer(&self) -> Layer {
        let model = Arc::new(ModelLayer::default());

        let mut lt = self.engine.layout_tree.write().unwrap();

        let layout = lt.new_leaf(Style::default()).unwrap();

        Layer {
            engine: self.engine.clone(),
            model,
            id: Arc::new(RwLock::new(None)),
            key: Arc::new(RwLock::new(String::new())),
            layout_node_id: layout,
            hidden: Arc::new(AtomicBool::new(false)),
            pointer_events: Arc::new(AtomicBool::new(true)),
            image_cache: Arc::new(AtomicBool::new(false)),
            state: Arc::new(RwLock::new(LayerDataProps::default())),
            effect: Arc::new(RwLock::new(None)),
        }
    }
    /// Create a new animation from the given transition
    ///
    /// # Arguments
    /// * `autostart`: If `autostart` is true, the animation start time will be the current engine time
    pub fn new_animation(&self, transition: Transition, autostart: bool) -> AnimationRef {
        self.engine
            .add_animation_from_transition(transition, autostart)
    }
    /// Attach an existing transaction to an exiting animation
    pub fn attach_animation(&self, transaction: TransactionRef, animation: AnimationRef) {
        self.engine.attach_animation(transaction, animation);
    }
    /// Start an animation with a delay
    ///
    /// # Arguments
    /// * `animation`: The animation to start
    /// * `delay`: The delay before the animation starts in seconds
    pub fn start_animation(&self, animation: AnimationRef, delay: f32) {
        self.engine.start_animation(animation, delay);
    }
    /// Add a new set of changes to the engine
    /// If an animation is provided, the changes will be animated
    /// using the same animation
    pub fn add_animated_changes(
        &self,
        animated_changes: &[AnimatedNodeChange],
        animation: impl Into<Option<AnimationRef>>,
    ) -> Vec<TransactionRef> {
        self.engine.schedule_changes(animated_changes, animation)
    }
    /// Tick the engine by the given delta time
    /// Returns true if the engine has changes that needs to be rendered
    /// # Arguments
    /// * `dt`: The delta time in seconds
    pub fn update(&self, dt: f32) -> bool {
        self.engine.update(dt)
    }
    /// Update the nodes in the scene and returns the bounding box of the changes
    pub fn update_nodes(&self) -> skia_safe::Rect {
        self.engine.update_nodes()
    }
    /// Add a new layer to the scene
    /// The layer will be added to the root of the scene
    pub fn scene_add_layer(&self, layer: impl Into<Layer>) -> NodeRef {
        self.engine.scene_add_layer(layer, None)
    }
    /// Add a new layer to the scene, attached to the given parent
    ///
    /// If the parent is None, the layer will be attached to the root of the scene
    pub fn scene_add_layer_to(
        &self,
        layer: impl Into<Layer>,
        parent: impl Into<Option<NodeRef>>,
    ) -> NodeRef {
        let parent = parent.into();
        self.engine.scene_add_layer(layer, parent)
    }
    /// Add a new layer to the scene, attached to the given parent,
    /// maintaining the given position.
    ///
    /// If the parent is None, the layer will be attached to the root of the scene
    pub fn scene_add_layer_to_positioned(
        &self,
        layer: impl Into<Layer>,
        parent: impl Into<Option<NodeRef>>,
    ) -> NodeRef {
        let parent = parent.into();
        self.engine.scene_add_layer_to_positioned(layer, parent)
    }
    /// Remove a layer from the scene
    ///
    /// Note: The layer is immediatly hidden, will be removed from the scene at the next update
    pub fn scene_remove_layer(&self, node: impl Into<Option<NodeRef>>) {
        if let Some(node) = node.into() {
            self.engine.mark_for_delete(node);
        }
    }
    /// Set the root layer of the scene
    pub fn scene_set_root(&self, layer: impl Into<Layer>) -> NodeRef {
        self.engine.scene_set_root(layer)
    }
    /// Get a node from the scene if it exists
    pub fn scene_get_node(&self, node: &NodeRef) -> Option<TreeStorageNode<SceneNode>> {
        self.engine.scene_get_node(node)
    }
    /// Get the parent of a node if it exists
    pub fn scene_get_node_parent(&self, node: &NodeRef) -> Option<NodeRef> {
        self.engine.scene_get_node_parent(node)
    }
    /// Return reference to the scene
    pub fn scene(&self) -> &Arc<Scene> {
        &self.engine.scene
    }
    /// Return reference to the root node of the scene
    pub fn scene_root(&self) -> Option<NodeRef> {
        *self.engine.scene_root.read().unwrap()
    }
    /// Return the root layer of the scene
    pub fn scene_root_layer(&self) -> Option<SceneNode> {
        let root_id = self.scene_root()?;
        let node = self.scene_get_node(&root_id)?;
        Some(node.get().clone())
    }
    /// Get the topmost layer under a given point
    pub fn scene_layer_at(&self, point: Point) -> Option<NodeRef> {
        self.engine.layer_at(point)
    }
    /// Get the current damage rect of the scene
    pub fn damage(&self) -> skia_safe::Rect {
        *self.engine.damage.read().unwrap()
    }
    /// Clear the damage rect of the scene
    #[profiling::function]
    pub fn clear_damage(&self) {
        *self.engine.damage.write().unwrap() = skia_safe::Rect::default();
    }
    /// Sends a pointer move event to the engine
    ///
    /// If `root_id` is provided, the event will be sent to the given root layer
    /// and propagated to its children
    pub fn pointer_move(&self, point: impl Into<Point>, root_id: impl Into<Option<NodeId>>) {
        self.engine.pointer_move(point, root_id);
    }
    /// Sends a pointer button down event to the engine
    pub fn pointer_button_down(&self) {
        self.engine.pointer_button_down();
    }
    /// Sends a pointer button up event to the engine
    pub fn pointer_button_up(&self) {
        self.engine.pointer_button_up();
    }
    /// Returns the current hover node
    pub fn current_hover(&self) -> Option<NodeRef> {
        self.engine.current_hover()
    }
    /// Returns the current pointer position
    pub fn get_pointer_position(&self) -> Point {
        self.engine.get_pointer_position()
    }
    /// Returns the transaction for the given reference if it exists
    pub fn get_transaction(&self, transaction: TransactionRef) -> Option<AnimatedNodeChange> {
        self.engine.get_transaction_for_value(transaction.value_id)
    }
    /// Returns the transaction for the given value_id if it exists
    pub fn get_transaction_for_value(&self, value_id: usize) -> Option<AnimatedNodeChange> {
        self.engine.get_transaction_for_value(value_id)
    }
    /// Returns the animation for the given reference if it exists
    pub fn get_animation(&self, animation: AnimationRef) -> Option<AnimationState> {
        self.engine.get_animation(animation)
    }

    pub fn on_finish<F: Into<TransactionCallback>>(
        &self,
        transaction: TransactionRef,
        handler: F,
        once: bool,
    ) {
        self.engine.on_finish(transaction, handler, once);
    }
    pub fn on_update<F: Into<TransactionCallback>>(
        &self,
        transaction: TransactionRef,
        handler: F,
        once: bool,
    ) {
        self.engine.on_update(transaction, handler, once);
    }
    pub fn on_start<F: Into<TransactionCallback>>(
        &self,
        transaction: TransactionRef,
        handler: F,
        once: bool,
    ) {
        self.engine.on_start(transaction, handler, once);
    }
    pub fn cancel_animation(&self, animation: AnimationRef) {
        self.engine.cancel_animation(animation);
    }
    #[cfg(feature = "debugger")]
    /// Start the debugger server
    ///
    /// Can be accessed at `http://localhost:8000/client/index.html`
    pub fn start_debugger(&self) {
        layers_debug_server::start_debugger_server(self.engine.clone());
    }
}
