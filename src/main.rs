#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod artwork;
pub mod config;
pub mod metadata;
mod models;
mod scanners;
mod ui;

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

#[cfg(windows)]
mod single_instance {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    const ERROR_ALREADY_EXISTS: u32 = 183;
    const SW_RESTORE: i32 = 9;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateMutexW(
            lp_mutex_attributes: *mut std::ffi::c_void,
            b_initial_owner: i32,
            lp_name: *const u16,
        ) -> *mut std::ffi::c_void;
        fn GetLastError() -> u32;
        fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(
            lp_class_name: *const u16,
            lp_window_name: *const u16,
        ) -> *mut std::ffi::c_void;
        fn IsIconic(h_wnd: *mut std::ffi::c_void) -> i32;
        fn ShowWindow(h_wnd: *mut std::ffi::c_void, n_cmd_show: i32) -> i32;
        fn SetForegroundWindow(h_wnd: *mut std::ffi::c_void) -> i32;
        fn BringWindowToTop(h_wnd: *mut std::ffi::c_void) -> i32;
    }

    pub struct SingleInstanceGuard {
        handle: *mut std::ffi::c_void,
    }

    impl Drop for SingleInstanceGuard {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                unsafe {
                    CloseHandle(self.handle);
                }
            }
        }
    }

    pub enum Status {
        Acquired(SingleInstanceGuard),
        AlreadyRunning,
    }

    /// Tries to acquire a system-wide named mutex for Gank.
    /// If an instance is already active, restores & focuses its existing window and returns `Status::AlreadyRunning`.
    pub fn acquire(mutex_name: &str, window_title: &str) -> Status {
        let name_wide: Vec<u16> = OsStr::new(mutex_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let handle = CreateMutexW(std::ptr::null_mut(), 1, name_wide.as_ptr());
            if handle.is_null() {
                // If creating mutex fails for unexpected OS permissions reasons, allow launch
                return Status::Acquired(SingleInstanceGuard {
                    handle: std::ptr::null_mut(),
                });
            }

            if GetLastError() == ERROR_ALREADY_EXISTS {
                // Another instance is already running: bring its existing window to the foreground
                let title_wide: Vec<u16> = OsStr::new(window_title)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                let hwnd = FindWindowW(std::ptr::null(), title_wide.as_ptr());
                if !hwnd.is_null() {
                    if IsIconic(hwnd) != 0 {
                        ShowWindow(hwnd, SW_RESTORE);
                    }
                    BringWindowToTop(hwnd);
                    SetForegroundWindow(hwnd);
                }

                CloseHandle(handle);
                return Status::AlreadyRunning;
            }

            Status::Acquired(SingleInstanceGuard { handle })
        }
    }
}

fn main() -> eframe::Result<()> {
    #[cfg(windows)]
    init_windows_app_id();

    #[cfg(windows)]
    let _single_instance_guard = match single_instance::acquire(
        "Local\\Gank_GameHub_SingleInstance_Mutex",
        "Gank - Game Hub",
    ) {
        single_instance::Status::AlreadyRunning => {
            return Ok(());
        }
        single_instance::Status::Acquired(guard) => Some(guard),
    };

    let wgpu_options = eframe::egui_wgpu::WgpuConfiguration {
        present_mode: eframe::wgpu::PresentMode::Fifo,
        power_preference: eframe::wgpu::PowerPreference::LowPower,
        supported_backends: eframe::wgpu::Backends::DX12 | eframe::wgpu::Backends::VULKAN,
        on_surface_error: std::sync::Arc::new(|err| {
            eprintln!("Wgpu surface error: {:?}", err);
            eframe::egui_wgpu::SurfaceErrorAction::RecreateSurface
        }),
        ..Default::default()
    };

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
