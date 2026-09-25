//! Rendering into a host-owned Vulkan image.
//!
//! The Vulkan counterpart of `skia_fbo`: the host creates the instance, the
//! device, the Skia [`DirectContext`] and the `VkImage`, and this module wraps
//! the image as a Skia render target. lay-rs never loads Vulkan itself; every
//! handle is a raw Skia Vulkan type (`skia_safe::gpu::vk`).
//!
//! A [`DirectContext`] for Vulkan comes from
//! `skia_safe::gpu::direct_contexts::make_vulkan` on a
//! `skia_safe::gpu::vk::BackendContext` built from the host's handles. Pin the
//! backend context to the instance's API version with
//! `BackendContext::set_max_api_version`: when the device advertises a newer
//! version than the instance, Skia otherwise asks for entry points the loader
//! does not hand out and context creation returns `None`.

use skia_safe::{
    gpu::{
        backend_render_targets, surfaces,
        vk::{self, Alloc},
        DirectContext, SurfaceOrigin,
    },
    ColorSpace, ColorType, Surface, SurfaceProps,
};

use crate::{
    drawing::scene::{set_node_transform, DrawScene},
    engine::{scene::Scene, NodeRef},
    prelude::render_node_tree,
};

/// A host-owned `VkImage` that Skia can render into.
///
/// Skia neither owns nor frees the image or its memory; the host keeps both
/// alive for as long as any surface wrapping them exists.
#[derive(Clone, Copy, Debug)]
pub struct VkImageTarget {
    /// The image handle. It must have `COLOR_ATTACHMENT` usage.
    pub image: vk::Image,
    /// The image format.
    pub format: vk::Format,
    /// The image tiling.
    pub tiling: vk::ImageTiling,
    /// The layout the image is in when Skia first uses it.
    pub layout: vk::ImageLayout,
    /// The queue family that owns the image, usually the one the
    /// [`DirectContext`] submits to.
    pub queue_family: u32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
}

/// Wraps `target` as a Skia surface with a top-left origin.
///
/// Returns `None` if Skia rejects the image, for example when `color_type`
/// does not match `format` or the format is not renderable.
///
/// # Safety
///
/// `target.image` must be a valid image created on the device `context` was
/// made for, with `COLOR_ATTACHMENT` usage and the given format, tiling and
/// size, currently in `target.layout`. The image and its memory must outlive
/// the returned surface and every GPU submission that uses it.
pub unsafe fn wrap_vk_image(
    context: &mut DirectContext,
    target: &VkImageTarget,
    color_type: ColorType,
    color_space: Option<ColorSpace>,
    props: Option<&SurfaceProps>,
) -> Option<Surface> {
    // SAFETY: the caller guarantees the handle is a live image matching the
    // description; `Alloc::default()` tells Skia it does not own the memory.
    let info = unsafe {
        vk::ImageInfo::new(
            target.image,
            Alloc::default(),
            target.tiling,
            target.layout,
            target.format,
            1,
            target.queue_family,
            None,
            None,
            None,
        )
    };
    let render_target = backend_render_targets::make_vk((target.width, target.height), &info);
    surfaces::wrap_backend_render_target(
        context,
        &render_target,
        SurfaceOrigin::TopLeft,
        color_type,
        color_space,
        props,
    )
}

/// Draws a scene into a host-owned `VkImage`.
#[derive(Clone)]
pub struct SkiaVkRenderer {
    /// The Vulkan context the surface belongs to.
    pub gr_context: DirectContext,
    /// The surface wrapping the host image.
    pub surface: Surface,
}

impl SkiaVkRenderer {
    /// Creates a renderer drawing into `target` through `context`.
    ///
    /// Returns `None` if Skia cannot wrap the image (see [`wrap_vk_image`]).
    /// Drawing records into Skia; call `flush_and_submit` on
    /// [`Self::gr_context`] to submit the work to the queue.
    ///
    /// # Safety
    ///
    /// The requirements of [`wrap_vk_image`] apply for the lifetime of the
    /// renderer.
    pub unsafe fn new(
        context: &DirectContext,
        target: &VkImageTarget,
        color_type: ColorType,
        color_space: Option<ColorSpace>,
    ) -> Option<Self> {
        let mut gr_context = context.clone();
        let props = SurfaceProps::new(Default::default(), skia_safe::PixelGeometry::Unknown);
        // SAFETY: forwarded from the caller.
        let surface = unsafe {
            wrap_vk_image(
                &mut gr_context,
                target,
                color_type,
                color_space,
                Some(&props),
            )
        }?;
        Some(Self {
            gr_context,
            surface,
        })
    }

    /// Returns the surface wrapping the host image.
    pub fn surface(&self) -> Surface {
        self.surface.clone()
    }
}

impl DrawScene for SkiaVkRenderer {
    #[profiling::function]
    fn draw_scene(
        &self,
        scene: std::sync::Arc<Scene>,
        root_id: NodeRef,
        damage: Option<skia_safe::Rect>,
    ) {
        let mut surface = self.surface();
        let canvas = surface.canvas();
        let save_point = canvas.save();
        if let Some(damage) = damage {
            canvas.clip_rect(damage, None, None);
        }
        scene.with_arena(|arena| {
            scene.with_renderable_arena(|renderable_arena| {
                if let Some(root) = arena.get(root_id.into()) {
                    set_node_transform(root.get(), canvas);
                    let occluded = scene.occluded_set(root_id);
                    render_node_tree(
                        root_id,
                        arena,
                        renderable_arena,
                        canvas,
                        1.0,
                        occluded.as_ref(),
                        None,
                        None,
                    );
                }
            });
        });
        canvas.restore_to_count(save_point);
    }
}

impl std::fmt::Debug for SkiaVkRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkiaVkRenderer").finish()
    }
}
