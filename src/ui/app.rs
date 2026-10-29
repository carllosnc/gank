use crate::models::{Game, Platform};
use crate::scanners::battlenet::BattleNetScanner;
use crate::scanners::ea::EaScanner;
use crate::scanners::epic::EpicScanner;
use crate::scanners::steam::SteamScanner;
use crate::scanners::ubisoft::UbisoftScanner;
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

#[cfg(target_os = "windows")]
pub fn set_window_titlebar_black(title: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wide_title: Vec<u16> = OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    #[link(name = "user32")]
    #[link(name = "dwmapi")]
    extern "system" {
        fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> *mut std::ffi::c_void;
        fn DwmSetWindowAttribute(
            hwnd: *mut std::ffi::c_void,
            dwAttribute: u32,
            pvAttribute: *const std::ffi::c_void,
            cbAttribute: u32,
        ) -> i32;
    }

    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), wide_title.as_ptr());
        if !hwnd.is_null() {
            // DWMWA_USE_IMMERSIVE_DARK_MODE = 20
            let dark_mode: i32 = 1;
            let _ = DwmSetWindowAttribute(hwnd, 20, &dark_mode as *const _ as _, 4);

            // DWMWA_CAPTION_COLOR = 35 (Windows 11 build 22000+)
            // COLORREF format 0x00BBGGRR -> pure black is 0x00000000
            let black_caption: u32 = 0x00000000;
            let _ = DwmSetWindowAttribute(hwnd, 35, &black_caption as *const _ as _, 4);

            // DWMWA_BORDER_COLOR = 34
            let _ = DwmSetWindowAttribute(hwnd, 34, &black_caption as *const _ as _, 4);
        }
    }
}

fn paint_card_bottom_scrim(
    painter: &egui::Painter,
    rect: egui::Rect,
    rounding_bottom: f32,
    hover_t: f32,
) {
    if hover_t <= 0.001 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    let col_top = egui::Color32::from_black_alpha(0);
    let col_mid = egui::Color32::from_black_alpha((90.0 * hover_t) as u8);
    let col_bottom = egui::Color32::from_black_alpha((225.0 * hover_t) as u8);

    let y1 = rect.min.y + rect.height() * 0.35;
    let y2 = rect.min.y + rect.height() * 0.68;
    let y3 = rect.max.y - rounding_bottom;
    let y4 = rect.max.y;

    let x_left = rect.min.x;
    let x_right = rect.max.x;
    let x_left_r = rect.min.x + rounding_bottom;
    let x_right_r = rect.max.x - rounding_bottom;

    let add_quad = |m: &mut egui::Mesh, p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2, c0: egui::Color32, c1: egui::Color32, c2: egui::Color32, c3: egui::Color32| {
        let base = m.vertices.len() as u32;
        m.vertices.push(egui::epaint::Vertex { pos: p0, uv: egui::epaint::WHITE_UV, color: c0 });
        m.vertices.push(egui::epaint::Vertex { pos: p1, uv: egui::epaint::WHITE_UV, color: c1 });
        m.vertices.push(egui::epaint::Vertex { pos: p2, uv: egui::epaint::WHITE_UV, color: c2 });
        m.vertices.push(egui::epaint::Vertex { pos: p3, uv: egui::epaint::WHITE_UV, color: c3 });
        m.add_triangle(base, base + 1, base + 2);
        m.add_triangle(base, base + 2, base + 3);
    };

    // Quad y1 (0% alpha) -> y2 (mid alpha)
    add_quad(
        &mut mesh,
        egui::pos2(x_left, y1),
        egui::pos2(x_right, y1),
        egui::pos2(x_right, y2),
        egui::pos2(x_left, y2),
        col_top,
        col_top,
        col_mid,
        col_mid,
    );

    // Quad y2 (mid alpha) -> y3 (bottom alpha)
    add_quad(
        &mut mesh,
        egui::pos2(x_left, y2),
        egui::pos2(x_right, y2),
        egui::pos2(x_right, y3),
        egui::pos2(x_left, y3),
        col_mid,
        col_mid,
        col_bottom,
        col_bottom,
    );

    // Centro inferior y3 -> y4
    add_quad(
        &mut mesh,
        egui::pos2(x_left_r, y3),
        egui::pos2(x_right_r, y3),
        egui::pos2(x_right_r, y4),
        egui::pos2(x_left_r, y4),
        col_bottom,
        col_bottom,
        col_bottom,
        col_bottom,
    );

    // Cantos arredondados na base (y3 -> y4):
    let diag = rounding_bottom * 0.293;
    let add_tri = |m: &mut egui::Mesh, p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2, c: egui::Color32| {
        let base = m.vertices.len() as u32;
        m.vertices.push(egui::epaint::Vertex { pos: p0, uv: egui::epaint::WHITE_UV, color: c });
        m.vertices.push(egui::epaint::Vertex { pos: p1, uv: egui::epaint::WHITE_UV, color: c });
        m.vertices.push(egui::epaint::Vertex { pos: p2, uv: egui::epaint::WHITE_UV, color: c });
        m.add_triangle(base, base + 1, base + 2);
    };

    // Canto inferior direito
    let p_br_c = egui::pos2(x_right_r, y3);
    let p_br_r = egui::pos2(x_right, y3);
    let p_br_corner = egui::pos2(x_right - diag, y4 - diag);
    let p_br_b = egui::pos2(x_right_r, y4);
    add_tri(&mut mesh, p_br_c, p_br_r, p_br_corner, col_bottom);
    add_tri(&mut mesh, p_br_c, p_br_corner, p_br_b, col_bottom);

    // Canto inferior esquerdo
    let p_bl_c = egui::pos2(x_left_r, y3);
    let p_bl_l = egui::pos2(x_left, y3);
    let p_bl_corner = egui::pos2(x_left + diag, y4 - diag);
    let p_bl_b = egui::pos2(x_left_r, y4);
    add_tri(&mut mesh, p_bl_c, p_bl_corner, p_bl_l, col_bottom);
    add_tri(&mut mesh, p_bl_c, p_bl_b, p_bl_corner, col_bottom);

    painter.add(egui::Shape::mesh(mesh));
}

pub struct AnimatedCoverPlayback {
    pub compressed_frames: Vec<Vec<u8>>,
    pub texture: Option<egui::TextureHandle>,
    pub current_frame_idx: usize,
    pub last_frame_time: std::time::Instant,
    pub frame_duration: std::time::Duration,
}

#[allow(dead_code)]
pub struct GankApp {
    games: Vec<Game>,
    selected_platform: Option<Platform>,
    status_message: Option<String>,
    status_tx: Sender<String>,
    status_rx: Receiver<String>,
    cover_tx: Sender<(String, std::path::PathBuf)>,
    cover_rx: Receiver<(String, std::path::PathBuf)>,
    reveal_start_time: Option<std::time::Instant>,
    start_time: std::time::Instant,
    cached_storage_text: String,
    cached_count_text: String,
    cover_textures: std::collections::HashMap<String, egui::TextureHandle>,
    tex_load_tx: Sender<(String, egui::ColorImage)>,
    tex_load_rx: Receiver<(String, egui::ColorImage)>,
    logo_texture: Option<egui::TextureHandle>,
    animated_covers: std::collections::HashMap<String, AnimatedCoverPlayback>,
    anim_loading: std::collections::HashSet<String>,
    animated_cover_ids: std::collections::HashSet<String>,
    titlebar_themed: bool,
    anim_tx: Sender<(String, Vec<Vec<u8>>)>,
    anim_rx: Receiver<(String, Vec<Vec<u8>>)>,
}

impl Default for GankApp {
    fn default() -> Self {
        let (status_tx, status_rx) = channel();
        let (cover_tx, cover_rx) = channel();
        let (anim_tx, anim_rx) = channel::<(String, Vec<Vec<u8>>)>();
        let (tex_load_tx, tex_load_rx) = channel::<(String, egui::ColorImage)>();

        let games = scan_all_platforms();

        // Dispatch asynchronous pre-warming of cover textures in background
        let covers_to_load: Vec<(String, std::path::PathBuf)> = games
            .iter()
            .filter_map(|g| g.cover_image_path.as_ref().map(|p| (g.id.clone(), p.clone())))
            .collect();

        let tx = tex_load_tx.clone();
        std::thread::spawn(move || {
            for (id, path) in covers_to_load {
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_raw();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                        let _ = tx.send((id, color_image));
                    }
                }
            }
        });

        let mut app = Self {
            games,
            selected_platform: None,
            status_message: None,
            status_tx,
            status_rx,
            cover_tx,
            cover_rx,
            reveal_start_time: None, // Initiated on first UI frame render
            start_time: std::time::Instant::now(),
            cached_storage_text: String::new(),
            cached_count_text: String::new(),
            cover_textures: std::collections::HashMap::new(),
            tex_load_tx,
            tex_load_rx,
            logo_texture: None,
            animated_covers: std::collections::HashMap::new(),
            anim_loading: std::collections::HashSet::new(),
            animated_cover_ids: crate::artwork::get_all_animated_cover_ids(),
            titlebar_themed: false,
            anim_tx,
            anim_rx,
        };

        app.update_cached_stats();
        app
    }
}

impl GankApp {
    pub fn update_cached_stats(&mut self) {
        let total_bytes: u64 = self.games.iter().filter_map(|g| g.install_size).sum();
        const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
        let gib = total_bytes as f64 / GIB;
        self.cached_storage_text = format!("{} {:.1} GB", egui_phosphor::regular::HARD_DRIVES, gib);
        self.cached_count_text = format!("{} {} jogos instalados", egui_phosphor::regular::GAME_CONTROLLER, self.games.len());
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 1. Configure Typography: Inter (Regular & SemiBold) + Phosphor Icons
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "Inter-Regular".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.otf")),
        );
        fonts.font_data.insert(
            "Inter-SemiBold".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.otf")),
        );

        // Add Phosphor icons directly into the egui font families
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        // Set Inter as the primary text font for all proportional text
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "Inter-Regular".to_owned());
        }

        cc.egui_ctx.set_fonts(fonts);

        // 2. Configure sleek scrollbar style flush to edge and dark UI palette
        cc.egui_ctx.style_mut(|s| {
            s.spacing.scroll.bar_outer_margin = 0.0;
            s.spacing.scroll.bar_inner_margin = 2.0;
            s.spacing.scroll.bar_width = 6.0;
            s.visuals.extreme_bg_color = egui::Color32::from_rgb(18, 20, 26);
            s.visuals.window_fill = egui::Color32::BLACK;
            s.visuals.panel_fill = egui::Color32::BLACK;
            s.visuals.window_stroke = egui::Stroke::NONE;
            s.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
        });

        // Initialize native WGPU pipeline for Floating Cosmic Dust shader
        if let Some(render_state) = &cc.wgpu_render_state {
            let dust_state = crate::ui::aurora_shader::CosmicDustBackgroundRenderState::new(
                &render_state.device,
                render_state.target_format,
            );
            render_state
                .renderer
                .write()
                .callback_resources
                .insert(dust_state);
        }

        Self::default()
    }

    #[allow(dead_code)]
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

            // If Ubisoft is selected but not yet running, start upc.exe first
            if game.platform == Platform::Ubisoft && !UbisoftScanner::is_launcher_running() {
                if let Some(launcher_exe) = UbisoftScanner::find_launcher_exe() {
                    let tx = self.status_tx.clone();
                    let title = game.title.clone();
                    let uri = game.launch_uri.clone();

                    self.status_message = Some("Iniciando Ubisoft Connect em segundo plano...".to_string());

                    std::thread::spawn(move || {
                        let mut cmd = std::process::Command::new(&launcher_exe);
                        if let Some(parent) = launcher_exe.parent() {
                            cmd.current_dir(parent);
                        }
                        let _ = cmd.spawn();
                        std::thread::sleep(std::time::Duration::from_millis(2500));
                        let _ = tx.send(format!("Iniciando: {}", title));
                        open_uri(&uri);
                    });
                    return;
                }
            }

            // If EA App is selected but not yet running, start EADesktop.exe first
            if game.platform == Platform::Ea && !EaScanner::is_launcher_running() {
                if let Some(launcher_exe) = EaScanner::find_launcher_exe() {
                    let tx = self.status_tx.clone();
                    let title = game.title.clone();
                    let uri = game.launch_uri.clone();

                    self.status_message = Some("Iniciando EA App em segundo plano...".to_string());

                    std::thread::spawn(move || {
                        let mut cmd = std::process::Command::new(&launcher_exe);
                        if let Some(parent) = launcher_exe.parent() {
                            cmd.current_dir(parent);
                        }
                        let _ = cmd.spawn();
                        std::thread::sleep(std::time::Duration::from_millis(2500));
                        let _ = tx.send(format!("Iniciando: {}", title));
                        open_uri(&uri);
                    });
                    return;
                }
            }

            // If Battle.net is selected but not yet running, start Battle.net.exe first
            if game.platform == Platform::BattleNet && !BattleNetScanner::is_launcher_running() {
                if let Some(launcher_exe) = BattleNetScanner::find_launcher_exe() {
                    let tx = self.status_tx.clone();
                    let title = game.title.clone();
                    let uri = game.launch_uri.clone();

                    self.status_message = Some("Iniciando Battle.net em segundo plano...".to_string());

                    std::thread::spawn(move || {
                        let mut cmd = std::process::Command::new(&launcher_exe);
                        if let Some(parent) = launcher_exe.parent() {
                            cmd.current_dir(parent);
                        }
                        let _ = cmd.spawn();
                        std::thread::sleep(std::time::Duration::from_millis(2500));
                        let _ = tx.send(format!("Iniciando: {}", title));
                        open_uri(&uri);
                    });
                    return;
                }
            }

            // Emulator / ROM launch handling
            if game.platform == Platform::Emulator {
                if game.launch_uri.starts_with('"') && game.launch_uri.contains("\" \"") {
                    let parts: Vec<&str> = game.launch_uri.split("\" \"").collect();
                    if parts.len() == 2 {
                        let emu_bin = parts[0].trim_start_matches('"');
                        let rom_file = parts[1].trim_end_matches('"');
                        let mut cmd = std::process::Command::new(emu_bin);
                        cmd.arg(rom_file);
                        if let Some(parent) = std::path::Path::new(emu_bin).parent() {
                            cmd.current_dir(parent);
                        }
                        if cmd.spawn().is_ok() {
                            self.status_message = Some(format!("Iniciando: {}", game.title));
                            return;
                        }
                    }
                } else if let Some(exe) = &game.executable_path {
                    let mut cmd = std::process::Command::new(exe);
                    if let Some(dir) = &game.install_dir {
                        cmd.current_dir(dir);
                    }
                    if cmd.spawn().is_ok() {
                        self.status_message = Some(format!("Iniciando: {}", game.title));
                        return;
                    }
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
    #[allow(dead_code)]
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

impl eframe::App for GankApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start reveal animation on the very first frame rendered to screen
        if self.reveal_start_time.is_none() {
            self.reveal_start_time = Some(std::time::Instant::now());
        }

        // Process asynchronous status updates
        while let Ok(msg) = self.status_rx.try_recv() {
            self.status_message = Some(msg);
            ctx.request_repaint();
        }

        // Process asynchronous animated cover frame arrivals
        while let Ok((id, frames)) = self.anim_rx.try_recv() {
            self.anim_loading.remove(&id);
            if !frames.is_empty() {
                self.animated_covers.insert(
                    id,
                    AnimatedCoverPlayback {
                        compressed_frames: frames,
                        texture: None,
                        current_frame_idx: 0,
                        last_frame_time: std::time::Instant::now(),
                        frame_duration: std::time::Duration::from_millis(55),
                    },
                );
            }
        }

        // Ingest pre-warmed cover textures from background decoding
        while let Ok((id, color_image)) = self.tex_load_rx.try_recv() {
            let handle = ctx.load_texture(
                format!("cover_tex_{}", id),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.cover_textures.insert(id, handle);
            ctx.request_repaint();
        }

        // Process asynchronous cover updates
        while let Ok((id, path)) = self.cover_rx.try_recv() {
            self.animated_covers.remove(&id);
            self.anim_loading.remove(&id);
            self.cover_textures.remove(&id);
            self.animated_cover_ids = crate::artwork::get_all_animated_cover_ids();
            if let Some(game) = self.games.iter_mut().find(|g| g.id == id) {
                game.set_cover_path(Some(path.clone()));
            }
            let tx = self.tex_load_tx.clone();
            let game_id = id.clone();
            std::thread::spawn(move || {
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_raw();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                        let _ = tx.send((game_id, color_image));
                    }
                }
            });
            ctx.request_repaint();
        }

        // Ensure native Windows titlebar is styled black
        if !self.titlebar_themed {
            #[cfg(target_os = "windows")]
            set_window_titlebar_black("Gank - Game Hub");
            self.titlebar_themed = true;
        }

        // Paint Floating Cosmic Dust on the background layer
        let screen_rect = ctx.screen_rect();
        let time = self.start_time.elapsed().as_secs_f32();
        let aspect = screen_rect.width() / screen_rect.height().max(1.0);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            screen_rect,
            crate::ui::aurora_shader::CosmicDustCallback { time, aspect },
        );
        ctx.layer_painter(egui::LayerId::background()).add(callback);

        // Frame pacing: smooth VSync rendering when focused, throttled to 15fps when unfocused to spare CPU/GPU
        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        let is_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if !is_minimized {
            if is_focused {
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(66));
            }
        }

        // Top Navigation Bar (Transparent, NO separator line)
        egui::TopBottomPanel::top("top_nav")
            .show_separator_line(false)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(crate::ui::theme::spacing::PAGE_PADDING_X, 16.0)),
            )
            .show(ctx, |ui| {
                let total_width = ui.available_width();
                let bar_h = 34.0;
                let bar_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(total_width, bar_h));

                // Breakpoint: Below 1100px, collapse pills into compact icon badges so they never crowd the header
                let is_compact = total_width < 1100.0;
                let show_pill_text = !is_compact;

                // 1. Right Actions Width & Rect (Refresh Button)
                let right_w = 32.0;
                let right_rect = egui::Rect::from_min_max(
                    egui::pos2(bar_rect.max.x - right_w, bar_rect.min.y),
                    bar_rect.max,
                );

                // Ensure top logo texture is loaded
                if self.logo_texture.is_none() {
                    let logo_bytes = include_bytes!("../../assets/logo.png");
                    if let Ok(img) = image::load_from_memory(logo_bytes) {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_raw();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                        self.logo_texture = Some(ctx.load_texture("top_logo", color_image, egui::TextureOptions::LINEAR));
                    }
                }

                // 2. Left Brand Width & Rect (official Gank wordmark logo)
                let brand_w = 68.0;
                let left_rect = egui::Rect::from_min_max(
                    bar_rect.min,
                    egui::pos2(bar_rect.min.x + brand_w, bar_rect.max.y),
                );

                // 3. Middle Rect strictly bounded between brand and right actions (IMPOSSIBLE TO OVERLAP)
                let middle_gap = if is_compact { 10.0 } else { 16.0 };
                let middle_rect = egui::Rect::from_min_max(
                    egui::pos2(bar_rect.min.x + brand_w + middle_gap, bar_rect.min.y),
                    egui::pos2(bar_rect.max.x - right_w - middle_gap, bar_rect.max.y),
                );

                // Allocate exact top bar height in parent UI
                ui.allocate_rect(bar_rect, egui::Sense::hover());

                // RENDER RIGHT ACTIONS (Fixed right anchor)
                let mut right_ui = ui.child_ui(right_rect, egui::Layout::right_to_left(egui::Align::Center));

                // 1. Refresh Button
                if crate::ui::components::icon_button(
                    &mut right_ui,
                    egui_phosphor::regular::ARROWS_CLOCKWISE,
                    "Atualizar biblioteca",
                ).clicked() {
                    self.games = scan_all_platforms();
                    self.update_cached_stats();
                    let covers_to_load: Vec<(String, std::path::PathBuf)> = self.games
                        .iter()
                        .filter_map(|g| g.cover_image_path.as_ref().map(|p| (g.id.clone(), p.clone())))
                        .collect();
                    let tx = self.tex_load_tx.clone();
                    std::thread::spawn(move || {
                        for (id, path) in covers_to_load {
                            if let Ok(bytes) = std::fs::read(&path) {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    let rgba = img.to_rgba8();
                                    let size = [rgba.width() as usize, rgba.height() as usize];
                                    let pixels = rgba.into_raw();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                    let _ = tx.send((id, color_image));
                                }
                            }
                        }
                    });
                    self.reveal_start_time = Some(std::time::Instant::now());
                    self.status_message = Some("Biblioteca atualizada!".to_string());
                }

                // RENDER LEFT BRAND (Fixed left anchor: small official Gank logo fitting top bar)
                let mut left_ui = ui.child_ui(left_rect, egui::Layout::left_to_right(egui::Align::Center));
                if let Some(tex) = &self.logo_texture {
                    let (logo_rect, resp) = left_ui.allocate_exact_size(egui::vec2(65.0, 18.0), egui::Sense::click());
                    let logo_img = egui::Image::new(tex).fit_to_exact_size(egui::vec2(65.0, 18.0));
                    logo_img.paint_at(&left_ui, logo_rect);
                    if resp.hovered() {
                        left_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        self.selected_platform = None;
                        self.reveal_start_time = Some(std::time::Instant::now());
                    }
                } else {
                    left_ui.label(
                        egui::RichText::new("Gank")
                            .color(egui::Color32::WHITE)
                            .strong()
                            .size(15.5),
                    );
                }

                // RENDER MIDDLE FILTERS (Strictly clipped & scrollable inside middle_rect)
                let mut middle_ui = ui.child_ui(middle_rect, egui::Layout::left_to_right(egui::Align::Center));
                middle_ui.set_clip_rect(middle_rect);

                egui::ScrollArea::horizontal()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([true, false])
                    .show(&mut middle_ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(if is_compact { 6.0 } else { 10.0 }, 0.0);

                        let all_selected = self.selected_platform.is_none();
                        if crate::ui::components::filter_pill_image(ui, all_selected, "Todos", None, true).clicked() {
                            self.selected_platform = None;
                            self.reveal_start_time = Some(std::time::Instant::now());
                        }

                        let platform_options = [
                            (Platform::Steam, "Steam", crate::ui::icons::STEAM_ICON),
                            (Platform::EpicGames, "Epic", crate::ui::icons::EPIC_ICON),
                            (Platform::Ubisoft, "Ubisoft", crate::ui::icons::UBISOFT_ICON),
                            (Platform::Ea, "EA", crate::ui::icons::EA_ICON),
                            (Platform::BattleNet, "Blizzard", crate::ui::icons::BATTLENET_ICON),
                            (Platform::Emulator, "Emuladores", crate::ui::icons::RETROARCH_ICON),
                            (Platform::Gog, "GOG", crate::ui::icons::GOG_ICON),
                            (Platform::Xbox, "Xbox", crate::ui::icons::XBOX_ICON),
                        ];

                        for (plat, label, icon) in platform_options {
                            let has_games = self.games.iter().any(|g| g.platform == plat);
                            if has_games || plat == Platform::Steam || plat == Platform::EpicGames {
                                let is_selected = self.selected_platform == Some(plat);
                                if crate::ui::components::filter_pill_image(
                                    ui,
                                    is_selected,
                                    label,
                                    Some(icon),
                                    show_pill_text,
                                ).clicked() {
                                    self.selected_platform = if is_selected { None } else { Some(plat) };
                                    self.reveal_start_time = Some(std::time::Instant::now());
                                }
                            }
                        }
                    });
            });

        // Bottom Status Bar (30% Opacity Black, NO separator line)
        egui::TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_black_alpha(77))
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(crate::ui::theme::spacing::PAGE_PADDING_X, 14.0)),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);

                    // Left Status Indicator (enhanced font and icon sizes, perfectly aligned)
                    if let Some(msg) = &self.status_message {
                        ui.label(
                            egui::RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                                .color(crate::ui::theme::colors::SUCCESS)
                                .size(20.0),
                        );
                        ui.label(
                            egui::RichText::new(msg)
                                .color(crate::ui::theme::colors::TEXT_PRIMARY)
                                .font(crate::ui::theme::typography::body_medium()),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                                .color(crate::ui::theme::colors::SUCCESS)
                                .size(20.0),
                        );
                        ui.label(
                            egui::RichText::new("Pronto")
                                .color(crate::ui::theme::colors::TEXT_SECONDARY)
                                .font(crate::ui::theme::typography::body_medium()),
                        );
                    }

                    // Right Side: Game counts & Total Disk Usage (increased font and icon sizes)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(&self.cached_storage_text)
                                .color(egui::Color32::from_rgb(145, 152, 165))
                                .size(13.5),
                        );
                        ui.label(
                            egui::RichText::new("•")
                                .color(egui::Color32::from_rgb(75, 82, 95))
                                .size(13.0),
                        );
                        ui.label(
                            egui::RichText::new(&self.cached_count_text)
                                .color(egui::Color32::from_rgb(145, 152, 165))
                                .size(13.5),
                        );
                    });
                });
            });

        // Main Responsive Games Grid with Transparent Frame revealing Mesh Gradient Background
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
                    .inner_margin(egui::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                let filtered_games: Vec<&Game> = self
                    .games
                    .iter()
                    .filter(|g| {
                        if let Some(plat) = self.selected_platform {
                            g.platform == plat
                        } else {
                            true
                        }
                    })
                    .collect();

                if filtered_games.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(
                            egui::RichText::new(egui_phosphor::regular::GAME_CONTROLLER)
                                .size(48.0)
                                .color(egui::Color32::from_rgb(90, 95, 110)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Nenhum jogo encontrado.")
                                .color(egui::Color32::from_rgb(150, 155, 165))
                                .size(16.0),
                        );
                    });
                    return;
                }

                let panel_rect = ui.available_rect_before_wrap();
                let mut game_to_launch: Option<Game> = None;
                let mut grid_ui = ui.child_ui(panel_rect, egui::Layout::top_down(egui::Align::Min));

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(&mut grid_ui, |ui| {
                            egui::Frame::none()
                                .inner_margin(crate::ui::theme::spacing::PAGE_PADDING)
                                .show(ui, |ui| {
                                // Responsive Dynamic Grid metrics: cards expand to fill 100% width
                                let available_w = ui.available_width();
                                let spacing = 20.0;
                                let target_card_w = 200.0;
                                let cols = ((available_w + spacing) / (target_card_w + spacing)).floor().max(1.0) as usize;
                                let card_w = ((available_w - (cols - 1) as f32 * spacing) / cols as f32).max(120.0);
                                let poster_h = (card_w * 1.5).round();

                                ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                                // Responsive, smooth cascade reveal metrics (pre-warms all visible images immediately)
                                let now = std::time::Instant::now();
                                let elapsed = self
                                    .reveal_start_time
                                    .map(|t| (now - t).as_secs_f32())
                                    .unwrap_or(10.0);
                                let anim_duration = 0.38_f32; // 380ms smooth ease per card
                                let mut any_animating = false;
                                let mut any_anim_playing = false;

                                for (row_idx, row_chunk) in filtered_games.chunks(cols).enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);
                                        for (col_idx, &game) in row_chunk.iter().enumerate() {
                                            // Enhanced cascade: 80ms initial + 75ms per row + 35ms per column (capped at 650ms)
                                            let card_delay = (0.08_f32
                                                + row_idx as f32 * 0.075_f32
                                                + col_idx as f32 * 0.035_f32)
                                                .min(0.65);

                                            let progress = if elapsed < card_delay {
                                                any_animating = true;
                                                0.0_f32
                                            } else {
                                                let p = ((elapsed - card_delay) / anim_duration).clamp(0.0, 1.0);
                                                if p < 1.0 {
                                                    any_animating = true;
                                                }
                                                p
                                            };

                                            let _card_response = egui::Frame::none()
                                                .inner_margin(egui::Margin::same(0.0))
                                                .show(ui, |ui| {
                                                    ui.set_width(card_w);
                                                    ui.vertical(|ui| {
                                                        // --- POSTER / COVER AREA ONLY (NO BORDERS, NO BADGES, NO FOOTER) ---
                                                        let (poster_rect, _) = ui.allocate_exact_size(
                                                            egui::vec2(card_w, poster_h),
                                                            egui::Sense::hover(),
                                                        );

                                                        // Viewport Culling: Skip rendering completely if card is scrolled off-screen (120px overscan buffer)
                                                        if !ui.is_rect_visible(poster_rect.expand(120.0)) {
                                                            return;
                                                        }

                                                        // Ease-out cubic: smooth slide-up and fade-in
                                                        let t = 1.0_f32 - (1.0_f32 - progress).powi(3);
                                                        let y_offset = (1.0_f32 - t) * 18.0_f32;
                                                        let alpha_u8 = (t * 255.0_f32).clamp(0.0, 255.0) as u8;
                                                        let animated_rect = poster_rect.translate(egui::vec2(0.0, y_offset));

                                                        // Robust geometric hover check
                                                        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                                                        let is_pointer_in_card = pointer_pos.map_or(false, |pos| animated_rect.contains(pos))
                                                            && t > 0.85;

                                                        let hover_t = ui.ctx().animate_bool_with_time(
                                                            ui.id().with("card_hover_anim").with(&game.id),
                                                            is_pointer_in_card,
                                                            0.20,
                                                        );

                                                        if hover_t > 0.01 && hover_t < 0.99 {
                                                            ui.ctx().request_repaint();
                                                        }

                                                        // 1. Draw solid poster base surface
                                                        let card_bg = egui::Color32::from_rgba_unmultiplied(20, 23, 30, alpha_u8);
                                                        ui.painter().rect_filled(
                                                            animated_rect,
                                                            egui::Rounding::same(8.0),
                                                            card_bg,
                                                        );

                                                        // 2. Draw Cover (Animated Video Loop or Static Fallback) with alpha tint
                                                        let mut painted_cover = false;
                                                        if self.animated_cover_ids.contains(&game.id) {
                                                            if let Some(playback) = self.animated_covers.get_mut(&game.id) {
                                                                let now = std::time::Instant::now();
                                                                if now.duration_since(playback.last_frame_time) >= playback.frame_duration {
                                                                    if !playback.compressed_frames.is_empty() {
                                                                        playback.current_frame_idx = (playback.current_frame_idx + 1) % playback.compressed_frames.len();
                                                                        playback.last_frame_time = now;
                                                                        if let Ok(img) = image::load_from_memory(&playback.compressed_frames[playback.current_frame_idx]) {
                                                                            let rgba = img.to_rgba8();
                                                                            let size = [rgba.width() as usize, rgba.height() as usize];
                                                                            let pixels = rgba.into_raw();
                                                                            let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                                                            if let Some(tex) = &mut playback.texture {
                                                                                tex.set(color_img, egui::TextureOptions::LINEAR);
                                                                            } else {
                                                                                playback.texture = Some(ui.ctx().load_texture(
                                                                                    format!("anim_tex_{}", game.id),
                                                                                    color_img,
                                                                                    egui::TextureOptions::LINEAR,
                                                                                ));
                                                                            }
                                                                        }
                                                                    }
                                                                }

                                                                if playback.texture.is_none() && !playback.compressed_frames.is_empty() {
                                                                    if let Ok(img) = image::load_from_memory(&playback.compressed_frames[playback.current_frame_idx]) {
                                                                        let rgba = img.to_rgba8();
                                                                        let size = [rgba.width() as usize, rgba.height() as usize];
                                                                        let pixels = rgba.into_raw();
                                                                        let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                                                        playback.texture = Some(ui.ctx().load_texture(
                                                                            format!("anim_tex_{}", game.id),
                                                                            color_img,
                                                                            egui::TextureOptions::LINEAR,
                                                                        ));
                                                                    }
                                                                }

                                                                if let Some(tex) = &playback.texture {
                                                                    let img = egui::Image::new(tex)
                                                                        .fit_to_exact_size(egui::vec2(card_w, poster_h))
                                                                        .rounding(egui::Rounding::same(8.0))
                                                                        .tint(egui::Color32::from_white_alpha(alpha_u8));
                                                                    img.paint_at(ui, animated_rect);
                                                                    painted_cover = true;
                                                                    any_anim_playing = true;
                                                                }
                                                            } else if !self.anim_loading.contains(&game.id) {
                                                                self.anim_loading.insert(game.id.clone());
                                                                let game_id = game.id.clone();
                                                                let tx = self.anim_tx.clone();
                                                                std::thread::spawn(move || {
                                                                    let frames = crate::artwork::load_animated_cover_compressed_frames(&game_id);
                                                                    let _ = tx.send((game_id, frames));
                                                                });
                                                            }
                                                        }

                                                        if !painted_cover {
                                                            if let Some(tex) = self.cover_textures.get(&game.id) {
                                                                let img = egui::Image::new(tex)
                                                                    .fit_to_exact_size(egui::vec2(card_w, poster_h))
                                                                    .rounding(egui::Rounding::same(8.0))
                                                                    .tint(egui::Color32::from_white_alpha(alpha_u8));
                                                                img.paint_at(ui, animated_rect);
                                                            } else if let Some(uri) = &game.cover_uri {
                                                                let img = egui::Image::new(uri)
                                                                    .fit_to_exact_size(egui::vec2(card_w, poster_h))
                                                                    .rounding(egui::Rounding::same(8.0))
                                                                    .tint(egui::Color32::from_white_alpha(alpha_u8))
                                                                    .show_loading_spinner(false);
                                                                img.paint_at(ui, animated_rect);
                                                            } else if let Some(cover_path) = &game.cover_image_path {
                                                                let uri = format!("file://{}", cover_path.to_string_lossy().replace('\\', "/"));
                                                                let img = egui::Image::new(uri)
                                                                    .fit_to_exact_size(egui::vec2(card_w, poster_h))
                                                                    .rounding(egui::Rounding::same(8.0))
                                                                    .tint(egui::Color32::from_white_alpha(alpha_u8))
                                                                    .show_loading_spinner(false);
                                                                img.paint_at(ui, animated_rect);
                                                            } else {
                                                                let mut bg_color = Self::title_accent_color(&game.title);
                                                                bg_color = egui::Color32::from_rgba_unmultiplied(
                                                                    bg_color.r(),
                                                                    bg_color.g(),
                                                                    bg_color.b(),
                                                                    alpha_u8,
                                                                );
                                                                ui.painter().rect_filled(
                                                                    animated_rect,
                                                                    egui::Rounding::same(8.0),
                                                                    bg_color,
                                                                );

                                                                let initials: String = game.title
                                                                    .split_whitespace()
                                                                    .filter_map(|w| w.chars().next())
                                                                    .take(3)
                                                                    .collect();
                                                                ui.painter().text(
                                                                    animated_rect.center() - egui::vec2(0.0, 16.0),
                                                                    egui::Align2::CENTER_CENTER,
                                                                    initials.to_uppercase(),
                                                                    egui::FontId::proportional(40.0),
                                                                    egui::Color32::from_white_alpha((45.0_f32 * t) as u8),
                                                                );

                                                                let display_title = if game.title.len() > 32 {
                                                                    format!("{}...", &game.title[..29])
                                                                } else {
                                                                    game.title.clone()
                                                                };
                                                                ui.painter().text(
                                                                    animated_rect.center() + egui::vec2(0.0, 24.0),
                                                                    egui::Align2::CENTER_CENTER,
                                                                    display_title,
                                                                    egui::FontId::proportional(14.0),
                                                                    egui::Color32::from_white_alpha((220.0_f32 * t) as u8),
                                                                );
                                                            }
                                                        }

                                                        // 3. Play Action Button Hover Overlay with bottom scrim gradient & top-to-bottom slide
                                                        if hover_t > 0.001 {
                                                            // Dark bottom scrim gradient (0% opacity at top, deep dark at bottom)
                                                            paint_card_bottom_scrim(ui.painter(), animated_rect, 8.0, hover_t);

                                                            let btn_h = 36.0;
                                                            let icon_btn_size = 36.0;
                                                            let gap = 8.0;
                                                            let row_w = (card_w - 24.0).clamp(118.0, 160.0);
                                                            let play_w = (row_w - gap - icon_btn_size).max(70.0);
                                                            let total_w = play_w + gap + icon_btn_size;

                                                            // Position at the bottom of the card with bottom margin
                                                            let bottom_margin = 14.0_f32;
                                                            let rest_y = animated_rect.max.y - bottom_margin - btn_h * 0.5;
                                                            let rest_x = animated_rect.center().x;
                                                            let rest_center = egui::pos2(rest_x, rest_y);

                                                            // Smooth ease-out cubic curve: slides from 22px above downwards into bottom rest position
                                                            let slide_ease = 1.0_f32 - (1.0_f32 - hover_t).powi(3);
                                                            let btn_y_offset = (1.0_f32 - slide_ease) * -22.0_f32;
                                                            let row_center = rest_center + egui::vec2(0.0, btn_y_offset);

                                                            let row_start_x = row_center.x - total_w * 0.5;
                                                            let play_rect = egui::Rect::from_min_size(
                                                                egui::pos2(row_start_x, row_center.y - btn_h * 0.5),
                                                                egui::vec2(play_w, btn_h),
                                                            );
                                                            let cover_rect = egui::Rect::from_min_size(
                                                                egui::pos2(row_start_x + play_w + gap, row_center.y - btn_h * 0.5),
                                                                egui::vec2(icon_btn_size, icon_btn_size),
                                                            );

                                                            // Play Button
                                                            let play_id = ui.id().with("btn_play").with(&game.id);
                                                            let play_resp = crate::ui::components::card_action_button(
                                                                ui,
                                                                play_id,
                                                                play_rect,
                                                                egui_phosphor::regular::PLAY,
                                                                "Jogar",
                                                                true,
                                                                hover_t,
                                                            );
                                                            if play_resp.clicked() {
                                                                game_to_launch = Some((*game).clone());
                                                            }

                                                            // Modify Cover Button
                                                            let cover_id = ui.id().with("btn_cover").with(&game.id);
                                                            let cover_resp = crate::ui::components::card_icon_button(
                                                                ui,
                                                                cover_id,
                                                                cover_rect,
                                                                egui_phosphor::regular::IMAGE,
                                                                "Alterar Capa",
                                                                hover_t,
                                                            );
                                                            if cover_resp.clicked() {
                                                                let tx = self.cover_tx.clone();
                                                                let status_tx = self.status_tx.clone();
                                                                let game_id = game.id.clone();
                                                                std::thread::spawn(move || {
                                                                    if let Some(src_path) = crate::artwork::pick_image_file() {
                                                                        if let Ok(new_cover) = crate::artwork::save_custom_cover(&game_id, &src_path) {
                                                                            let _ = tx.send((game_id, new_cover));
                                                                            let _ = status_tx.send("Capa atualizada com sucesso!".to_string());
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        }
                                                    });
                                                });
                                        }
                                    });
                                }

                                if any_animating {
                                    ui.ctx().request_repaint();
                                }

                                if any_anim_playing {
                                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
                                }
                            });
                    });

                if let Some(game) = game_to_launch {
                    self.launch_game(&game);
                }
        });
    }
}
