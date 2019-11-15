use wayland_client::protocol::{wl_keyboard, wl_seat};
use wayland_client::Filter;
mod vulkan;
mod window;

pub struct Drawer {
    vk: vulkan::VkSession,
    window: window::Window,
}

impl Drawer {
    pub fn initialize() -> Drawer {
        let window = window::Window::spawn(500, 500, 100, 0).unwrap();

        let physical_device_id = 0;
        let (vk, window) = vulkan::VkSession::initialize(physical_device_id, window).unwrap();

        Self { vk, window }
    }

    pub fn listen_events(mut self) {
        let common_filter = Filter::new(move |event, _| match event {
            Events::Keyboard { event, .. } => match event {
                wl_keyboard::Event::Enter { .. } => println!("Gained keyboard focus"),
                wl_keyboard::Event::Leave { .. } => println!("Lost keyboard focus"),
                wl_keyboard::Event::Key { key, state, .. } => {
                    let state_str = match state {
                        wl_keyboard::KeyState::Pressed => "pressed",
                        wl_keyboard::KeyState::Released => "released",
                        _ => unreachable!(),
                    };
                    println!("Key {} entered state {}", key, state_str);
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

        self.test_draw();
        loop {
            self.window.events.dispatch(|_, _| {}).unwrap();
        }
    }

    fn test_draw(&mut self) {}
}

event_enum!(
    Events | Keyboard => wl_keyboard::WlKeyboard
);
