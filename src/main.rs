#[macro_use(event_enum)]
extern crate wayland_client;

mod draw;

fn main() {
    // Load config file

    // Parse flags

    // Initialize drawing
    let drawer = draw::Drawer::initialize();
    // drawer.open_window();

    drawer.listen_events();
}
