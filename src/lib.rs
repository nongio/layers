#![deny(warnings)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
// negative_impl is used to prevent the compiler from using
// the default implementation of the trait Interpolable for PaintColor
#![feature(negative_impls)]

//! # Layers
//! Layers is an engine to manage, interact and animate 2D graphical objects.
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
pub mod layers;
pub mod renderer;
pub mod types;

#[cfg(feature = "export-skia")]
pub extern crate skia_safe as skia;
#[cfg(feature = "export-taffy")]
pub extern crate taffy;
