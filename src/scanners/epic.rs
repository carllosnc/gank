use crate::models::{Game, Platform};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[derive(Debug, Deserialize)]
struct EpicManifest {
    #[serde(rename = "DisplayName")]
    display_name: Option<String>,
    #[serde(rename = "AppName")]
    app_name: Option<String>,
    #[serde(rename = "CatalogNamespace")]
    catalog_namespace: Option<String>,
    #[serde(rename = "CatalogItemId")]
    catalog_item_id: Option<String>,
    #[serde(rename = "InstallLocation")]
    install_location: Option<String>,
    #[serde(rename = "LaunchExecutable")]
    launch_executable: Option<String>,
    #[serde(rename = "MainGameAppName", default)]
    main_game_app_name: Option<String>,
    #[serde(rename = "bIsIncompleteInstall", default)]
    is_incomplete_install: Option<bool>,
}

pub struct EpicScanner;

impl EpicScanner {
    /// Detects the full path to `EpicGamesLauncher.exe` via Windows registry or standard locations.
    pub fn find_epic_launcher_exe() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            use winreg::enums::{HKEY_CLASSES_ROOT, KEY_READ};
            let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
            if let Ok(key) = hkcr.open_subkey_with_flags(r"com.epicgames.launcher\shell\open\command", KEY_READ) {
                if let Ok(cmd_str) = key.get_value::<String, _>("") {
                    let exe_str = if let Some(first_quote) = cmd_str.find('"') {
                        if let Some(second_quote) = cmd_str[first_quote + 1..].find('"') {
                            &cmd_str[first_quote + 1..first_quote + 1 + second_quote]
                        } else {
                            &cmd_str
                        }
                    } else {
                        cmd_str.split_whitespace().next().unwrap_or(&cmd_str)
                    };

                    let path = PathBuf::from(exe_str);
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        for fallback in [
            r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
            r"C:\Program Files\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
            r"E:\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
        ] {
            let p = PathBuf::from(fallback);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    /// Checks if EpicGamesLauncher.exe is currently running in Windows.
    pub fn is_launcher_running() -> bool {
        crate::scanners::process::is_process_running("epicgameslauncher.exe")
    }

    /// Detects the manifests directory of Epic Games Launcher.
    pub fn find_manifests_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            // 1. Try registry
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for key_path in [
                r"SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher",
                r"SOFTWARE\Epic Games\EpicGamesLauncher",
            ] {
                if let Ok(key) = hklm.open_subkey_with_flags(key_path, KEY_READ) {
                    if let Ok(app_data_path) = key.get_value::<String, _>("AppDataPath") {
                        let manifests = PathBuf::from(app_data_path).join("Manifests");
                        if manifests.exists() {
                            return Some(manifests);
                        }
                    }
                }
            }

            // 2. Try ProgramData environment variable
            if let Ok(program_data) = std::env::var("ProgramData") {
                let manifests = PathBuf::from(program_data)
                    .join("Epic")
                    .join("EpicGamesLauncher")
                    .join("Data")
                    .join("Manifests");
                if manifests.exists() {
                    return Some(manifests);
                }
            }
        }

        // 3. Fallback to default path
        let default_path = PathBuf::from(r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests");
        if default_path.exists() {
            Some(default_path)
        } else {
            None
        }
    }

    /// Scans all `.item` files in Epic manifests directory and returns installed games.
    pub fn scan_installed_games() -> Vec<Game> {
        let mut games = Vec::new();
        let manifests_dir = match Self::find_manifests_dir() {
            Some(dir) => dir,
            None => return games,
        };

        if let Ok(entries) = fs::read_dir(manifests_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("item") {
                    if let Some(game) = Self::parse_item_file(&path) {
                        games.push(game);
                    }
                }
            }
        }

        games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        games
    }

    fn parse_item_file(path: &Path) -> Option<Game> {
        let content = fs::read_to_string(path).ok()?;
        let manifest: EpicManifest = serde_json::from_str(&content).ok()?;

        // Skip incomplete installations
        if manifest.is_incomplete_install.unwrap_or(false) {
            return None;
        }

        // Skip DLCs / Addons where main game is different
        if let Some(main_app) = &manifest.main_game_app_name {
            if !main_app.trim().is_empty() {
                return None;
            }
        }

        let title = manifest.display_name?.trim().to_string();
        let app_name = manifest.app_name?.trim().to_string();

        if title.is_empty() || app_name.is_empty() {
            return None;
        }

        // Filter out non-game software like Unreal Engine editors
        if title.starts_with("Unreal Engine") || app_name == "Launcher" {
            return None;
        }

        // Epic URI format: com.epicgames.launcher://apps/<CatalogNamespace>%3A<CatalogItemId>%3A<AppName>?action=launch&silent=true
        let app_identifier = if let (Some(ns), Some(item_id)) = (&manifest.catalog_namespace, &manifest.catalog_item_id) {
            if !ns.is_empty() && !item_id.is_empty() {
                format!("{}%3A{}%3A{}", ns, item_id, app_name)
            } else {
                app_name.clone()
            }
        } else {
            app_name.clone()
        };

        let launch_uri = format!("com.epicgames.launcher://apps/{}?action=launch&silent=true", app_identifier);
        let mut game = Game::new(&app_name, title, Platform::EpicGames, launch_uri);

        if let Some(install_dir) = manifest.install_location {
            let p = PathBuf::from(install_dir);
            if p.exists() {
                if let Some(exe_name) = &manifest.launch_executable {
                    let full_exe = p.join(exe_name);
                    if full_exe.exists() {
                        game.executable_path = Some(full_exe);
                    }
                }
                game.install_dir = Some(p);
            }
        }

        Some(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_epic_games() {
        println!("Epic launcher path: {:?}", EpicScanner::find_epic_launcher_exe());
        println!("Epic is running: {}", EpicScanner::is_launcher_running());

        let games = EpicScanner::scan_installed_games();
        println!("Epic Games encontradas: {}", games.len());
        for game in &games {
            println!("- {} (Exe: {:?})", game.title, game.executable_path);
        }
    }
}

