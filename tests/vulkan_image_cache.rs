//! The GPU image-cache path on Vulkan.
//!
//! An `image_cache`d layer only gets its offscreen surface when the canvas it
//! is drawn into has a GPU context, so this test renders through
//! `renderer::skia_vk` into a `VkImage` created here with `ash`, and checks
//! the result against the same scene drawn by the raster path, which draws the
//! cached layer directly.
//!
//! Run with `cargo test --features vulkan --test vulkan_image_cache`. Where no
//! Vulkan device is available the test prints a note and returns.
#![cfg(feature = "vulkan")]

use std::ffi::{c_void, CStr};

use ash::vk::{self, Handle};
use layers::{
    drawing::node_surfaces_stats,
    prelude::*,
    renderer::skia_vk::{SkiaVkRenderer, VkImageTarget},
    skia::{
        self,
        gpu::{direct_contexts, vk as skvk, DirectContext},
    },
    taffy,
    types::{Color, Size},
};

const W: i32 = 64;
const H: i32 = 64;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

/// Root (transparent) -> an image-cached grey layer holding a translucent red
/// child, plus a translucent blue sibling drawn over both.
fn scene() -> (std::sync::Arc<Engine>, NodeRef) {
    let engine = Engine::create(W as f32, H as f32);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(W as f32, H as f32), None);
    root.set_background_color(Color::new_rgba255(0, 0, 0, 0), None);

    let grey = engine.new_layer();
    engine.append_layer(&grey, Some(root.id)).unwrap();
    grey.set_layout_style(absolute());
    grey.set_position((0.0, 0.0), None);
    grey.set_size(Size::points(W as f32, H as f32), None);
    grey.set_background_color(Color::new_rgba(0.5, 0.5, 0.5, 1.0), None);
    grey.set_image_cached(true);

    let red = engine.new_layer();
    engine.append_layer(&red, Some(grey.id)).unwrap();
    red.set_layout_style(absolute());
    red.set_position((8.0, 8.0), None);
    red.set_size(Size::points(24.0, 24.0), None);
    red.set_background_color(Color::new_rgba(0.85, 0.2, 0.35, 0.6), None);

    let blue = engine.new_layer();
    engine.append_layer(&blue, Some(root.id)).unwrap();
    blue.set_layout_style(absolute());
    blue.set_position((36.0, 4.0), None);
    blue.set_size(Size::points(24.0, 40.0), None);
    blue.set_background_color(Color::new_rgba(0.2, 0.3, 0.9, 0.7), None);

    engine.update(0.016);
    engine.update(0.016);
    (engine, root.id)
}

fn render_into(engine: &std::sync::Arc<Engine>, root: NodeRef, surface: &mut skia::Surface) {
    let canvas = surface.canvas();
    canvas.clear(skia::Color::TRANSPARENT);
    let scene = engine.scene();
    scene.with_arena(|arena| {
        scene.with_renderable_arena(|renderables| {
            render_node_tree(root, arena, renderables, canvas, 1.0, None, None, None);
        });
    });
}

fn rgba8_info(w: i32, h: i32) -> skia::ImageInfo {
    skia::ImageInfo::new(
        (w, h),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Premul,
        None,
    )
}

fn surface_bytes(surface: &mut skia::Surface) -> Vec<u8> {
    let mut px = vec![0u8; (W * H * 4) as usize];
    assert!(surface.read_pixels(&rgba8_info(W, H), &mut px, (W * 4) as usize, (0, 0)));
    px
}

fn image_bytes(image: &skia::Image, context: &mut DirectContext) -> Vec<u8> {
    let (w, h) = (image.width(), image.height());
    let mut px = vec![0u8; (w * h * 4) as usize];
    assert!(image.read_pixels_with_context(
        context,
        &rgba8_info(w, h),
        &mut px,
        (w * 4) as usize,
        (0, 0),
        skia::image::CachingHint::Allow,
    ));
    px
}

/// Largest per-channel difference between two RGBA buffers.
fn max_diff(a: &[u8], b: &[u8]) -> u8 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

/// A headless Vulkan device with a Skia context and one colour image.
struct Vulkan {
    _entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,
    queue_family: u32,
    image: vk::Image,
    memory: vk::DeviceMemory,
    context: DirectContext,
}

impl Vulkan {
    fn new() -> Option<Self> {
        // SAFETY: loading the system Vulkan loader.
        let entry = unsafe { ash::Entry::load() }.ok()?;
        let api_version = vk::API_VERSION_1_1;
        let app = vk::ApplicationInfo::default()
            .application_name(c"lay-rs test")
            .api_version(api_version);
        let create = vk::InstanceCreateInfo::default().application_info(&app);
        // SAFETY: the create info outlives the call.
        let instance = unsafe { entry.create_instance(&create, None) }.ok()?;

        match Self::with_instance(entry, instance.clone(), api_version) {
            Some(vulkan) => Some(vulkan),
            None => {
                // SAFETY: nothing created from the instance is alive.
                unsafe { instance.destroy_instance(None) };
                None
            }
        }
    }

    fn with_instance(entry: ash::Entry, instance: ash::Instance, api_version: u32) -> Option<Self> {
        // SAFETY: plain queries on a live instance.
        let physical = unsafe { instance.enumerate_physical_devices() }.ok()?;
        let (physical, queue_family) = physical
            .into_iter()
            .filter_map(|pd| {
                // SAFETY: `pd` comes from this instance.
                let families = unsafe { instance.get_physical_device_queue_family_properties(pd) };
                let family = families
                    .iter()
                    .position(|f| f.queue_flags.contains(vk::QueueFlags::GRAPHICS))?;
                // SAFETY: as above.
                let props = unsafe { instance.get_physical_device_properties(pd) };
                // A real GPU first, a software rasteriser as a fallback.
                let rank = u32::from(props.device_type == vk::PhysicalDeviceType::CPU);
                Some((rank, pd, family as u32))
            })
            .min_by_key(|(rank, ..)| *rank)
            .map(|(_, pd, family)| (pd, family))?;

        // SAFETY: as above.
        let props = unsafe { instance.get_physical_device_properties(physical) };
        let name = props
            .device_name_as_c_str()
            .map(CStr::to_string_lossy)
            .unwrap_or_default();
        eprintln!("vulkan device: {name}");

        let priorities = [1.0];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&priorities)];
        let device_info = vk::DeviceCreateInfo::default().queue_create_infos(&queue_info);
        // SAFETY: the create info outlives the call.
        let device = unsafe { instance.create_device(physical, &device_info, None) }.ok()?;
        // SAFETY: the device was created with one queue in this family.
        let queue = unsafe { device.get_device_queue(queue_family, 0) };

        let (image, memory) = match Self::color_image(&instance, physical, &device) {
            Some(target) => target,
            None => {
                // SAFETY: nothing created from the device is alive.
                unsafe { device.destroy_device(None) };
                return None;
            }
        };

        let get_instance_proc_addr = entry.static_fn().get_instance_proc_addr;
        let get_device_proc_addr = instance.fp_v1_0().get_device_proc_addr;
        let get_proc = |of: skvk::GetProcOf| -> *const c_void {
            // SAFETY: Skia passes handles it was given and a NUL-terminated name.
            let f = unsafe {
                match of {
                    skvk::GetProcOf::Instance(inst, name) => {
                        get_instance_proc_addr(vk::Instance::from_raw(inst as u64), name)
                    }
                    skvk::GetProcOf::Device(dev, name) => {
                        get_device_proc_addr(vk::Device::from_raw(dev as u64), name)
                    }
                }
            };
            f.map_or(std::ptr::null(), |f| f as *const c_void)
        };
        // SAFETY: all handles are live and belong together.
        let mut backend = unsafe {
            skvk::BackendContext::new(
                instance.handle().as_raw() as _,
                physical.as_raw() as _,
                device.handle().as_raw() as _,
                (queue.as_raw() as _, queue_family as usize),
                &get_proc,
            )
        };
        backend.set_max_api_version(skvk::Version::from(api_version));
        let Some(context) = direct_contexts::make_vulkan(&backend, None) else {
            // SAFETY: nothing else uses the image or the device.
            unsafe {
                device.destroy_image(image, None);
                device.free_memory(memory, None);
                device.destroy_device(None);
            }
            return None;
        };

        Some(Self {
            _entry: entry,
            instance,
            device,
            queue_family,
            image,
            memory,
            context,
        })
    }

    /// A `W`×`H` RGBA8 image in device-local memory that Skia can render into
    /// and read back.
    fn color_image(
        instance: &ash::Instance,
        physical: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Option<(vk::Image, vk::DeviceMemory)> {
        let info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(vk::Extent3D {
                width: W as u32,
                height: H as u32,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
            )
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        // SAFETY: the create info outlives the call.
        let image = unsafe { device.create_image(&info, None) }.ok()?;
        // SAFETY: `image` is live.
        let requirements = unsafe { device.get_image_memory_requirements(image) };
        // SAFETY: `physical` comes from `instance`.
        let memory_props = unsafe { instance.get_physical_device_memory_properties(physical) };
        let memory_type = (0..memory_props.memory_type_count).find(|&i| {
            requirements.memory_type_bits & (1 << i) != 0
                && memory_props.memory_types[i as usize]
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
        });
        let memory = memory_type.and_then(|memory_type| {
            let alloc = vk::MemoryAllocateInfo::default()
                .allocation_size(requirements.size)
                .memory_type_index(memory_type);
            // SAFETY: the allocate info outlives the call.
            let memory = unsafe { device.allocate_memory(&alloc, None) }.ok()?;
            // SAFETY: fresh memory of the required type and size.
            match unsafe { device.bind_image_memory(image, memory, 0) } {
                Ok(()) => Some(memory),
                Err(_) => {
                    // SAFETY: the memory is unbound.
                    unsafe { device.free_memory(memory, None) };
                    None
                }
            }
        });
        match memory {
            Some(memory) => Some((image, memory)),
            None => {
                // SAFETY: the image is unused.
                unsafe { device.destroy_image(image, None) };
                None
            }
        }
    }

    fn target(&self) -> VkImageTarget {
        VkImageTarget {
            image: self.image.as_raw() as _,
            format: skvk::Format::R8G8B8A8_UNORM,
            tiling: skvk::ImageTiling::OPTIMAL,
            layout: skvk::ImageLayout::UNDEFINED,
            queue_family: self.queue_family,
            width: W,
            height: H,
        }
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        // Surfaces cached by lay-rs outlive this test's device; abandoning the
        // context makes Skia release them without touching Vulkan.
        self.context.flush_submit_and_sync_cpu();
        self.context.abandon();
        // SAFETY: the GPU is idle and Skia makes no further Vulkan calls.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_image(self.image, None);
            self.device.free_memory(self.memory, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

/// An `image_cache`d layer drawn into a wrapped `VkImage` allocates its cache
/// surface on the Vulkan context and composites to the same pixels as the
/// raster path; a GPU subtree buffer matches too.
#[test]
fn image_cache_renders_into_wrapped_vk_image() {
    let Some(mut vulkan) = Vulkan::new() else {
        eprintln!("no Vulkan device available; skipping Vulkan image-cache test");
        return;
    };
    let (engine, root) = scene();

    let mut raster = skia::surfaces::raster(&rgba8_info(W, H), None, None).unwrap();
    render_into(&engine, root, &mut raster);
    let expected = surface_bytes(&mut raster);

    // SAFETY: the image is live, owned by `vulkan`, and outlives the renderer.
    let renderer = unsafe {
        SkiaVkRenderer::new(
            &vulkan.context,
            &vulkan.target(),
            skia::ColorType::RGBA8888,
            None,
        )
    }
    .expect("wrap VkImage as a render target");
    let mut surface = renderer.surface();

    let (cached_before, _) = node_surfaces_stats();
    render_into(&engine, root, &mut surface);
    vulkan.context.flush_and_submit();
    let (cached_after, _) = node_surfaces_stats();
    assert!(
        cached_after > cached_before,
        "the image-cached layer must get a GPU cache surface ({cached_before} -> {cached_after})"
    );

    let first = surface_bytes(&mut surface);
    let diff = max_diff(&expected, &first);
    assert!(diff <= 2, "Vulkan render differs from raster by {diff}");

    // Grey outside every child, straight from the cache surface.
    let i = (((H - 4) * W + 4) * 4) as usize;
    assert!(
        (first[i] as i32 - 128).abs() <= 1 && first[i + 3] == 255,
        "grey via cache: {:?}",
        &first[i..i + 4]
    );

    // A second frame reuses the cache surface.
    render_into(&engine, root, &mut surface);
    vulkan.context.flush_and_submit();
    assert_eq!(node_surfaces_stats().0, cached_after);
    assert_eq!(surface_bytes(&mut surface), first);

    // A subtree buffer on the Vulkan context matches the raster frame.
    let buffer = engine
        .render_subtree(root, None, Some(&mut vulkan.context))
        .expect("subtree buffer");
    assert!(buffer.image.is_texture_backed());
    let bytes = image_bytes(&buffer.image, &mut vulkan.context);
    let (bw, bh) = (
        buffer.image.width() as usize,
        buffer.image.height() as usize,
    );
    assert!(bw >= W as usize && bh >= H as usize);
    let (ox, oy) = (buffer.origin.x as i32, buffer.origin.y as i32);
    assert_eq!((ox, oy), (0, 0));
    let mut cropped = Vec::with_capacity(expected.len());
    for row in bytes.chunks(bw * 4).take(H as usize) {
        cropped.extend_from_slice(&row[..(W * 4) as usize]);
    }
    let diff = max_diff(&expected, &cropped);
    assert!(
        diff <= 2,
        "Vulkan subtree buffer differs from raster by {diff}"
    );

    drop(surface);
    drop(renderer);
}
