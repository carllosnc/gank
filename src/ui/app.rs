use crate::models::{Game, Platform};
use crate::scanners::scan_all_platforms;
use eframe::egui;

pub struct KiroApp {
    games: Vec<Game>,
    search_query: String,
    selected_platform: Option<Platform>,
    status_message: Option<String>,
}

impl Default for KiroApp {
    fn default() -> Self {
        let games = scan_all_platforms();
        Self {
            games,
            search_query: String::new(),
            selected_platform: None,
            status_message: None,
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
            let res = std::process::Command::new("rundll32")
                .args(["url.dll,FileProtocolHandler", &game.launch_uri])
                .spawn();

            match res {
                Ok(_) => {
                    self.status_message = Some(format!("Iniciando: {}", game.title));
                }
                Err(err) => {
                    self.status_message = Some(format!("Erro ao iniciar: {}", err));
                }
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
