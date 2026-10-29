#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "Gank");
    res.set("FileDescription", "Gank - Game Hub");
    res.set("LegalCopyright", "Copyright © 2026 Gank Contributors");
    res.set("ProductVersion", "0.1.0.0");
    res.set("FileVersion", "0.1.0.0");
    res.set("OriginalFilename", "Gank.exe");
    if let Err(e) = res.compile() {
        eprintln!("Warning: failed to compile Windows icon resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
