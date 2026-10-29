use crate::models::{Game, Platform};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub struct EaScanner;

impl EaScanner {
    /// Detects the full path to `EADesktop.exe` or `Origin.exe`.
    pub fn find_launcher_exe() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for subkey in [
                r"SOFTWARE\WOW6432Node\Electronic Arts\EA Desktop",
                r"SOFTWARE\Electronic Arts\EA Desktop",
                r"SOFTWARE\WOW6432Node\Origin",
                r"SOFTWARE\Origin",
            ] {
                if let Ok(key) = hklm.open_subkey_with_flags(subkey, KEY_READ) {
                    if let Ok(install_path) = key.get_value::<String, _>("InstallLocation")
                        .or_else(|_| key.get_value::<String, _>("ClientPath"))
                    {
                        let p = PathBuf::from(install_path);
                        if p.is_file() && p.exists() {
                            return Some(p);
                        } else if p.is_dir() {
                            for exe in ["EADesktop.exe", "Origin.exe"] {
                                let target = p.join(exe);
                                if target.exists() {
                                    return Some(target);
                                }
                            }
                        }
                    }
                }
            }
        }

        for fallback in [
            r"C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EADesktop.exe",
            r"C:\Program Files (x86)\Origin\Origin.exe",
            r"D:\EA Games\EA Desktop\EADesktop.exe",
        ] {
            let p = PathBuf::from(fallback);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    /// Checks if EA App or Origin is currently running in Windows.
    pub fn is_launcher_running() -> bool {
        crate::scanners::process::is_process_running("eadesktop.exe")
            || crate::scanners::process::is_process_running("origin.exe")
    }

    /// Scans installed EA games from Windows Registry and local manifest directories.
    pub fn scan_installed_games() -> Vec<Game> {
        let mut games = Vec::new();
        let mut seen_ids = HashSet::new();

        #[cfg(target_os = "windows")]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

            // 1. Scan Origin Games / Electronic Arts registry branches
            for root_key in [
                r"SOFTWARE\WOW6432Node\Origin Games",
                r"SOFTWARE\Origin Games",
                r"SOFTWARE\WOW6432Node\Electronic Arts",
            ] {
                if let Ok(parent) = hklm.open_subkey_with_flags(root_key, KEY_READ) {
                    for sub_name in parent.enum_keys().flatten() {
                        if seen_ids.contains(&sub_name) {
                            continue;
                        }

                        if let Ok(sub) = parent.open_subkey_with_flags(&sub_name, KEY_READ) {
                            let title = sub
                                .get_value::<String, _>("DisplayName")
                                .or_else(|_| sub.get_value::<String, _>("Title"))
                                .unwrap_or_else(|_| sub_name.clone());

                            let install_dir = sub
                                .get_value::<String, _>("Install Dir")
                                .or_else(|_| sub.get_value::<String, _>("InstallDir"))
                                .ok()
                                .map(PathBuf::from);

                            if let Some(dir) = install_dir {
                                if dir.exists() {
                                    seen_ids.insert(sub_name.clone());

                                    let launch_uri = format!("origin2://game/launch?offerIds={}", sub_name);
                                    let mut game = Game::new(
                                        format!("ea_{}", sub_name),
                                        title,
                                        Platform::Ea,
                                        launch_uri,
                                    );
                                    game.install_dir = Some(dir.clone());
                                    game.executable_path = Self::find_game_exe(&dir);

                                    games.push(game);
                                }
                            }
                        }
                    }
                }
            }

            // 2. Scan Manifest Files in %ProgramData%\Origin\LocalContent and EA Desktop
            if let Ok(prog_data) = std::env::var("ProgramData") {
                let local_content = PathBuf::from(prog_data).join("Origin").join("LocalContent");
                if local_content.exists() {
                    if let Ok(entries) = fs::read_dir(local_content) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.is_dir() {
                                let id = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                                if !seen_ids.contains(&id) && !id.is_empty() {
                                    if let Some(game) = Self::parse_manifest_dir(&p, &id) {
                                        seen_ids.insert(id);
                                        games.push(game);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        games
    }

    fn parse_manifest_dir(dir: &Path, id: &str) -> Option<Game> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("mfst") {
                    if let Ok(content) = fs::read_to_string(&p) {
                        let mut title = None;
                        let mut install_path = None;

                        for part in content.split('&') {
                            if part.starts_with("title=") {
                                title = Some(part.trim_start_matches("title=").replace('+', " "));
                            } else if part.starts_with("dipinstallpath=") {
                                install_path = Some(PathBuf::from(part.trim_start_matches("dipinstallpath=").replace('+', " ")));
                            }
                        }

                        let title = title.unwrap_or_else(|| id.to_string());
                        let launch_uri = format!("origin2://game/launch?offerIds={}", id);

                        let mut game = Game::new(
                            format!("ea_{}", id),
                            title,
                            Platform::Ea,
                            launch_uri,
                        );

                        if let Some(ip) = install_path {
                            if ip.exists() {
                                game.executable_path = Self::find_game_exe(&ip);
                                game.install_dir = Some(ip);
                            }
                        }

                        return Some(game);
                    }
                }
            }
        }
        None
    }

    fn find_game_exe(dir: &Path) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("exe") {
                            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                            if !name.starts_with("unins")
                                && !name.contains("cleanup")
                                && !name.contains("installer")
                                && !name.contains("touchup")
                                && !name.contains("activation")
                            {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ea_scanner_runs_without_panic() {
        let games = EaScanner::scan_installed_games();
        println!("Discovered {} EA games", games.len());
        let _ = EaScanner::find_launcher_exe();
        let _ = EaScanner::is_launcher_running();
    }
}
