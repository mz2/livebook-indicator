use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process;
use subprocess::{Popen, PopenConfig};

use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};

use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;

use ctrlc;

fn xdg_open(url: &str) {
  Command::new("xdg-open")
    .args([url])
    .output()
    .expect("Failed to open browser");
}

fn create_menu() -> gtk::Menu {
  let menu = gtk::Menu::new();
  let open_browser_item = gtk::CheckMenuItem::with_label("Open Browser");
  open_browser_item.connect_activate(|_| {
    xdg_open("https://127.0.0.1:8082");
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

  let server = Popen::create(
    &["livebook"],
    PopenConfig {
      stdout: subprocess::Redirection::Pipe,
      ..Default::default()
    },
  );

  match server {
    Err(_) => process::exit(-1),
    Ok(mut server_process) => {
      let server_stdout = &server_process.stdout;
      match &server_stdout {
        Some(f) => {
          let mut reader = BufReader::new(f);
          let mut line = String::new();
          reader
            .read_line(&mut line)
            .expect("Failed to read from livebook server stdout");

          lazy_static! {
            static ref APP_URL_PATTERN: Regex = Regex::new(r"(http.*)").unwrap();
          }

          // hack hack.
          match &APP_URL_PATTERN.captures(&line) {
            None => {
              println!("Failed to find livebook server URL");
              server_process
                .terminate()
                .expect("Failed to terminate livebook server");
              process::exit(-1);
            }
            Some(url_capture) => {
              xdg_open(&url_capture[1]);
            }
          }
        }
        None => {
          println!("livebook server stdout unexpectedly not readable.");
          server_process
            .terminate()
            .expect("Failed to terminate livebook server process");
          process::exit(-1);
        }
      }

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
