#[macro_use]
extern crate lazy_static;
#[macro_use(event_enum)]
extern crate wayland_client;
use std::path::Path;

mod draw;

fn main() {
    assert!(Path::new("src/draw/shader/test_vert.spv").exists());
    assert!(Path::new("src/draw/shader/test_frag.spv").exists());
    // Load config file

    // Parse flags

    // Initialize drawing
    let drawer = draw::Drawer::initialize();
    // drawer.open_window();

    drawer.listen_events();
}
