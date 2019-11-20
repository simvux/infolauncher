use super::window::Window;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image;
use vulkano::instance::{Instance, InstanceCreationError, InstanceExtensions, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{
    ColorSpace, PresentMode, Surface, SurfaceCreationError, SurfaceTransform, Swapchain,
};

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

pub struct VkSession {
    pub device: Arc<Device>,
    pub instance: Arc<Instance>,
    pub queue: Arc<Queue>,
    pub draw_surface: Arc<Surface<Window>>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub images: Vec<Arc<image::SwapchainImage<Window>>>,
    pub dynamic_state: DynamicState,
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

pub unsafe trait SafeBorrow<T>: Borrow<T> {}

unsafe impl<T> SafeBorrow<T> for T {}
unsafe impl<'a, T> SafeBorrow<T> for &'a T {}
unsafe impl<'a, T> SafeBorrow<T> for &'a mut T {}
unsafe impl<T> SafeBorrow<T> for Rc<T> {}
unsafe impl<T> SafeBorrow<T> for Arc<T> {}
unsafe impl<T> SafeBorrow<T> for Box<T> {}
pub fn create_vk_surface<W>(
    window: W,
    instance: Arc<Instance>,
) -> Result<Arc<Surface<W>>, SurfaceCreationError>
where
    W: SafeBorrow<super::window::Window>,
{
    unsafe {
        Surface::from_wayland(
            instance,
            window.borrow().display.c_ptr() as *mut _,
            window.borrow().surface.as_ref().c_ptr() as *mut _,
            window,
        )
    }
}

impl<'a> VkSession {
    pub fn initialize(physical_device_id: usize, window: Window) -> Result<Self, VkSessionError> {
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

        let vksurface = create_vk_surface(window, instance.clone()).unwrap();

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
            ColorSpace::SrgbNonLinear,
        )
        .unwrap();

        let dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            ..DynamicState::default()
        };

        Ok(Self {
            device,
            instance,
            queue,
            draw_surface: vksurface,
            swapchain,
            images,
            dynamic_state,
        })
    }

    pub fn new_framebuffers(
        &mut self,
    ) -> (
        Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
        Arc<dyn RenderPassAbstract + Send + Sync>,
    ) {
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [500.0, 500.0],
            depth_range: 0.0..1.0,
        };
        self.dynamic_state.viewports = Some(vec![viewport]);
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                self.device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: self.swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );
        let buffers = self
            .images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>();
        (buffers, render_pass)
    }
}
