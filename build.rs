#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "Gank");
    res.set("FileDescription", "Gank - Game Hub");
    if let Err(e) = res.compile() {
        eprintln!("Warning: failed to compile Windows icon resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
