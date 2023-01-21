use gtk::prelude::*;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

// passing a PID to avoid having to mess with borrowed, shared mutable state (the server process handle).
pub fn create_menu(livebook_url: String, livebook_pid: u32) -> gtk::Menu {
    let menu = gtk::Menu::new();
    let open_browser_item = gtk::MenuItem::with_label("Open Browser");
    open_browser_item.connect_activate(move |_| super::open::open_url(&livebook_url));
    let quit_item = gtk::MenuItem::with_label("Quit");
    quit_item.connect_activate(move |_| match i32::try_from(livebook_pid) {
        Ok(livebook_pid) => {
            signal::kill(Pid::from_raw(livebook_pid), Signal::SIGKILL).expect(&format!(
                "Failed to SIGTERM kill livebook server with pid {}",
                livebook_pid,
            ));
            gtk::main_quit();
        }
        _ => {
            eprintln!(
                "Unexpected PID for livebook server (could not convert) -> could not send SIGTERM."
            );
            gtk::main_quit();
        }
    });

    menu.append(&open_browser_item);
    menu.append(&quit_item);

    return menu;
}
