pub mod animations;
pub mod command;
pub mod node;
pub mod rendering;
pub mod scene;
pub mod storage;

use self::animations::{Easing, Transition};
use self::node::{NodeFlags, SceneNode};
use self::scene::{Scene, SceneRef};
use std::sync::Arc;

use self::animations::*;
use self::storage::*;

pub struct Timestamp(f64);

pub trait Command {
    fn execute(&self, progress: f64) -> NodeFlags;
}

pub trait WithTransition {
    fn transition(&self) -> Option<Transition<Easing>>;
}

pub trait CommandWithTransition: Command + WithTransition + Send + Sync {}

#[derive(Clone)]
pub struct AnimatedNodeChange {
    pub change: Arc<dyn CommandWithTransition>,
    animation_id: Option<FlatStorageId>,
    node: SceneNode,
}

#[derive(Clone)]
pub struct AnimationState(Animation, f64, bool);

pub fn setup_ecs() -> SceneRef {
    Scene::create()
}

pub trait Engine: Send + Sync {
    // fn add_renderable(&self, r: Arc<dyn Renderable>) -> TreeStorageId;
    // fn add_animation(&self, animation: Animation) -> FlatStorageId;
    // fn add_change_with_animation(
    //     &self,
    //     target_id: TreeStorageId,
    //     change: Arc<dyn CommandWithTransition>,
    //     animation_id: Option<FlatStorageId>,
    // ) -> FlatStorageId;
    fn add_change(&self, target_id: TreeStorageId, change: Arc<dyn CommandWithTransition>)
        -> usize;
}
pub trait ChangeInvoker {
    fn set_engine(&self, engine: Arc<dyn Engine>, id: TreeStorageId);
}
