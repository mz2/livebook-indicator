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

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

fn xdg_open(url: &str) {
  Command::new("xdg-open")
    .args([url])
    .output()
    .expect("Failed to open browser");
}

// passing a PID to avoid having to mess with borrowed, shared mutable state (the server process handle).
fn create_menu(livebook_url: String, livebook_pid: u32) -> gtk::Menu {
  let menu = gtk::Menu::new();
  let open_browser_item = gtk::CheckMenuItem::with_label("Open Browser");
  open_browser_item.connect_activate(move |_| {
    xdg_open(&livebook_url);
  });
  let quit_item = gtk::CheckMenuItem::with_label("Quit");
  quit_item.connect_activate(move |_| match i32::try_from(livebook_pid) {
    Ok(livebook_pid) => {
      signal::kill(Pid::from_raw(livebook_pid), Signal::SIGTERM).expect(&format!(
        "Failed to SIGTERM kill livebook server with pid {}",
        livebook_pid,
      ));
      gtk::main_quit();
    }
    _ => {
      println!("Unexpected PID for livebook server (could not convert) -> could not send SIGTERM.");
      gtk::main_quit();
    }
  });

  menu.append(&open_browser_item);
  menu.append(&quit_item);

  return menu;
}

fn create_indicator() -> AppIndicator {
  let mut indicator = AppIndicator::new("Livebook", "");
  indicator.set_status(AppIndicatorStatus::Active);
  let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
  indicator.set_icon_theme_path(icon_path.to_str().unwrap());
  indicator.set_icon_full("livebook-icon", "icon");

  return indicator;
}

fn main() {
  gtk::init().unwrap();

  // livebook server path different depending on whether we're confined or not.
  let server_path = match env::var("SNAP") {
    Ok(snap_root) => format!("{}/wrappers/start-livebook.sh", snap_root).to_string(),
    Err(_) => "livebook.server".to_string(),
  };

  // start the livebook server
  let server = Popen::create(
    &[server_path],
    PopenConfig {
      stdout: subprocess::Redirection::Pipe,
      ..Default::default()
    },
  );

  // if the livebook server started successfully, wasn't immediately killed, and its pid was found ->
  // create the app indicator.
  match server {
    Err(_) => {
      panic!("Failed to start livebook server. Is it installed and in $PATH?");
    }
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
              let url = &url_capture[1];
              xdg_open(url);

              match (server_process.poll(), server_process.pid()) {
                (Some(exit_status), _) => {
                  panic!(
                    "Failed to start livebook server (exited successfully: {:?}) (is it running already?)",
                    exit_status.success()
                  );
                }
                (None, Some(server_pid)) => {
                  println!("Started livebook server successfully.");
                  ctrlc::set_handler(move || match server_process.terminate() {
                    Err(_) => println!("Failed to kill livebook server"),
                    Ok(_) => process::exit(0),
                  })
                  .expect("Error setting Ctrl-C handler");

                  // finally, set up the app indicator.
                  let mut indicator = create_indicator();
                  let mut menu = create_menu(url.to_string(), server_pid);
                  indicator.set_menu(&mut menu);
                  menu.show_all();
                  gtk::main();
                }
                _ => {
                  panic!("Unexpected state: server process running but could not get its pid?");
                }
              }
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
    }
  }
}
