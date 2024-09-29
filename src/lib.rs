#![deny(warnings)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

//! # Layers Engine
//!
//! **Layers** is a rendering engine designed for animated user interfaces.
//! It utilizes a scene graph for efficient, retained mode rendering of UI elements,
//! optimizing common interpolations such as opacity, 2D transformations, and blending.
//!
//! ## Key Features
//!
//! - **Scene Graph**: Manages nodes (graphical layers) including text, simple shapes (e.g., rectangles), and external textures.
//! - **Animatable Properties**: Nodes have properties that can be animated, with changes scheduled and executed by the engine using a Command pattern. This ensures a consistent API for both immediate and animated changes.
//! - **Optimized Rendering**: Uses a display list for efficient rendering commands.
//! - **Animation Hooks**: The API exposes hooks at different stages of the animation progress.
//!
//! ## Layer
//!
//! A `Layer` is a 2D object in the engine with the following capabilities:
//!
//! - **Rasterized Content**: Can contain rasterized images or be a container for other layers.
//! - **Transformations**: Supports positioning, rotation, scaling, and animation.
//! - **Nesting**: Layers can be nested to create complex 2D objects, similar to the DOM in web browsers.
//! - **Drawing Properties**: Includes properties such as border, background, shadow, and opacity.
//!
//! ## Rendering Model
//!
//! - **Retained Mode**: The engine maintains a tree of layers and only redraws those that have changed.
//!
//! ## Multi-threaded Support
//!
//! - **Concurrent Updates**: Layer properties are updated across multiple threads.
//!
//! ## Backend Support
//!
//! - **Drawing Library**: Uses the Skia library.
//! - **Supported Backends**:
//!   - **OpenGL**: Using FBO (Framebuffer Objects)
//!   - **EGL**: For OpenGL ES contexts
//!   - **Image**: For testing purposes
//!
//! ## Layout
//!
//! - **Layout Engine**: Utilizes the Taffy library, based on the Flexbox model.
//!

pub mod api;
pub mod drawing;
mod easing;
pub mod engine;
mod layers;
pub mod prelude;
pub mod renderer;
pub mod types;
mod utils;
pub mod view;

#[cfg(feature = "export-skia")]
pub extern crate skia_bindings as sb;
#[cfg(feature = "export-skia")]
pub extern crate skia_safe as skia;
#[cfg(feature = "export-taffy")]
pub extern crate taffy;
