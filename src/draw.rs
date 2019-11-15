use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use vulkano::sync::GpuFuture;
use wayland_client::protocol::{wl_keyboard, wl_seat};
use wayland_client::Filter;
mod vulkan;
mod window;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain;

mod shader;

#[derive(PartialEq, Eq)]
pub enum Status {
    Running,
    Closing,
}
lazy_static! {
    pub static ref STATUS: Arc<Mutex<Status>> = Arc::new(Mutex::new(Status::Running));
}

pub struct Drawer {
    // vk: vulkan::VkSession,
    window: window::Window,
}

impl Drawer {
    pub fn initialize() -> Drawer {
        let window = window::Window::spawn(500, 500, 100, 0).unwrap();

        let physical_device_id = 0;
        // let (vk, window) = vulkan::VkSession::initialize(physical_device_id, window).unwrap();

        Self { /* vk,*/ window, }
    }

    pub fn listen_events(mut self) {
        let common_filter = Filter::new(move |event, _| match event {
            Events::Keyboard { event, .. } => match event {
                wl_keyboard::Event::Enter { .. } => println!("Gained keyboard focus"),
                wl_keyboard::Event::Leave { .. } => println!("Lost keyboard focus"),
                wl_keyboard::Event::Key { key, state, .. } => {
                    if key == 1 && state == wl_keyboard::KeyState::Pressed {
                        *STATUS.lock().unwrap() = Status::Closing;
                    }
                }
                _ => (),
            },
        });
        let mut keyboard_created = false;
        self.window
            .globals
            .instantiate_exact::<wl_seat::WlSeat>(1)
            .unwrap()
            .assign_mono(move |seat, event| {
                use wayland_client::protocol::wl_seat::{Capability, Event as SeatEvent};
                if let SeatEvent::Capabilities { capabilities } = event {
                    if !keyboard_created && capabilities.contains(Capability::Keyboard) {
                        keyboard_created = true;
                        seat.get_keyboard().assign(common_filter.clone())
                    }
                }
            });
        self.window
            .events
            .sync_roundtrip(|_, _| { /* we ignore unfiltered messages */ })
            .unwrap();
        eprintln!("Syncing after vulkan and wl_seat configuration");
        // self.test_draw();
        loop {
            if *STATUS.lock().unwrap() == Status::Closing {
                eprintln!("Closing!!!");
                self.window.surface.destroy();
                self.window.events.sync_roundtrip(|_, _| {}).unwrap();
                return;
            }
            eprintln!("Before event dispatch!");
            self.window.events.dispatch(|_, _| {}).unwrap();
            eprintln!("Event dispatch!");
        }
    }

    /*
    fn test_draw(&mut self) {
        let (image_num, acquire_future) =
            swapchain::acquire_next_image(self.vk.swapchain.clone(), None).unwrap();
        let clear = vec![[1.0, 1.0, 0.0, 1.0].into()];

        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                self.vk.device.clone(),
                BufferUsage::all(),
                [
                    shader::Vertex {
                        position: [-0.5, -0.25],
                        color: [0.2, 0.2, 0.2],
                    },
                    shader::Vertex {
                        position: [0.0, 0.5],
                        color: [0.2, 0.2, 0.2],
                    },
                    shader::Vertex {
                        position: [0.25, -0.1],
                        color: [0.2, 0.2, 0.2],
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        };
        let vs = shader::open(
            Path::new("src/draw/shader/test_vert.spv"),
            self.vk.device.clone(),
        );
        let fs = shader::open(
            Path::new("src/draw/shader/test_frag.spv"),
            self.vk.device.clone(),
        );

        let (framebuffers, render_pass) = self.vk.new_framebuffers();
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(shader::get_entry_vertex(&vs), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(shader::get_entry_fragment(&fs), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(self.vk.device.clone())
                .unwrap(),
        );

        let cb = AutoCommandBufferBuilder::primary_one_time_submit(
            self.vk.device.clone(),
            self.vk.queue.family(),
        )
        .unwrap()
        .begin_render_pass(framebuffers[image_num].clone(), false, clear)
        .unwrap()
        .draw(
            pipeline.clone(),
            &self.vk.dynamic_state,
            vertex_buffer.clone(),
            (),
            (),
        )
        .unwrap()
        .end_render_pass()
        .unwrap()
        .build()
        .unwrap();

        let _frame_end = acquire_future
            .then_execute(self.vk.queue.clone(), cb)
            .unwrap()
            .then_swapchain_present(self.vk.queue.clone(), self.vk.swapchain.clone(), image_num);

        /*
        self.window.surface.commit();
        let eventsn = self.window.events.sync_roundtrip(|_, _| {}).unwrap();
        println!("Completed {} wl events", eventsn);
        */
    }
    */
}

event_enum!(
    Events | Keyboard => wl_keyboard::WlKeyboard
);
