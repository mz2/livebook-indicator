use std::env;
use std::path::Path;
use std::process;
use subprocess::{Popen, PopenConfig};

use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};

use signal_hook::flag;
use std::process::Command;
use std::sync::Arc;

use ctrlc;

fn create_menu() -> gtk::Menu {
  let menu = gtk::Menu::new();
  let open_browser_item = gtk::CheckMenuItem::with_label("Open Browser");
  open_browser_item.connect_activate(|_| {
    Command::new("xdg-open")
      .args(["https://127.0.0.1:8082"])
      .output()
      .expect("Failed to open browser");
  });
  let quit_item = gtk::CheckMenuItem::with_label("Quit");
  quit_item.connect_activate(|_| {
    gtk::main_quit();
  });

  menu.append(&open_browser_item);
  menu.append(&quit_item);

  return menu;
}

fn create_indicator() -> AppIndicator {
  let mut indicator = AppIndicator::new("libappindicator test application", "");
  indicator.set_status(AppIndicatorStatus::Active);
  let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
  indicator.set_icon_theme_path(icon_path.to_str().unwrap());
  indicator.set_icon_full("livebook-icon", "icon");

  return indicator;
}

fn main() {
  gtk::init().unwrap();

  let mut server = Popen::create(
    &["livebook"],
    PopenConfig {
      ..Default::default()
    },
  );

  match server {
    Err(_) => process::exit(-1),
    Ok(mut server_process) => {
      let mut communicator = server_process.communicate_start(None);

      match server_process.poll() {
        Some(exit_status) => {
          println!(
            "Failed to start livebook server (exited successfully: {:?}) (is it running already?)",
            exit_status.success()
          );
          process::exit(-1);
        }
        None => {
          println!("Started livebook server");

          ctrlc::set_handler(move || match server_process.terminate() {
            Err(_) => println!("Failed to kill livebook server"),
            Ok(_) => process::exit(0),
          })
          .expect("Error setting Ctrl-C handler");

          let mut indicator = create_indicator();
          let mut menu = create_menu();
          indicator.set_menu(&mut menu);
          menu.show_all();
          gtk::main();
        }
      }
    }
  }
}
