use std::thread;
use std::time::Duration;

mod server;
mod gui;

fn main() {
    // Start the server in a separate thread
    thread::spawn(|| {
        server::run_server();
    });

    // Give the server a moment to start up
    thread::sleep(Duration::from_millis(500));

    // Launch the GUI in the main thread
    if let Err(e) = gui::run_gui() {
        eprintln!("GUI error: {}", e);
    }
}