use crate::models::{Game, Platform};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub struct UbisoftScanner;

impl UbisoftScanner {
    /// Detects the full path to Ubisoft Connect launcher (`upc.exe` or `UbisoftConnect.exe`).
    pub fn find_launcher_exe() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for subkey in [
                r"SOFTWARE\WOW6432Node\Ubisoft\Launcher",
                r"SOFTWARE\Ubisoft\Launcher",
            ] {
                if let Ok(key) = hklm.open_subkey_with_flags(subkey, KEY_READ) {
                    if let Ok(install_dir) = key.get_value::<String, _>("InstallDir") {
                        let base = PathBuf::from(install_dir);
                        for exe_name in ["upc.exe", "UbisoftConnect.exe", "Uplay.exe"] {
                            let p = base.join(exe_name);
                            if p.exists() {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }

        for fallback in [
            r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\upc.exe",
            r"C:\Program Files\Ubisoft\Ubisoft Game Launcher\upc.exe",
            r"D:\Ubisoft Game Launcher\upc.exe",
        ] {
            let p = PathBuf::from(fallback);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    /// Checks if Ubisoft Connect is currently running.
    pub fn is_launcher_running() -> bool {
        crate::scanners::process::is_process_running("upc.exe")
            || crate::scanners::process::is_process_running("ubisoftconnect.exe")
            || crate::scanners::process::is_process_running("uplay.exe")
    }

    /// Scans installed Ubisoft Connect games from Windows Registry.
    pub fn scan_installed_games() -> Vec<Game> {
        let mut games = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let mut seen_ids = HashSet::new();

            // Registry keys that Ubisoft Connect uses to record installed games
            let install_paths = [
                r"SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs",
                r"SOFTWARE\Ubisoft\Launcher\Installs",
            ];

            for path in install_paths {
                if let Ok(installs_key) = hklm.open_subkey_with_flags(path, KEY_READ) {
                    for key_name in installs_key.enum_keys().flatten() {
                        if seen_ids.contains(&key_name) {
                            continue;
                        }

                        if let Ok(sub) = installs_key.open_subkey_with_flags(&key_name, KEY_READ) {
                            let install_dir: Option<PathBuf> = sub
                                .get_value::<String, _>("InstallDir")
                                .ok()
                                .map(PathBuf::from);

                            if let Some(dir) = install_dir {
                                if dir.exists() {
                                    seen_ids.insert(key_name.clone());

                                    let title = Self::resolve_game_title(&key_name, &dir);
                                    let launch_uri = format!("uplay://launch/{}/0", key_name);

                                    let mut game = Game::new(
                                        format!("ubisoft_{}", key_name),
                                        title,
                                        Platform::Ubisoft,
                                        launch_uri,
                                    );

                                    game.install_dir = Some(dir.clone());
                                    game.executable_path = Self::find_game_exe(&dir);
                                    game.install_size = Self::estimate_folder_size(&dir);

                                    games.push(game);
                                }
                            }
                        }
                    }
                }
            }
        }

        games.sort_by_key(|a| a.title.to_lowercase());
        games
    }

    /// Resolves clean display title for a Ubisoft game.
    #[cfg(target_os = "windows")]
    fn resolve_game_title(game_id: &str, dir: &Path) -> String {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // 1. Try Windows Uninstall registry: "Uplay Install <id>"
        let uninstall_paths = [
            format!(
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Uplay Install {}",
                game_id
            ),
            format!(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Uplay Install {}",
                game_id
            ),
        ];

        for u_path in uninstall_paths {
            if let Ok(u_key) = hklm.open_subkey_with_flags(&u_path, KEY_READ) {
                if let Ok(name) = u_key.get_value::<String, _>("DisplayName") {
                    let clean = name.trim();
                    if !clean.is_empty() {
                        return clean.to_string();
                    }
                }
            }
        }

        // 2. Fallback: extract from folder name
        if let Some(folder_name) = dir.file_name().and_then(|n| n.to_str()) {
            let clean = folder_name.replace('_', " ").trim().to_string();
            if !clean.is_empty() {
                return clean;
            }
        }

        format!("Ubisoft Game ({})", game_id)
    }

    #[cfg(not(target_os = "windows"))]
    fn resolve_game_title(game_id: &str, _dir: &Path) -> String {
        format!("Ubisoft Game ({})", game_id)
    }

    /// Locates the primary executable in the game's directory.
    fn find_game_exe(dir: &Path) -> Option<PathBuf> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("exe") {
                            let name = p
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if !name.starts_with("unins")
                                && !name.contains("crash")
                                && !name.contains("report")
                                && !name.contains("easyanticheat")
                                && !name.contains("battleye")
                                && !name.contains("uplay")
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

    /// Quickly estimates directory size from first-level files without deep recursion.
    fn estimate_folder_size(dir: &Path) -> Option<u64> {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        total += meta.len();
                    }
                }
            }
        }
        if total > 0 {
            Some(total)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ubisoft_scanner_runs_without_panic() {
        let games = UbisoftScanner::scan_installed_games();
        println!("Discovered {} Ubisoft games", games.len());
        let _ = UbisoftScanner::find_launcher_exe();
        let _ = UbisoftScanner::is_launcher_running();
    }
}
