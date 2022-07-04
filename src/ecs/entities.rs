use std::sync::{atomic::AtomicBool, Arc, RwLock};

use skia_safe::Picture;

use crate::{
    layer::{ModelLayer, RenderLayer},
    skcache::render_layer_cache,
};

#[derive(Clone, Debug)]
pub struct SkiaCache {
    pub picture: Option<Picture>,
}

#[derive(Clone, Debug)]
pub enum Entities {
    Root {
        children: Arc<RwLock<Vec<Entities>>>,
    },
    Layer {
        model: ModelLayer,
        layer: Arc<RwLock<RenderLayer>>,
        cache: Arc<RwLock<SkiaCache>>,
        needs_paint: Arc<AtomicBool>,
        parent: Arc<RwLock<Option<Entities>>>,
        children: Arc<RwLock<Vec<Entities>>>,
    },
}

pub trait HasId {
    fn id(&self) -> usize;
}
pub trait HasHierarchy {
    fn parent(&self) -> Option<Entities>;
    fn set_parent(&mut self, parent: Entities);
    fn children(&self) -> Vec<Entities>;
    fn children_mut(&self) -> Arc<RwLock<Vec<Entities>>>;
    fn add_child(&mut self, child: &mut Entities);
    fn remove_child(&self, child_id: usize);
}

impl HasId for Entities {
    fn id(&self) -> usize {
        match self {
            Entities::Root { .. } => 0,
            Entities::Layer { model, .. } => model.id,
        }
    }
}

impl HasHierarchy for Entities {
    fn parent(&self) -> Option<Entities> {
        match self {
            Entities::Root { .. } => None,
            Entities::Layer {
                parent: maybe_parent,
                ..
            } => match &*maybe_parent.read().unwrap() {
                Some(parent) => Some(parent.clone()),
                None => None,
            },
        }
    }
    fn set_parent(&mut self, new_parent: Entities) {
        let id = self.id();
        match self {
            Entities::Root { .. } => (),
            Entities::Layer {
                parent: maybe_parent,
                ..
            } => {
                let parent_handle = maybe_parent.clone();
                let mut parent_handle = parent_handle.write().unwrap();
                if let Some(parent) = &*parent_handle {
                    if new_parent.id() != parent.id() {
                        parent.remove_child(id);
                    }
                }

                *parent_handle = Some(new_parent.clone());
            }
        }
    }
    fn children(&self) -> Vec<Entities> {
        match self {
            Entities::Root { children, .. } => children.read().unwrap().clone(),
            Entities::Layer { children, .. } => children.read().unwrap().clone(),
        }
    }
    fn children_mut(&self) -> Arc<RwLock<Vec<Entities>>> {
        match self {
            Entities::Root { children, .. } => children.clone(),
            Entities::Layer { children, .. } => children.clone(),
        }
    }
    fn add_child(&mut self, child: &mut Entities) {
        let children = match self {
            Entities::Root { children, .. } => children.clone(),
            Entities::Layer { children, .. } => children.clone(),
        };
        child.set_parent(self.clone());
        children.write().unwrap().push(child.clone());
    }
    fn remove_child(&self, child_id: usize) {
        let children = self.children_mut();
        children.write().unwrap().remove(child_id);
    }
}

pub trait PaintCache {
    fn repaint_if_needed(&self);
}

impl PaintCache for Entities {
    fn repaint_if_needed(&self) {
        match self {
            Entities::Root { .. } => (),
            Entities::Layer {
                model,
                layer,
                needs_paint,
                cache,
                ..
            } => {
                let new_layer = model.render_layer();
                if needs_paint.swap(false, std::sync::atomic::Ordering::Relaxed) {
                    cache.write().unwrap().picture = render_layer_cache(&new_layer);
                }
                {
                    *layer.write().unwrap() = new_layer;
                }
            }
        }
    }
}
impl Entities {
    pub fn new_root() -> Entities {
        Entities::Root {
            children: Arc::new(RwLock::new(Vec::new())),
        }
    }
    pub fn new_layer(model: ModelLayer) -> Entities {
        let cache = SkiaCache { picture: None };
        let children = Vec::new();
        let layer = model.render_layer();

        Entities::Layer {
            model,
            layer: Arc::new(RwLock::new(layer)),
            cache: Arc::new(RwLock::new(cache)),
            needs_paint: Arc::new(AtomicBool::new(true)),
            parent: Arc::new(RwLock::new(None)),
            children: Arc::new(RwLock::new(children)),
        }
    }
}

impl PartialEq for Entities {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
