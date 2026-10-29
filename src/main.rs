#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod scanners;
mod ui;
pub mod artwork;
pub mod config;
pub mod metadata;

use ui::GankApp;

fn load_app_icon() -> Option<eframe::egui::IconData> {
    let png_bytes = include_bytes!("../assets/icon.png");
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        Some(eframe::egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

#[cfg(windows)]
fn init_windows_app_id() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "shell32")]
    extern "system" {
        fn SetCurrentProcessExplicitAppUserModelID(app_id: *const u16) -> i32;
    }

    let app_id: Vec<u16> = OsStr::new("Gank.GameHub.Desktop")
        .encode_wide()
        .chain(Some(0))
        .collect();

    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr());
    }
}



fn main() -> eframe::Result<()> {
    #[cfg(windows)]
    init_windows_app_id();

    let mut wgpu_options = eframe::egui_wgpu::WgpuConfiguration::default();
    wgpu_options.present_mode = eframe::wgpu::PresentMode::Fifo;
    wgpu_options.power_preference = eframe::wgpu::PowerPreference::LowPower;
    wgpu_options.supported_backends = eframe::wgpu::Backends::DX12 | eframe::wgpu::Backends::VULKAN;
    wgpu_options.on_surface_error = std::sync::Arc::new(|err| {
        eprintln!("Wgpu surface error: {:?}", err);
        eframe::egui_wgpu::SurfaceErrorAction::RecreateSurface
    });

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([980.0, 680.0])
        .with_min_inner_size([540.0, 420.0])
        .with_title("Gank - Game Hub");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Wgpu,
        wgpu_options,
        ..Default::default()
    };

    eframe::run_native(
        "Gank - Game Hub",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(GankApp::new(cc))
        }),
    )
}
