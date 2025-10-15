#![deny(warnings)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nongio/layers/refs/heads/main/assets/LayersEngine-dark.png"
)]

//! # Layers Engine
//!
//! **Layers** is an animation and rendering engine designed for user interfaces.
//! It utilizes a scene graph for efficient, retained mode rendering of UI elements,
//! optimizing common interpolations such as opacity, 2D transformations, and blending.
//!
//! ## Key Features
//!
//! - **Scene Graph**: Manages nodes (graphical layers) including text, simple shapes (e.g., rectangles), and external textures.
//! - **Animatable Properties**: Nodes have properties that can be animated, with changes scheduled and executed by the engine using a Command pattern. This ensures a consistent API for both immediate and animated changes.
//! - **Optimized Rendering**: Uses a display list for efficient rendering commands.
//! - **Animation Hooks**: The API exposes hooks at different stages of the animation progress.
//! - **Layout Engine**: Uses the Taffy library for layout calculations.
//! - **Multi-threaded**: Supports concurrent updates to layer properties.
//! - **Spring Physics Animations**: Supports spring physics for animations.
//! - **Easing Functions**: Includes a variety of easing functions for animations.
//! - **Incremental Rendering**: Only redraws the portions of the scene that have changed.
//! - **Debugger**: The engine has a built-in debugger for visualizing the scene graph
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
//! - **Display List**: Uses a display list to optimize rendering commands.
//! - **Damage tracking**: On every update, the damage of the scene is calculated.
//!
//! ## Multi-threaded Support
//!
//! - **Concurrent Updates**: Layer properties are updated across multiple threads.
//!
//! ## Backend Support (Skia)
//!
//! - **Drawing Library**: Uses the Skia library.
//! - **Supported Backends**:
//!   - **OpenGL**: Using FBO (Framebuffer Objects)
//!   - **EGL**: For OpenGL ES contexts
//!   - **Image**: For testing purposes
//!
//! ## Taffy Layout
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
pub mod utils;
pub mod view;

#[cfg(doc)]
pub mod guides {
    /// Detailed walkthrough of `Engine::update` pipeline stages.
    pub mod engine_update_stages {
        #![doc = include_str!("../docs/engine-update-stages.md")]
    }

    /// Overview of how damage regions are tracked and propagated.
    pub mod damage_tracking {
        #![doc = include_str!("../docs/damage-tracking.md")]
    }

    /// Reference for the layer follower system.
    pub mod layer_followers {
        #![doc = include_str!("../docs/layer-followers.md")]
    }

    /// Guide to pointer hit-testing and event dispatch.
    pub mod pointers {
        #![doc = include_str!("../docs/pointers.md")]
    }

    /// Walkthrough of the portal system.
    pub mod portals {
        #![doc = include_str!("../docs/portals.md")]
    }

    /// Primer on scene damage concepts and terminology.
    pub mod damage {
        #![doc = include_str!("../docs/damage.md")]
    }
}

#[cfg(feature = "export-skia")]
pub extern crate skia_bindings as sb;
#[cfg(feature = "export-skia")]
pub extern crate skia_safe as skia;
#[cfg(feature = "export-taffy")]
pub extern crate taffy;
