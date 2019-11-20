use std::cell::RefCell;
use wayland_client::protocol::wl_display::WlDisplay;
use wayland_client::protocol::{wl_compositor, wl_surface};
use wayland_client::Main;
use wayland_client::{Attached, Display, EventQueue, GlobalManager};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_shell_v1::{
    Layer, ZwlrLayerShellV1,
};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_surface_v1::{
    Anchor, Event, ZwlrLayerSurfaceV1,
};

pub struct Window {
    pub display: Display,
    pub events: RefCell<EventQueue>,
    pub attached_display: Attached<WlDisplay>,
    pub globals: GlobalManager,
    pub surface: Main<wl_surface::WlSurface>,
    pub layer: Main<ZwlrLayerShellV1>,
    pub layer_surface: Main<ZwlrLayerSurfaceV1>,
}
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

#[derive(Debug)]
pub enum WindowError {
    Todo(Box<dyn std::error::Error>),
}

impl From<wayland_client::ConnectError> for WindowError {
    fn from(e: wayland_client::ConnectError) -> Self {
        Self::Todo(Box::new(e))
    }
}

impl Window {
    pub fn spawn(
        width: u32,
        height: u32,
        offset_top: i32,
        offset_left: i32,
    ) -> Result<Window, WindowError> {
        let display = Display::connect_to_env()?;

        let mut event_queue = display.create_event_queue();
        let attached_display = (*display).clone().attach(event_queue.get_token());

        let globals = GlobalManager::new(&attached_display);
        event_queue.sync_roundtrip(|_, _| unreachable!()).unwrap();

        let compositor = globals
            .instantiate_exact::<wl_compositor::WlCompositor>(1)
            .unwrap();
        let surface = compositor.create_surface();

        let layer = globals.instantiate_exact::<ZwlrLayerShellV1>(1).unwrap();
        let layer_surface =
            layer.get_layer_surface(&surface, None, Layer::Top, String::from("infolauncher"));
        layer_surface.set_size(width, height);
        layer_surface.set_anchor(Anchor::Top);
        layer_surface.set_margin(offset_top, 0, 0, offset_left);
        layer_surface.set_keyboard_interactivity(1);
        layer_surface.assign_mono(move |layer_surface, event| match event {
            Event::Configure {
                width: given_width, height: given_height, serial
            } => {
                println!("Configuring layer surface with {}:{} -> {}", width, height, serial);
                if (given_width, given_height) != (width, height) {
                    panic!("I was prevented by the compositor from using the appropriate window width and height");
                }
                layer_surface.ack_configure(serial);
            }
            Event::Closed => {eprintln!("Got close event!"); *super::STATUS.lock().unwrap() = super::Status::Closing},
            _ => println!("Unhandled event"),
        });
        let _eventsn = event_queue.sync_roundtrip(|_, _| {}).unwrap();
        eprintln!("Syncing after Configure event");
        surface.commit();
        let _eventsn = event_queue.sync_roundtrip(|_, _| {}).unwrap();

        Ok(Self {
            display,
            events: RefCell::new(event_queue),
            attached_display,
            globals,
            surface,
            layer,
            layer_surface,
        })
    }
}
