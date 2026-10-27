use crate::models::{Game, Platform};
use crate::scanners::epic::EpicScanner;
use crate::scanners::steam::SteamScanner;
use crate::scanners::scan_all_platforms;
use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(target_os = "windows")]
pub fn open_uri(uri: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wide_uri: Vec<u16> = OsStr::new(uri)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let wide_open: Vec<u16> = OsStr::new("open")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: *mut std::ffi::c_void,
            lpOperation: *const u16,
            lpFile: *const u16,
            lpParameters: *const u16,
            lpDirectory: *const u16,
            nShowCmd: i32,
        ) -> isize;
    }

    let ret = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            wide_open.as_ptr(),
            wide_uri.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };

    ret > 32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Alphabetical,
    Size,
    RecentlyPlayed,
}

impl SortOrder {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Alphabetical => "🔤 Nome (A-Z)",
            Self::Size => "💾 Maior Tamanho",
            Self::RecentlyPlayed => "🕒 Recentes",
        }
    }
}

pub struct KiroApp {
    games: Vec<Game>,
    search_query: String,
    selected_platform: Option<Platform>,
    sort_order: SortOrder,
    status_message: Option<String>,
    status_tx: Sender<String>,
    status_rx: Receiver<String>,
}

impl Default for KiroApp {
    fn default() -> Self {
        let (status_tx, status_rx) = channel();
        let games = scan_all_platforms();
        Self {
            games,
            search_query: String::new(),
            selected_platform: None,
            sort_order: SortOrder::Alphabetical,
            status_message: None,
            status_tx,
            status_rx,
        }
    }
}

impl KiroApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn calculate_total_storage(&self) -> String {
        let total_bytes: u64 = self.games.iter().filter_map(|g| g.install_size).sum();
        const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
        let gib = total_bytes as f64 / GIB;
        format!("{:.1} GB", gib)
    }

    fn launch_game(&mut self, game: &Game) {
        #[cfg(target_os = "windows")]
        {
            // If Epic Games is selected but the launcher is not yet running in background,
            // start EpicGamesLauncher in silent mode first and dispatch the game after initialization.
            if game.platform == Platform::EpicGames && !EpicScanner::is_launcher_running() {
                if let Some(launcher_exe) = EpicScanner::find_epic_launcher_exe() {
                    let tx = self.status_tx.clone();
                    let title = game.title.clone();
                    let uri = game.launch_uri.clone();

                    self.status_message = Some("Iniciando Epic Games em segundo plano...".to_string());

                    std::thread::spawn(move || {
                        let mut cmd = std::process::Command::new(&launcher_exe);
                        cmd.arg("-silent");
                        if let Some(parent) = launcher_exe.parent() {
                            cmd.current_dir(parent);
                        }
                        let _ = cmd.spawn();

                        // Give Epic launcher time to initialize its IPC listener (~2.5s)
                        std::thread::sleep(std::time::Duration::from_millis(2500));

                        let _ = tx.send(format!("Iniciando: {}", title));
                        open_uri(&uri);
                    });
                    return;
                }
            }

            // If Steam is selected but not yet running,
            // start steam.exe in silent mode first (minimized to system tray) and dispatch the game.
            if game.platform == Platform::Steam && !SteamScanner::is_running() {
                if let Some(steam_exe) = SteamScanner::find_steam_exe() {
                    let tx = self.status_tx.clone();
                    let title = game.title.clone();
                    let uri = game.launch_uri.clone();

                    self.status_message = Some("Iniciando Steam em segundo plano...".to_string());

                    std::thread::spawn(move || {
                        let mut cmd = std::process::Command::new(&steam_exe);
                        cmd.arg("-silent");
                        if let Some(parent) = steam_exe.parent() {
                            cmd.current_dir(parent);
                        }
                        let _ = cmd.spawn();

                        // Give Steam time to initialize its background service (~2.0s)
                        std::thread::sleep(std::time::Duration::from_millis(2000));

                        let _ = tx.send(format!("Iniciando: {}", title));
                        open_uri(&uri);
                    });
                    return;
                }
            }

            // Direct protocol launch (for already running launchers)
            if open_uri(&game.launch_uri) {
                self.status_message = Some(format!("Iniciando: {}", game.title));
            } else {
                self.status_message = Some(format!("Erro ao iniciar: {}", game.title));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = &game.launch_uri;
            self.status_message = Some("Execução disponível no Windows".to_string());
        }
    }

    /// Generates a pleasant distinctive accent color based on title hash for games without cover art.
    fn title_accent_color(title: &str) -> egui::Color32 {
        let mut hash: u32 = 5381;
        for b in title.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u32);
        }
        let colors = [
            egui::Color32::from_rgb(34, 52, 85),
            egui::Color32::from_rgb(60, 36, 80),
            egui::Color32::from_rgb(26, 65, 60),
            egui::Color32::from_rgb(85, 45, 30),
            egui::Color32::from_rgb(45, 45, 65),
            egui::Color32::from_rgb(20, 55, 75),
        ];
        colors[(hash as usize) % colors.len()]
    }
}

impl eframe::App for KiroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process asynchronous status updates
        while let Ok(msg) = self.status_rx.try_recv() {
            self.status_message = Some(msg);
            ctx.request_repaint();
        }

        // Top Control Panel
        egui::TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                    .fill(egui::Color32::from_rgb(18, 20, 26)),
            )
            .show(ctx, |ui| {
                // Brand Header Row
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new("⚡ KIRO")
                            .color(egui::Color32::from_rgb(255, 120, 50))
                            .strong()
                            .size(20.0),
                    );
                    ui.label(
                        egui::RichText::new("Game Hub")
                            .color(egui::Color32::from_rgb(160, 165, 175))
                            .size(14.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                egui::RichText::new("🔄 Atualizar")
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .clicked()
                        {
                            self.games = scan_all_platforms();
                            self.status_message = Some("Lista de jogos atualizada!".to_string());
                        }

                        let total_storage = self.calculate_total_storage();
                        let total_text = format!(
                            "{} jogos instalados • {}",
                            self.games.len(),
                            total_storage
                        );
                        ui.label(
                            egui::RichText::new(total_text)
                                .color(egui::Color32::from_rgb(140, 145, 155))
                                .small(),
                        );
                    });
                });

                ui.add_space(10.0);

                // Search & Filter Toolbar
                ui.horizontal(|ui| {
                    // Search Input with Icon
                    ui.label(egui::RichText::new("🔍").size(15.0));
                    let search_width = (ui.available_width() * 0.40).clamp(160.0, 320.0);
                    let search_input = egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Buscar jogo...")
                        .desired_width(search_width);
                    ui.add(search_input);

                    if !self.search_query.is_empty() {
                        if ui.small_button("✖").clicked() {
                            self.search_query.clear();
                        }
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Platform Filter Chips
                    let steam_count = self.games.iter().filter(|g| g.platform == Platform::Steam).count();
                    let epic_count = self.games.iter().filter(|g| g.platform == Platform::EpicGames).count();

                    let all_selected = self.selected_platform.is_none();
                    if ui.selectable_label(all_selected, format!("Todos ({})", self.games.len())).clicked() {
                        self.selected_platform = None;
                    }

                    let steam_selected = self.selected_platform == Some(Platform::Steam);
                    if ui.selectable_label(steam_selected, format!("Steam ({})", steam_count)).clicked() {
                        self.selected_platform = if steam_selected { None } else { Some(Platform::Steam) };
                    }

                    let epic_selected = self.selected_platform == Some(Platform::EpicGames);
                    if ui.selectable_label(epic_selected, format!("Epic Games ({})", epic_count)).clicked() {
                        self.selected_platform = if epic_selected { None } else { Some(Platform::EpicGames) };
                    }

                    // Sort Order selector on the far right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::ComboBox::from_id_source("sort_order_combo")
                            .selected_text(self.sort_order.label())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.sort_order, SortOrder::Alphabetical, SortOrder::Alphabetical.label());
                                ui.selectable_value(&mut self.sort_order, SortOrder::Size, SortOrder::Size.label());
                                ui.selectable_value(&mut self.sort_order, SortOrder::RecentlyPlayed, SortOrder::RecentlyPlayed.label());
                            });
                        ui.label(egui::RichText::new("Ordenar:").small().color(egui::Color32::from_rgb(140, 145, 155)));
                    });
                });

                if let Some(msg) = &self.status_message {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(msg)
                                .color(egui::Color32::from_rgb(120, 220, 140))
                                .small(),
                        );
                    });
                }
            });

        // Main Responsive Games Grid
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(13, 14, 18))
                    .inner_margin(egui::Margin::symmetric(20.0, 16.0)),
            )
            .show(ctx, |ui| {
                let query = self.search_query.trim().to_lowercase();
                let mut filtered_games: Vec<&Game> = self
                    .games
                    .iter()
                    .filter(|game| {
                        if !query.is_empty() && !game.title.to_lowercase().contains(&query) {
                            return false;
                        }
                        if let Some(platform) = &self.selected_platform {
                            if &game.platform != platform {
                                return false;
                            }
                        }
                        true
                    })
                    .collect();

                // Sort games according to selected criteria
                match self.sort_order {
                    SortOrder::Alphabetical => {
                        filtered_games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
                    }
                    SortOrder::Size => {
                        filtered_games.sort_by(|a, b| b.install_size.unwrap_or(0).cmp(&a.install_size.unwrap_or(0)));
                    }
                    SortOrder::RecentlyPlayed => {
                        filtered_games.sort_by(|a, b| b.last_played.unwrap_or(0).cmp(&a.last_played.unwrap_or(0)));
                    }
                }

                if filtered_games.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(egui::RichText::new("🎮").size(40.0));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Nenhum jogo encontrado com esse filtro.")
                                .color(egui::Color32::from_rgb(150, 155, 165))
                                .size(16.0),
                        );
                    });
                    return;
                }

                let mut game_to_launch: Option<Game> = None;

                // Responsive Grid metrics
                let available_w = ui.available_width();
                let card_w = 186.0;
                let poster_h = 248.0;
                let spacing = 16.0;
                let cols = ((available_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                        for row_chunk in filtered_games.chunks(cols) {
                            ui.horizontal(|ui| {
                                for game in row_chunk {
                                    let mut clicked_card = false;

                                    let card_response = egui::Frame::group(ui.style())
                                        .fill(egui::Color32::from_rgb(22, 24, 31))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(38, 42, 54)))
                                        .rounding(10.0)
                                        .inner_margin(egui::Margin::same(10.0))
                                        .show(ui, |ui| {
                                            ui.set_width(card_w);
                                            ui.vertical(|ui| {
                                                // --- POSTER / COVER AREA ---
                                                let (poster_rect, poster_resp) = ui.allocate_exact_size(
                                                    egui::vec2(card_w, poster_h),
                                                    egui::Sense::click(),
                                                );

                                                if poster_resp.clicked() {
                                                    clicked_card = true;
                                                }

                                                // If real cover exists, render it; otherwise draw stylized placeholder
                                                if let Some(cover_path) = &game.cover_image_path {
                                                    let uri = format!("file://{}", cover_path.to_string_lossy().replace('\\', "/"));
                                                    let img = egui::Image::new(uri)
                                                        .fit_to_exact_size(egui::vec2(card_w, poster_h))
                                                        .rounding(egui::Rounding::same(8.0));
                                                    img.paint_at(ui, poster_rect);
                                                } else {
                                                    // Beautiful stylized gradient card
                                                    let bg_color = Self::title_accent_color(&game.title);
                                                    ui.painter().rect_filled(
                                                        poster_rect,
                                                        egui::Rounding::same(8.0),
                                                        bg_color,
                                                    );

                                                    // Stylized game initials / title watermark
                                                    let initials: String = game.title
                                                        .split_whitespace()
                                                        .filter_map(|w| w.chars().next())
                                                        .take(3)
                                                        .collect();
                                                    ui.painter().text(
                                                        poster_rect.center() - egui::vec2(0.0, 16.0),
                                                        egui::Align2::CENTER_CENTER,
                                                        initials.to_uppercase(),
                                                        egui::FontId::proportional(36.0),
                                                        egui::Color32::from_white_alpha(50),
                                                    );

                                                    // Centered game title text
                                                    let display_title = if game.title.len() > 36 {
                                                        format!("{}...", &game.title[..33])
                                                    } else {
                                                        game.title.clone()
                                                    };
                                                    ui.painter().text(
                                                        poster_rect.center() + egui::vec2(0.0, 20.0),
                                                        egui::Align2::CENTER_CENTER,
                                                        display_title,
                                                        egui::FontId::proportional(14.0),
                                                        egui::Color32::from_white_alpha(220),
                                                    );
                                                }

                                                // Platform Badge Overlay (Top Left)
                                                let (badge_bg, badge_text) = match game.platform {
                                                    Platform::Steam => (egui::Color32::from_rgb(22, 60, 110), "STEAM"),
                                                    Platform::EpicGames => (egui::Color32::from_rgb(45, 45, 55), "EPIC"),
                                                    Platform::Gog => (egui::Color32::from_rgb(90, 30, 120), "GOG"),
                                                    Platform::Xbox => (egui::Color32::from_rgb(14, 110, 50), "XBOX"),
                                                    Platform::Custom => (egui::Color32::from_rgb(70, 70, 70), "LOCAL"),
                                                };

                                                let badge_rect = egui::Rect::from_min_size(
                                                    poster_rect.min + egui::vec2(8.0, 8.0),
                                                    egui::vec2(52.0, 18.0),
                                                );
                                                ui.painter().rect_filled(
                                                    badge_rect,
                                                    egui::Rounding::same(4.0),
                                                    badge_bg,
                                                );
                                                ui.painter().text(
                                                    badge_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    badge_text,
                                                    egui::FontId::proportional(10.0),
                                                    egui::Color32::WHITE,
                                                );

                                                // Size Badge Overlay (Bottom Right of poster)
                                                if let Some(size_str) = game.formatted_size() {
                                                    let size_rect = egui::Rect::from_min_size(
                                                        poster_rect.max - egui::vec2(66.0, 24.0),
                                                        egui::vec2(58.0, 18.0),
                                                    );
                                                    ui.painter().rect_filled(
                                                        size_rect,
                                                        egui::Rounding::same(4.0),
                                                        egui::Color32::from_black_alpha(190),
                                                    );
                                                    ui.painter().text(
                                                        size_rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        size_str,
                                                        egui::FontId::proportional(10.0),
                                                        egui::Color32::from_rgb(230, 230, 230),
                                                    );
                                                }

                                                ui.add_space(8.0);

                                                // --- CARD FOOTER & TITLE ---
                                                let title_label = egui::Label::new(
                                                    egui::RichText::new(&game.title)
                                                        .color(egui::Color32::WHITE)
                                                        .strong()
                                                        .size(13.5),
                                                )
                                                .truncate(true);
                                                ui.add(title_label);

                                                ui.add_space(2.0);

                                                // Last Played / Extra Info Row
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(game.formatted_last_played())
                                                            .color(egui::Color32::from_rgb(140, 145, 155))
                                                            .size(11.0),
                                                    );

                                                    if let Some(ver) = &game.version {
                                                        let short_ver = if ver.len() > 10 {
                                                            &ver[..10]
                                                        } else {
                                                            ver.as_str()
                                                        };
                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                            ui.label(
                                                                egui::RichText::new(short_ver)
                                                                    .color(egui::Color32::from_rgb(110, 115, 125))
                                                                    .size(10.0),
                                                            );
                                                        });
                                                    }
                                                });

                                                ui.add_space(8.0);

                                                // Play Button
                                                let play_btn = egui::Button::new(
                                                    egui::RichText::new("▶ JOGAR")
                                                        .color(egui::Color32::WHITE)
                                                        .strong()
                                                        .size(12.5),
                                                )
                                                .fill(egui::Color32::from_rgb(34, 160, 68))
                                                .rounding(6.0)
                                                .min_size(egui::vec2(card_w, 28.0));

                                                if ui.add(play_btn).clicked() {
                                                    clicked_card = true;
                                                }
                                            });
                                        });

                                    // Hover Tooltip with Full Game Details
                                    card_response.response.on_hover_ui(|ui| {
                                        ui.heading(egui::RichText::new(&game.title).size(15.0));
                                        ui.add_space(4.0);
                                        ui.label(format!("Plataforma: {}", game.platform.display_name()));
                                        if let Some(size) = game.formatted_size() {
                                            ui.label(format!("Espaço em disco: {}", size));
                                        }
                                        ui.label(format!("Status: {}", game.formatted_last_played()));
                                        if let Some(dir) = &game.install_dir {
                                            ui.label(format!("Pasta: {}", dir.display()));
                                        }
                                        if let Some(exe) = &game.executable_path {
                                            ui.label(format!("Executável: {}", exe.display()));
                                        }
                                        if let Some(ver) = &game.version {
                                            ui.label(format!("Versão: {}", ver));
                                        }
                                    });

                                    if clicked_card {
                                        game_to_launch = Some((*game).clone());
                                    }
                                }
                            });
                        }
                    });

                if let Some(game) = game_to_launch {
                    self.launch_game(&game);
                }
            });
    }
}
