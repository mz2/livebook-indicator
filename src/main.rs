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

use url::Url;
mod livebook;

fn create_indicator() -> AppIndicator {
    let mut indicator = AppIndicator::new("Livebook", "");
    indicator.set_status(AppIndicatorStatus::Active);

    // because the app and the icon are not bundled together in any particular way,
    // let's try finding icon from some known expected locations...
    // $SNAP/assets/livebook-icon.png
    // $CARGO_MANIFEST_DIR/assets/livebook-icon.png (i.e. root of source repo when doing `cargo run`)
    let icon_path = match env::var("SNAP") {
        Ok(snap_root) => Path::new(&snap_root).join("assets"),
        Err(_) => Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"),
    };

    indicator.set_icon_theme_path(icon_path.to_str().unwrap());
    indicator.set_icon_full("livebook-icon", "icon");

    return indicator;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // dbg!(args);

    let url_to_open = args
        .iter()
        .map(|arg| Url::parse(arg))
        .find(|url| match url {
            Ok(url) => url.scheme() == "livebook",
            Err(_) => false,
        });

    gtk::init().unwrap();

    // livebook server path different depending on whether inside or outside confinement.
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
                            eprintln!("Failed to find livebook server URL");
                            server_process
                                .terminate()
                                .expect("Failed to terminate livebook server");
                            process::exit(-1);
                        }
                        Some(url_capture) => {
                            let url = &url_capture[1];

                            match Url::parse(url) {
                                Ok(parsed_url) => {
                                    println!("Parsed URL:");
                                    dbg!(parsed_url);
                                }
                                Err(_) => {
                                    server_process
                    .terminate()
                    .expect("Failed to parse Livebook URL from server process stdout -> exiting.");
                                    process::exit(-1);
                                }
                            };

                            livebook::open::open_url(url);

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
                                        Err(_) => eprintln!("Failed to kill livebook server"),
                                        Ok(_) => process::exit(0),
                                    })
                                    .expect("Error setting Ctrl-C handler");

                                    // finally, set up the app indicator.
                                    let mut indicator = create_indicator();
                                    let mut menu =
                                        livebook::menu::create_menu(url.to_string(), server_pid);
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
                    eprintln!("livebook server stdout unexpectedly not readable.");
                    server_process
                        .terminate()
                        .expect("Failed to terminate livebook server process");
                    process::exit(-1);
                }
            }
        }
    }
}
