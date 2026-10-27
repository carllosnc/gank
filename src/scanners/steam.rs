use crate::models::{Game, Platform};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub struct SteamScanner;

impl SteamScanner {
    /// Checks if steam.exe is currently running in Windows.
    pub fn is_running() -> bool {
        crate::scanners::process::is_process_running("steam.exe")
    }

    /// Detects the full path to `steam.exe` via Windows registry or default locations.
    pub fn find_steam_exe() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(steam_key) = hkcu.open_subkey_with_flags(r"Software\Valve\Steam", KEY_READ) {
                if let Ok(exe_str) = steam_key.get_value::<String, _>("SteamExe") {
                    let path = PathBuf::from(exe_str.replace('/', "\\"));
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        if let Some(base) = Self::find_steam_install_path() {
            let exe = base.join("steam.exe");
            if exe.exists() {
                return Some(exe);
            }
        }

        let default_exe = PathBuf::from(r"C:\Program Files (x86)\Steam\steam.exe");
        if default_exe.exists() {
            Some(default_exe)
        } else {
            None
        }
    }

    /// Detects the base Steam installation path from the Windows registry.
    pub fn find_steam_install_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            // 1. Try HKCU\Software\Valve\Steam
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(steam_key) = hkcu.open_subkey_with_flags(r"Software\Valve\Steam", KEY_READ) {
                if let Ok(path_str) = steam_key.get_value::<String, _>("SteamPath") {
                    let path = PathBuf::from(path_str);
                    if path.exists() {
                        return Some(path);
                    }
                }
            }

            // 2. Try HKLM (64-bit and 32-bit fallback)
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for subkey in [
                r"SOFTWARE\WOW6432Node\Valve\Steam",
                r"SOFTWARE\Valve\Steam",
            ] {
                if let Ok(steam_key) = hklm.open_subkey_with_flags(subkey, KEY_READ) {
                    if let Ok(path_str) = steam_key.get_value::<String, _>("InstallPath") {
                        let path = PathBuf::from(path_str);
                        if path.exists() {
                            return Some(path);
                        }
                    }
                }
            }
        }

        // Common default fallback
        let default_path = PathBuf::from(r"C:\Program Files (x86)\Steam");
        if default_path.exists() {
            Some(default_path)
        } else {
            None
        }
    }

    /// Scans `libraryfolders.vdf` to discover all Steam library directories on all drives.
    pub fn find_library_folders(steam_path: &Path) -> Vec<PathBuf> {
        let mut libraries = Vec::new();
        libraries.push(steam_path.to_path_buf());

        let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
        if let Ok(content) = fs::read_to_string(&vdf_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("\"path\"") {
                    let parts: Vec<&str> = trimmed.split('"').collect();
                    if parts.len() >= 4 {
                        let dir = PathBuf::from(parts[3].replace(r"\\", r"\"));
                        if dir.exists() && !libraries.contains(&dir) {
                            libraries.push(dir);
                        }
                    }
                }
            }
        }

        libraries
    }

    /// Scans all library folders for installed `appmanifest_<appid>.acf` files.
    pub fn scan_installed_games() -> Vec<Game> {
        let mut games = Vec::new();
        let steam_path = match Self::find_steam_install_path() {
            Some(path) => path,
            None => return games,
        };

        let library_folders = Self::find_library_folders(&steam_path);
        let mut seen_ids = HashSet::new();

        for lib_path in library_folders {
            let steamapps = lib_path.join("steamapps");
            if let Ok(entries) = fs::read_dir(&steamapps) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();

                    if name_str.starts_with("appmanifest_") && name_str.ends_with(".acf") {
                        if let Some(game) = Self::parse_manifest(&entry.path(), &lib_path) {
                            // Filter out Steamworks Common Redistributables and common tools
                            if game.id != "228980" && !seen_ids.contains(&game.id) {
                                seen_ids.insert(game.id.clone());
                                games.push(game);
                            }
                        }
                    }
                }
            }
        }

        // Sort alphabetically by title
        games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        games
    }

    fn parse_manifest(manifest_path: &Path, library_root: &Path) -> Option<Game> {
        let content = fs::read_to_string(manifest_path).ok()?;

        let mut app_id = None;
        let mut name = None;
        let mut install_dir_name = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\"appid\"") {
                let parts: Vec<&str> = trimmed.split('"').collect();
                if parts.len() >= 4 {
                    app_id = Some(parts[3].to_string());
                }
            } else if trimmed.starts_with("\"name\"") {
                let parts: Vec<&str> = trimmed.split('"').collect();
                if parts.len() >= 4 {
                    name = Some(parts[3].to_string());
                }
            } else if trimmed.starts_with("\"installdir\"") {
                let parts: Vec<&str> = trimmed.split('"').collect();
                if parts.len() >= 4 {
                    install_dir_name = Some(parts[3].to_string());
                }
            }
        }

        if let (Some(id), Some(title)) = (app_id, name) {
            let launch_uri = format!("steam://rungameid/{}", id);
            let mut game = Game::new(&id, title, Platform::Steam, launch_uri);

            if let Some(folder) = install_dir_name {
                let full_install_path = library_root
                    .join("steamapps")
                    .join("common")
                    .join(folder);
                game.install_dir = Some(full_install_path);
            }

            Some(game)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_detection() {
        println!("Steam install path: {:?}", SteamScanner::find_steam_install_path());
        println!("Steam exe: {:?}", SteamScanner::find_steam_exe());
        println!("Steam is running: {}", SteamScanner::is_running());

        let games = SteamScanner::scan_installed_games();
        println!("Steam games encontradas: {}", games.len());
        for game in games.iter().take(5) {
            println!("- {} ({})", game.title, game.launch_uri);
        }
    }
}
