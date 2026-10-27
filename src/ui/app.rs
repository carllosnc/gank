use crate::models::{Game, Platform};
use crate::scanners::epic::EpicScanner;
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

pub struct KiroApp {
    games: Vec<Game>,
    search_query: String,
    selected_platform: Option<Platform>,
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

            // Direct protocol launch (for Steam or when Epic is already running)
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
                    .inner_margin(egui::Margin::same(14.0))
                    .fill(egui::Color32::from_rgb(20, 22, 28)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new("⚡ KIRO")
                            .color(egui::Color32::from_rgb(255, 120, 50))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("Game Hub")
                            .color(egui::Color32::from_rgb(160, 165, 175)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(egui::RichText::new("🔄 Atualizar").color(egui::Color32::WHITE))
                            .clicked()
                        {
                            self.games = scan_all_platforms();
                            self.status_message = Some("Lista de jogos atualizada!".to_string());
                        }

                        let total_text = format!("{} jogos encontrados", self.games.len());
                        ui.label(
                            egui::RichText::new(total_text)
                                .color(egui::Color32::from_rgb(140, 145, 155))
                                .small(),
                        );
                    });
                });

                ui.add_space(10.0);

                // Search Bar
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍").size(16.0));
                    let search_input = egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Buscar jogo...")
                        .desired_width(ui.available_width() - 10.0);
                    ui.add(search_input);
                });

                ui.add_space(8.0);

                // Platform Filter Chips
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Filtrar:").small().color(egui::Color32::from_rgb(140, 145, 155)));

                    let all_selected = self.selected_platform.is_none();
                    if ui.selectable_label(all_selected, "Todos").clicked() {
                        self.selected_platform = None;
                    }

                    let steam_selected = self.selected_platform == Some(Platform::Steam);
                    if ui.selectable_label(steam_selected, "Steam").clicked() {
                        self.selected_platform = if steam_selected { None } else { Some(Platform::Steam) };
                    }

                    let epic_selected = self.selected_platform == Some(Platform::EpicGames);
                    if ui.selectable_label(epic_selected, "Epic Games").clicked() {
                        self.selected_platform = if epic_selected { None } else { Some(Platform::EpicGames) };
                    }
                });

                if let Some(msg) = &self.status_message {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(msg)
                            .color(egui::Color32::from_rgb(100, 200, 120))
                            .small(),
                    );
                }
            });

        // Main Games Grid/List
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(15, 16, 20))
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                let query = self.search_query.trim().to_lowercase();
                let filtered_games: Vec<&Game> = self
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

                if filtered_games.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(
                            egui::RichText::new("Nenhum jogo encontrado.")
                                .color(egui::Color32::from_rgb(140, 140, 150))
                                .size(16.0),
                        );
                    });
                    return;
                }

                let mut game_to_launch: Option<Game> = None;

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

                        for game in filtered_games {
                            let mut clicked_play = false;

                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgb(26, 28, 35))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 48, 58)))
                                .rounding(8.0)
                                .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Platform Badge
                                        let badge_color = match game.platform {
                                            Platform::Steam => egui::Color32::from_rgb(27, 80, 140),
                                            Platform::EpicGames => egui::Color32::from_rgb(50, 50, 50),
                                            Platform::Gog => egui::Color32::from_rgb(110, 40, 140),
                                            Platform::Xbox => egui::Color32::from_rgb(16, 124, 65),
                                            Platform::Custom => egui::Color32::from_rgb(80, 80, 80),
                                        };

                                        egui::Frame::none()
                                            .fill(badge_color)
                                            .rounding(4.0)
                                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(game.platform.display_name())
                                                        .color(egui::Color32::WHITE)
                                                        .small()
                                                        .strong(),
                                                );
                                            });

                                        ui.add_space(8.0);

                                        // Title
                                        ui.label(
                                            egui::RichText::new(&game.title)
                                                .color(egui::Color32::WHITE)
                                                .size(15.0)
                                                .strong(),
                                        );

                                        // Right side Play Button
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let play_btn = egui::Button::new(
                                                    egui::RichText::new("▶ JOGAR")
                                                        .color(egui::Color32::WHITE)
                                                        .strong(),
                                                )
                                                .fill(egui::Color32::from_rgb(40, 150, 70))
                                                .rounding(6.0);

                                                if ui.add(play_btn).clicked() {
                                                    clicked_play = true;
                                                }
                                            },
                                        );
                                    });
                                });

                            if clicked_play {
                                game_to_launch = Some(game.clone());
                            }
                        }
                    });

                if let Some(game) = game_to_launch {
                    self.launch_game(&game);
                }
            });
    }
}
