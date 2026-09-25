//! `BackgroundBlur` on the Vulkan path must match the raster path.
//!
//! Run with `cargo test --features vulkan --test vulkan_blur`. Where no
//! Vulkan device is available the test prints a note and returns.
#![cfg(feature = "vulkan")]

use std::ffi::{c_void, CStr};

use ash::vk::{self, Handle};
use layers::{
    prelude::*,
    renderer::skia_vk::{SkiaVkRenderer, VkImageTarget},
    skia::{
        self,
        gpu::{direct_contexts, vk as skvk, DirectContext},
    },
    taffy,
    types::{BlendMode, Color, Size},
};

const W: i32 = 128;
const H: i32 = 128;

fn absolute() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
        ..Default::default()
    }
}

/// Black/white 4px vertical stripes under a translucent white frosted panel,
/// optionally hosted by an image-cached parent (the dock's menu shape).
fn scene(cached_parent: bool) -> (std::sync::Arc<Engine>, NodeRef) {
    let engine = Engine::create(W as f32, H as f32);

    let root = engine.new_layer();
    engine.add_layer(&root).unwrap();
    root.set_layout_style(absolute());
    root.set_position((0.0, 0.0), None);
    root.set_size(Size::points(W as f32, H as f32), None);
    root.set_background_color(Color::new_rgba(0.0, 0.0, 0.0, 1.0), None);

    for i in 0..(W / 8) {
        let stripe = engine.new_layer();
        engine.append_layer(&stripe, Some(root.id)).unwrap();
        stripe.set_layout_style(absolute());
        stripe.set_position(((i * 8) as f32, 0.0), None);
        stripe.set_size(Size::points(4.0, H as f32), None);
        stripe.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 1.0), None);
    }

    let host = if cached_parent {
        let host = engine.new_layer();
        engine.append_layer(&host, Some(root.id)).unwrap();
        host.set_layout_style(absolute());
        host.set_position((0.0, 0.0), None);
        host.set_size(Size::points(W as f32, H as f32), None);
        host.set_background_color(Color::new_rgba(0.0, 0.0, 0.0, 0.0), None);
        host.set_image_cached(true);
        host
    } else {
        root.clone()
    };

    let panel = engine.new_layer();
    engine.append_layer(&panel, Some(host.id)).unwrap();
    panel.set_layout_style(absolute());
    panel.set_position((16.0, 16.0), None);
    panel.set_size(Size::points(96.0, 96.0), None);
    panel.set_background_color(Color::new_rgba(1.0, 1.0, 1.0, 0.3), None);
    panel.set_blend_mode(BlendMode::BackgroundBlur);

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

fn red(px: &[u8], x: i32, y: i32) -> u8 {
    px[((y * W + x) * 4) as usize]
}

/// Peak-to-peak red across one stripe period in the middle of the panel.
fn ripple(px: &[u8]) -> u8 {
    let y = H / 2;
    let vals: Vec<u8> = (56..72).map(|x| red(px, x, y)).collect();
    vals.iter().max().unwrap() - vals.iter().min().unwrap()
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

/// The frosted panel blurs the stripes on the Vulkan path as the raster path
/// does, on every frame (the second and third replay the kept blur).
fn check(cached_parent: bool) {
    let Some(mut vulkan) = Vulkan::new() else {
        eprintln!("no Vulkan device available; skipping Vulkan blur test");
        return;
    };
    let (engine, root) = scene(cached_parent);

    let mut raster = skia::surfaces::raster(&rgba8_info(W, H), None, None).unwrap();
    render_into(&engine, root, &mut raster);
    let expected = surface_bytes(&mut raster);
    let raster_ripple = ripple(&expected);
    let outside = red(&expected, 4, 4).abs_diff(red(&expected, 8, 4));
    eprintln!("raster: ripple under panel {raster_ripple}, outside {outside}");
    assert!(outside > 200, "stripes are sharp outside the panel");
    assert!(
        raster_ripple < 40,
        "raster blurs the stripes under the panel"
    );

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
    for frame in 0..3 {
        render_into(&engine, root, &mut surface);
        vulkan.context.flush_and_submit();
        let got = surface_bytes(&mut surface);
        let vk_ripple = ripple(&got);
        eprintln!("vulkan frame {frame}: ripple under panel {vk_ripple}");
        assert!(
            vk_ripple < 40,
            "frame {frame}: Vulkan leaves the stripes sharp under the panel (ripple {vk_ripple})"
        );
    }
}

#[test]
fn background_blur_blurs_on_vulkan() {
    check(false);
}

#[test]
fn background_blur_in_cached_parent_blurs_on_vulkan() {
    check(true);
}
