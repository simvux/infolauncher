use super::window::Window;
use std::marker::PhantomData;
use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::image;
use vulkano::instance::{Instance, InstanceCreationError, InstanceExtensions, PhysicalDevice};
use vulkano::swapchain::{PresentMode, Surface, SurfaceTransform, Swapchain};

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

pub struct VkSession {
    pub device: Arc<Device>,
    pub instance: Arc<Instance>,
    pub queue: Arc<Queue>,
    pub draw_surface: Arc<Surface<()>>,
    pub swapchain: Arc<Swapchain<()>>,
    pub images: Vec<Arc<image::SwapchainImage<()>>>,
}

#[derive(Debug)]
pub enum VkSessionError {
    PhysicalDeviceIdNotFound(usize),
    Todo(Box<dyn std::error::Error>),
}
impl From<InstanceCreationError> for VkSessionError {
    fn from(e: InstanceCreationError) -> Self {
        Self::Todo(Box::new(e))
    }
}

impl<'a> VkSession {
    pub fn initialize(
        physical_device_id: usize,
        window: Window,
    ) -> Result<(Self, Window), VkSessionError> {
        let extensions = InstanceExtensions {
            khr_wayland_surface: true,
            khr_surface: true,
            ..InstanceExtensions::none()
        };

        let instance = Instance::new(None, &extensions, None)?;
        let physical = PhysicalDevice::from_index(&instance, physical_device_id)
            .ok_or_else(|| VkSessionError::PhysicalDeviceIdNotFound(physical_device_id))?;

        let queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics())
            .expect("Could not find a graphical queue family of physical vulkan device");

        let (device, mut queues) = {
            Device::new(
                physical,
                physical.supported_features(),
                &DeviceExtensions {
                    khr_swapchain: true,
                    ..DeviceExtensions::none()
                },
                [(queue_family, 0.5)].iter().cloned(),
            )
            .expect("Failed to create device")
        };
        let queue = queues.next().unwrap();

        let vksurface = unsafe {
            Surface::from_wayland(
                instance.clone(),
                window.display.c_ptr(),
                window.layer_surface.as_ref().c_ptr(),
                (),
            )
            .unwrap()
        };

        let caps = vksurface.capabilities(physical).unwrap();
        let dimensions = caps.current_extent.unwrap_or([WIDTH, HEIGHT]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            vksurface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            caps.supported_usage_flags,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        )
        .unwrap();

        Ok((
            Self {
                device,
                instance,
                queue,
                draw_surface: vksurface,
                swapchain,
                images,
            },
            window,
        ))
    }
}
