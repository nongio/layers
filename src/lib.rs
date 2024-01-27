#![deny(warnings)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

//! # Layers engine
//! Layers is a rendering engine for animated user interfaces. It uses a scene graph to render the nodes in retained mode, optmising the most common UI interpolations (opacity, 2d transformations, blending).
//! Nodes of the scene graph are graphical layers like text or simple shapes like rectangles but can also be external textures. Nodes have animatable properties that accepts changes and schedule them in the engine to be executed. Using this Command pattern, changes to the nodes have a consistent api between immediate changes and animated changes.
//! The rendering commands are optimised using display list.Node properties can be animated, hooks to different stages of the animation progress are exposed by the API.

//! A `Layer` similar to other graphics engines is a 2D object that can contains
//! a rasterised content, can be positioned, rotated, scaled and animated.
//! Similar to the DOM in a web browser, the layers can be nested to create
//! complex 2D objects.
//! The layers can either contain a rasterised content or be a container for
//! other layers.
//! The layers have also drawing properties like border, background, shadow,
//! opacity, etc.
//! Layers engine uses a retained mode rendering model. It means that the engine
//! keeps a tree of layers and only redraws the layers that have changed.
//!
//! The engine is designed to be used in a multi-threaded environment. The
//! layers properties are updated in multiple threads.
//!
//! The drawing is done using the Skia library.
//! The backendd supported are:
//! - OpenGL, EGL using FBO,
//! - Image (for testing purpose)
//!
//! The layout is done using the Taffy library based on the Flexbox model.
//!

pub mod api;
pub mod drawing;
mod easing;
pub mod engine;
mod layers;
pub mod prelude;
pub mod renderer;
pub mod types;
#[cfg(feature = "export-skia")]
pub extern crate skia_safe as skia;
#[cfg(feature = "export-taffy")]
pub extern crate taffy;
