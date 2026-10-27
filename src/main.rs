#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod scanners;
mod ui;

use ui::KiroApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([760.0, 560.0])
            .with_min_inner_size([480.0, 320.0])
            .with_title("Kiro - Game Hub"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Kiro - Game Hub",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(KiroApp::new(cc))
        }),
    )
}
