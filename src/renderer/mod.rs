//! Renderers that use skia to draw the models to different backends.
//!
//! - [`skia_image`]: a raster image, always available.
//! - `skia_fbo`: an OpenGL framebuffer object, behind the `gl` feature.
//! - `skia_vk`: a host-owned `VkImage`, behind the `vulkan` feature.
#[cfg(feature = "gl")]
pub mod skia_fbo;
pub mod skia_image;
#[cfg(feature = "vulkan")]
pub mod skia_vk;
