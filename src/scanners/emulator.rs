use crate::models::{Game, Platform};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct EmulatorScanner;

#[derive(Debug, Clone)]
struct KnownEmulator {
    id: &'static str,
    name: &'static str,
    executable_names: &'static [&'static str],
    candidate_paths: &'static [&'static str],
    supported_extensions: &'static [&'static str],
}

const KNOWN_EMULATORS: &[KnownEmulator] = &[
    KnownEmulator {
        id: "retroarch",
        name: "RetroArch",
        executable_names: &["retroarch.exe"],
        candidate_paths: &[
            r"C:\RetroArch-Win64\retroarch.exe",
            r"C:\Program Files\RetroArch\retroarch.exe",
            r"C:\Program Files (x86)\Steam\steamapps\common\RetroArch\retroarch.exe",
            r"D:\SteamLibrary\steamapps\common\RetroArch\retroarch.exe",
            r"D:\RetroArch\retroarch.exe",
        ],
        supported_extensions: &["nes", "sfc", "smc", "gba", "gb", "gbc", "n64", "z64", "chd"],
    },
    KnownEmulator {
        id: "rpcs3",
        name: "RPCS3 (PlayStation 3)",
        executable_names: &["rpcs3.exe"],
        candidate_paths: &[
            r"C:\RPCS3\rpcs3.exe",
            r"C:\Program Files\RPCS3\rpcs3.exe",
            r"D:\RPCS3\rpcs3.exe",
            r"E:\RPCS3\rpcs3.exe",
        ],
        supported_extensions: &["pkg"],
    },
    KnownEmulator {
        id: "dolphin",
        name: "Dolphin (GameCube / Wii)",
        executable_names: &["Dolphin.exe", "DolphinWx.exe"],
        candidate_paths: &[
            r"C:\Program Files\Dolphin\Dolphin.exe",
            r"C:\Dolphin\Dolphin.exe",
            r"D:\Dolphin\Dolphin.exe",
        ],
        supported_extensions: &["rvz", "gcm", "wbfs", "cso"],
    },
    KnownEmulator {
        id: "pcsx2",
        name: "PCSX2 (PlayStation 2)",
        executable_names: &["pcsx2-qt.exe", "pcsx2-qtx64.exe", "pcsx2.exe"],
        candidate_paths: &[
            r"C:\Program Files\PCSX2\pcsx2-qt.exe",
            r"C:\Program Files (x86)\PCSX2\pcsx2.exe",
            r"C:\PCSX2\pcsx2-qt.exe",
            r"D:\PCSX2\pcsx2-qt.exe",
        ],
        supported_extensions: &["iso", "chd", "cso"],
    },
    KnownEmulator {
        id: "ppsspp",
        name: "PPSSPP (PSP)",
        executable_names: &["PPSSPPWindows64.exe", "PPSSPPWindows.exe"],
        candidate_paths: &[
            r"C:\Program Files\PPSSPP\PPSSPPWindows64.exe",
            r"C:\PPSSPP\PPSSPPWindows64.exe",
            r"D:\PPSSPP\PPSSPPWindows64.exe",
        ],
        supported_extensions: &["cso", "iso"],
    },
    KnownEmulator {
        id: "duckstation",
        name: "DuckStation (PlayStation 1)",
        executable_names: &["duckstation-qt-x64-ReleaseLTCG.exe", "duckstation-nogui-x64-ReleaseLTCG.exe"],
        candidate_paths: &[
            r"C:\DuckStation\duckstation-qt-x64-ReleaseLTCG.exe",
            r"C:\Program Files\DuckStation\duckstation-qt-x64-ReleaseLTCG.exe",
            r"D:\DuckStation\duckstation-qt-x64-ReleaseLTCG.exe",
        ],
        supported_extensions: &["chd", "cue", "iso"],
    },
    KnownEmulator {
        id: "cemu",
        name: "Cemu (Wii U)",
        executable_names: &["Cemu.exe"],
        candidate_paths: &[
            r"C:\Cemu\Cemu.exe",
            r"C:\Program Files\Cemu\Cemu.exe",
            r"D:\Cemu\Cemu.exe",
        ],
        supported_extensions: &["wua", "rpx"],
    },
    KnownEmulator {
        id: "ryujinx",
        name: "Ryujinx (Switch)",
        executable_names: &["Ryujinx.exe"],
        candidate_paths: &[
            r"C:\Ryujinx\Ryujinx.exe",
            r"D:\Ryujinx\Ryujinx.exe",
        ],
        supported_extensions: &["nsp", "xci"],
    },
];

impl EmulatorScanner {
    /// Detects installed standalone emulators and discovered ROMs.
    pub fn scan_installed_games() -> Vec<Game> {
        let mut results = Vec::new();
        let mut detected_emulators: Vec<(KnownEmulator, PathBuf)> = Vec::new();
        let mut seen_ids = HashSet::new();

        // 1. Scan for installed emulator executables
        for emu in KNOWN_EMULATORS {
            if let Some(exe_path) = Self::find_emulator_binary(emu) {
                detected_emulators.push((emu.clone(), exe_path.clone()));

                let id = format!("emu_{}", emu.id);
                if !seen_ids.contains(&id) {
                    seen_ids.insert(id.clone());

                    let mut game = Game::new(
                        id,
                        emu.name,
                        Platform::Emulator,
                        exe_path.to_string_lossy().to_string(),
                    );
                    game.executable_path = Some(exe_path.clone());
                    game.install_dir = exe_path.parent().map(|p| p.to_path_buf());
                    results.push(game);
                }
            }
        }

        // 2. Scan standard ROM directories for individual games
        let rom_directories = Self::get_standard_rom_dirs();
        for rom_dir in rom_directories {
            if rom_dir.exists() && rom_dir.is_dir() {
                Self::scan_rom_directory(&rom_dir, &detected_emulators, &mut seen_ids, &mut results);
            }
        }

        results.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        results
    }

    /// Finds candidate paths for a given emulator.
    fn find_emulator_binary(emu: &KnownEmulator) -> Option<PathBuf> {
        // 1. Check AppData
        if let Ok(app_data) = std::env::var("APPDATA") {
            let app_data_dir = PathBuf::from(app_data).join(emu.id);
            for exe_name in emu.executable_names {
                let p = app_data_dir.join(exe_name);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // 2. Check Candidate absolute paths
        for candidate in emu.candidate_paths {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    /// Returns standard folders where users typically store ROMs on Windows.
    fn get_standard_rom_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Ok(profile) = std::env::var("USERPROFILE") {
            let p = PathBuf::from(profile);
            dirs.push(p.join("Roms"));
            dirs.push(p.join("Games").join("Roms"));
            dirs.push(p.join("Emulators").join("Roms"));
            dirs.push(p.join("Documents").join("Roms"));
        }

        for drive in ["C:\\", "D:\\", "E:\\", "F:\\"] {
            dirs.push(PathBuf::from(drive).join("Roms"));
            dirs.push(PathBuf::from(drive).join("Games").join("Roms"));
        }

        dirs
    }

    /// Scans a directory for ROM files and matches them with available emulators.
    fn scan_rom_directory(
        dir: &Path,
        emulators: &[(KnownEmulator, PathBuf)],
        seen_ids: &mut HashSet<String>,
        results: &mut Vec<Game>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if Self::is_rom_extension(&ext_lower) {
                        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        let title = Self::clean_rom_title(file_stem);
                        let id = format!("rom_{}_{}", ext_lower, title.to_lowercase().replace(' ', "_"));

                        if !seen_ids.contains(&id) && !title.is_empty() {
                            seen_ids.insert(id.clone());

                            // Find best matching emulator for this extension
                            let matching_emu = emulators.iter().find(|(emu, _)| {
                                emu.supported_extensions.contains(&ext_lower.as_str())
                            });

                            let launch_uri = if let Some((_, exe_path)) = matching_emu {
                                format!("\"{}\" \"{}\"", exe_path.to_string_lossy(), path.to_string_lossy())
                            } else {
                                path.to_string_lossy().to_string()
                            };

                            let mut game = Game::new(
                                id,
                                title,
                                Platform::Emulator,
                                launch_uri,
                            );

                            game.install_dir = path.parent().map(|p| p.to_path_buf());
                            game.install_size = path.metadata().ok().map(|m| m.len());
                            results.push(game);
                        }
                    }
                }
            } else if path.is_dir() {
                // One level of subfolder recursion (e.g. Roms/PS2, Roms/SNES)
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if sub_path.is_file() {
                            if let Some(ext) = sub_path.extension().and_then(|e| e.to_str()) {
                                let ext_lower = ext.to_lowercase();
                                if Self::is_rom_extension(&ext_lower) {
                                    let file_stem = sub_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                                    let title = Self::clean_rom_title(file_stem);
                                    let id = format!("rom_{}_{}", ext_lower, title.to_lowercase().replace(' ', "_"));

                                    if !seen_ids.contains(&id) && !title.is_empty() {
                                        seen_ids.insert(id.clone());

                                        let matching_emu = emulators.iter().find(|(emu, _)| {
                                            emu.supported_extensions.contains(&ext_lower.as_str())
                                        });

                                        let launch_uri = if let Some((_, exe_path)) = matching_emu {
                                            format!("\"{}\" \"{}\"", exe_path.to_string_lossy(), sub_path.to_string_lossy())
                                        } else {
                                            sub_path.to_string_lossy().to_string()
                                        };

                                        let mut game = Game::new(
                                            id,
                                            title,
                                            Platform::Emulator,
                                            launch_uri,
                                        );

                                        game.install_dir = sub_path.parent().map(|p| p.to_path_buf());
                                        game.install_size = sub_path.metadata().ok().map(|m| m.len());
                                        results.push(game);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn is_rom_extension(ext: &str) -> bool {
        matches!(
            ext,
            "iso" | "chd" | "rvz" | "cso" | "wbfs" | "nes" | "sfc" | "smc" | "gba" | "gb"
                | "gbc" | "n64" | "z64" | "pkg" | "nsp" | "xci" | "wua"
        )
    }

    /// Cleans standard dump tags from ROM titles (e.g. "Super Mario World (USA) [!].sfc" -> "Super Mario World")
    fn clean_rom_title(raw: &str) -> String {
        let mut clean = String::new();
        let mut in_paren = false;
        let mut in_bracket = false;

        for c in raw.chars() {
            if c == '(' {
                in_paren = true;
            } else if c == ')' {
                in_paren = false;
            } else if c == '[' {
                in_bracket = true;
            } else if c == ']' {
                in_bracket = false;
            } else if !in_paren && !in_bracket {
                clean.push(c);
            }
        }

        clean.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_scanner_runs_without_panic() {
        let games = EmulatorScanner::scan_installed_games();
        println!("Discovered {} emulator/ROM items", games.len());
    }

    #[test]
    fn test_clean_rom_title() {
        assert_eq!(
            EmulatorScanner::clean_rom_title("Super Mario World (USA) [!]"),
            "Super Mario World"
        );
        assert_eq!(
            EmulatorScanner::clean_rom_title("The Legend of Zelda - Ocarina of Time (Europe) (En,Fr,De)"),
            "The Legend of Zelda - Ocarina of Time"
        );
    }

    #[test]
    fn test_is_rom_extension() {
        assert!(EmulatorScanner::is_rom_extension("iso"));
        assert!(EmulatorScanner::is_rom_extension("chd"));
        assert!(EmulatorScanner::is_rom_extension("rvz"));
        assert!(!EmulatorScanner::is_rom_extension("txt"));
        assert!(!EmulatorScanner::is_rom_extension("exe"));
    }
}
