use core::fmt;
use derive_builder::Builder;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::layers::layer::model::{ContentDrawFunction, PointerHandlerFunction};
use crate::prelude::*;
use crate::types::Size;

/// A trait for structs that can produce into a layertree
pub trait RenderLayerTree {
    fn key(&self) -> String;
    fn mount_layer(&self, layer: Layer);
    fn set_path(&mut self, path: String);
    fn render_layertree(&self) -> LayerTree;
}
/// A layertree renders itself into a layertree
impl RenderLayerTree for LayerTree {
    fn key(&self) -> String {
        self.key.clone()
    }
    fn mount_layer(&self, _layer: Layer) {}
    fn set_path(&mut self, path: String) {
        self.path = path;
    }
    fn render_layertree(&self) -> LayerTree {
        self.clone()
    }
}

/// A struct that represents a definition of a layer hierearchy
/// that can be rendered by the engine into layers
/// key value is used to optimize the rendering of the layer
/// by reusing the layer when the key is the same
#[derive(Clone, Builder, Default)]
#[builder(public, default)]
pub struct LayerTree {
    path: String,
    #[builder(setter(into, strip_option))]
    pub key: String,
    #[builder(setter(into, strip_option), default)]
    pub background_color: Option<(PaintColor, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub border_color: Option<(Color, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub border_width: Option<(f32, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub border_style: Option<BorderStyle>,
    #[builder(setter(into, strip_option), default)]
    pub border_corner_radius: Option<(BorderRadius, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub size: Option<(Size, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub position: Option<(Point, Option<Transition>)>,
    #[builder(setter(into, strip_option))]
    pub scale: Option<(Point, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_offset: Option<(Point, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_radius: Option<(f32, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_color: Option<(Color, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub shadow_spread: Option<(f32, Option<Transition>)>,
    #[builder(setter(custom))]
    pub content: Option<ContentDrawFunction>,
    #[builder(setter(into, strip_option), default)]
    pub blend_mode: Option<BlendMode>,
    #[builder(setter(into, strip_option), default)]
    pub layout_style: Option<taffy::Style>,
    #[builder(setter(into, strip_option))]
    pub opacity: Option<(f32, Option<Transition>)>,
    #[builder(setter(into, strip_option), default)]
    pub image_cache: Option<bool>,

    #[builder(setter(custom))]
    pub on_pointer_move: Option<PointerHandlerFunction>,
    #[builder(setter(custom))]
    pub on_pointer_in: Option<PointerHandlerFunction>,
    #[builder(setter(custom))]
    pub on_pointer_out: Option<PointerHandlerFunction>,
    #[builder(setter(custom))]
    pub on_pointer_press: Option<PointerHandlerFunction>,
    #[builder(setter(custom))]
    pub on_pointer_release: Option<PointerHandlerFunction>,
    
    
    /// The children of the layer tree are elements that can render a layertree
    #[builder(setter(custom))]
    pub children: Option<Vec<Arc<dyn RenderLayerTree>>>,
}
/// A builder for the LayerTree struct
///
impl LayerTreeBuilder {
    pub fn children(&mut self, children: Vec<impl RenderLayerTree + 'static>) -> &mut Self {
        let children = children
            .into_iter()
            .map(|child| {
                let child: Arc<dyn RenderLayerTree> = Arc::new(child);
                child
            })
            .collect::<Vec<Arc<dyn RenderLayerTree>>>();

        self.children = Some(children.into());

        self
    }

    pub fn content<F: Into<ContentDrawFunction>>(
        &mut self,
        content_handler: Option<F>,
    ) -> &mut Self {
        if let Some(content_handler) = content_handler {
            let content = Some(content_handler.into());
            self.content = Some(content);
        }
        self
    }
    pub fn on_pointer_move<F: Into<PointerHandlerFunction>>(
        &mut self,
        on_pointer_move: F,
    ) -> &mut Self {
        let on_pointer_move = Some(on_pointer_move.into());
        self.on_pointer_move = Some(on_pointer_move);
        self
    }
    pub fn on_pointer_in<F: Into<PointerHandlerFunction>>(
        &mut self,
        on_pointer_in: F,
    ) -> &mut Self {
        let on_pointer_in = Some(on_pointer_in.into());
        self.on_pointer_in = Some(on_pointer_in);
        self
    }
    pub fn on_pointer_out<F: Into<PointerHandlerFunction>>(
        &mut self,
        on_pointer_out: F,
    ) -> &mut Self {
        let on_pointer_out = Some(on_pointer_out.into());
        self.on_pointer_out = Some(on_pointer_out);
        self
    }
    pub fn on_pointer_press<F: Into<PointerHandlerFunction>>(
        &mut self,
        on_pointer_press: F,
    ) -> &mut Self {
        let on_pointer_press = Some(on_pointer_press.into());
        self.on_pointer_press = Some(on_pointer_press);
        self
    }
    pub fn on_pointer_release<F: Into<PointerHandlerFunction>>(
        &mut self,
        on_pointer_release: F,
    ) -> &mut Self {
        let on_pointer_release = Some(on_pointer_release.into());
        self.on_pointer_release = Some(on_pointer_release);
        self
    }
}

impl fmt::Debug for LayerTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let children = self.children.as_ref().map(|children| {
            children
                .iter()
                .map(|child| child.as_ref().render_layertree())
                .collect::<Vec<LayerTree>>()
        });
        f.debug_struct("ViewLayer")
            .field("key", &self.key)
            .field("background_color", &self.background_color)
            .field("border_color", &self.border_color)
            .field("border_width", &self.border_width)
            .field("border_style", &self.border_style)
            .field("border_corner_radius", &self.border_corner_radius)
            .field("size", &self.size)
            .field("position", &self.position)
            .field("scale", &self.scale)
            .field("shadow_offset", &self.shadow_offset)
            .field("shadow_radius", &self.shadow_radius)
            .field("shadow_color", &self.shadow_color)
            .field("shadow_spread", &self.shadow_spread)
            // .field("content", &self.content)
            .field("blend_mode", &self.blend_mode)
            .field("layout_style", &self.layout_style)
            .field("opacity", &self.opacity)
            .field("children", &children)
            .finish()
    }
}

impl Hash for LayerTree {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}
impl PartialEq for LayerTree {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for LayerTree {}

impl From<PaintColor> for (PaintColor, Option<Transition>) {
    fn from(val: PaintColor) -> Self {
        (val, None)
    }
}

impl From<Color> for (Color, Option<Transition>) {
    fn from(val: Color) -> Self {
        (val, None)
    }
}

impl From<BorderStyle> for (BorderStyle, Option<Transition>) {
    fn from(val: BorderStyle) -> Self {
        (val, None)
    }
}

impl From<BorderRadius> for (BorderRadius, Option<Transition>) {
    fn from(val: BorderRadius) -> Self {
        (val, None)
    }
}

impl From<Size> for (Size, Option<Transition>) {
    fn from(val: Size) -> Self {
        (val, None)
    }
}

impl From<Point> for (Point, Option<Transition>) {
    fn from(val: Point) -> Self {
        (val, None)
    }
}

// Add specific implementations for other types if needed

#[allow(clippy::from_over_into)]
impl Into<Vec<LayerTree>> for LayerTree {
    fn into(self) -> Vec<LayerTree> {
        vec![self]
    }
}

impl RenderLayerTree for Arc<dyn RenderLayerTree> {
    fn key(&self) -> String {
        self.as_ref().key()
    }
    fn set_path(&mut self, _path: String) {
        // self.as_ref().set_path(path);
    }
    fn mount_layer(&self, layer: Layer) {
        self.as_ref().mount_layer(layer);
    }
    fn render_layertree(&self) -> LayerTree {
        self.as_ref().render_layertree()
    }
}

#[macro_export]
macro_rules! layer_trees {
    ($($arg:expr),* $(,)?) => {
        {
            let mut vec = Vec::new();
            $(
                let item: std::sync::Arc<dyn RenderLayerTree> = std::sync::Arc::new($arg);
                vec.push(item);
            )*
            vec
        }
    };
}

#[macro_export]
macro_rules! layer_trees_opt {
    ($($arg:expr),* $(,)?) => {
        {
            let mut vec = Vec::new();
            $(
                if let Some(item) = $arg {
                    let item: std::sync::Arc<dyn RenderLayerTree> = std::sync::Arc::new(item);
                    vec.push(item);
                }
            )*
            vec
        }
    };
}
