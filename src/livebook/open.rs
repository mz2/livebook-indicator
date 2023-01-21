pub fn open_url(url: &str) {
    println!("Opening '{}'", url);
    match open::that(&url) {
        Ok(()) => (),
        Err(err) => eprintln!("An error occurred when opening '{}': {}", &url, err),
    }
}
