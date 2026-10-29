use crate::models::{Game, Platform};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub struct BattleNetScanner;

impl BattleNetScanner {
    /// Detects the full path to `Battle.net.exe`.
    pub fn find_launcher_exe() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for subkey in [
                r"SOFTWARE\WOW6432Node\Blizzard Entertainment\Battle.net\Capabilities",
                r"SOFTWARE\Blizzard Entertainment\Battle.net\Capabilities",
            ] {
                if let Ok(key) = hklm.open_subkey_with_flags(subkey, KEY_READ) {
                    if let Ok(cmd) = key.get_value::<String, _>("ApplicationIcon") {
                        let path_str = cmd.split(',').next().unwrap_or(&cmd).trim_matches('"');
                        let p = PathBuf::from(path_str);
                        if p.exists() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        for fallback in [
            r"C:\Program Files (x86)\Battle.net\Battle.net.exe",
            r"C:\Program Files\Battle.net\Battle.net.exe",
            r"D:\Battle.net\Battle.net.exe",
        ] {
            let p = PathBuf::from(fallback);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    /// Checks if Battle.net or its background agent is running.
    pub fn is_launcher_running() -> bool {
        crate::scanners::process::is_process_running("battle.net.exe")
            || crate::scanners::process::is_process_running("agent.exe")
    }

    /// Scans installed Blizzard / Battle.net games.
    pub fn scan_installed_games() -> Vec<Game> {
        let mut games = Vec::new();
        let mut seen_titles = HashSet::new();

        #[cfg(target_os = "windows")]
        {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

            // 1. Scan Windows Uninstall registry for Blizzard Entertainment titles
            for uninstall_root in [
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ] {
                if let Ok(parent) = hklm.open_subkey_with_flags(uninstall_root, KEY_READ) {
                    for key_name in parent.enum_keys().flatten() {
                        if let Ok(sub) = parent.open_subkey_with_flags(&key_name, KEY_READ) {
                            let publisher =
                                sub.get_value::<String, _>("Publisher").unwrap_or_default();
                            let display_name = sub
                                .get_value::<String, _>("DisplayName")
                                .unwrap_or_default();
                            let install_loc = sub
                                .get_value::<String, _>("InstallLocation")
                                .unwrap_or_default();

                            let is_blizzard = publisher.to_lowercase().contains("blizzard")
                                || Self::is_known_blizzard_title(&display_name);

                            if is_blizzard && !display_name.is_empty() {
                                let title_lower = display_name.to_lowercase();
                                if seen_titles.contains(&title_lower)
                                    || title_lower.contains("battle.net")
                                {
                                    continue;
                                }

                                let dir = PathBuf::from(&install_loc);
                                if dir.exists() {
                                    seen_titles.insert(title_lower);
                                    let code = Self::map_title_to_product_code(&display_name);
                                    let launch_uri = format!("battlenet://{}", code);

                                    let mut game = Game::new(
                                        format!("bnet_{}", code.to_lowercase()),
                                        display_name,
                                        Platform::BattleNet,
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

            // 2. Scan HKLM\SOFTWARE\WOW6432Node\Blizzard Entertainment\ for direct installations
            for blizzard_root in [
                r"SOFTWARE\WOW6432Node\Blizzard Entertainment",
                r"SOFTWARE\Blizzard Entertainment",
            ] {
                if let Ok(parent) = hklm.open_subkey_with_flags(blizzard_root, KEY_READ) {
                    for key_name in parent.enum_keys().flatten() {
                        let name_lower = key_name.to_lowercase();
                        if name_lower == "battle.net" || seen_titles.contains(&name_lower) {
                            continue;
                        }

                        if let Ok(sub) = parent.open_subkey_with_flags(&key_name, KEY_READ) {
                            if let Ok(install_path) = sub.get_value::<String, _>("InstallPath") {
                                let dir = PathBuf::from(&install_path);
                                if dir.exists() {
                                    seen_titles.insert(name_lower);
                                    let code = Self::map_title_to_product_code(&key_name);
                                    let launch_uri = format!("battlenet://{}", code);

                                    let mut game = Game::new(
                                        format!("bnet_{}", code.to_lowercase()),
                                        key_name,
                                        Platform::BattleNet,
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
        }

        games.sort_by_key(|a| a.title.to_lowercase());
        games
    }

    fn is_known_blizzard_title(name: &str) -> bool {
        let n = name.to_lowercase();
        n.contains("overwatch")
            || n.contains("world of warcraft")
            || n.contains("diablo")
            || n.contains("hearthstone")
            || n.contains("starcraft")
            || n.contains("heroes of the storm")
            || n.contains("warcraft iii")
    }

    fn map_title_to_product_code(title: &str) -> &'static str {
        let lower = title.to_lowercase();
        if lower.contains("overwatch") {
            "Pro"
        } else if lower.contains("diablo iv") || lower.contains("diablo 4") {
            "Fenris"
        } else if lower.contains("diablo iii") || lower.contains("diablo 3") {
            "D3"
        } else if lower.contains("diablo ii") || lower.contains("diablo 2") {
            "OSI"
        } else if lower.contains("hearthstone") {
            "WTCG"
        } else if lower.contains("starcraft ii") || lower.contains("starcraft 2") {
            "S2"
        } else if lower.contains("starcraft") {
            "S1"
        } else if lower.contains("heroes of the storm") {
            "Hero"
        } else if lower.contains("warcraft iii") || lower.contains("warcraft 3") {
            "W3"
        } else if lower.contains("world of warcraft") {
            "WoW"
        } else {
            "Pro"
        }
    }

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
                                && !name.contains("cleanup")
                                && !name.contains("installer")
                                && !name.contains("crash")
                                && !name.contains("error")
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
    fn test_battlenet_scanner_runs_without_panic() {
        let games = BattleNetScanner::scan_installed_games();
        println!("Discovered {} Battle.net games", games.len());
        let _ = BattleNetScanner::find_launcher_exe();
        let _ = BattleNetScanner::is_launcher_running();
    }

    #[test]
    fn test_battlenet_product_code_mappings() {
        assert_eq!(
            BattleNetScanner::map_title_to_product_code("Overwatch 2"),
            "Pro"
        );
        assert_eq!(
            BattleNetScanner::map_title_to_product_code("Diablo IV"),
            "Fenris"
        );
        assert_eq!(
            BattleNetScanner::map_title_to_product_code("Hearthstone"),
            "WTCG"
        );
        assert_eq!(
            BattleNetScanner::map_title_to_product_code("StarCraft II"),
            "S2"
        );
    }
}
