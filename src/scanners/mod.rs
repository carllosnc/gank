pub mod battlenet;
pub mod ea;
pub mod emulator;
pub mod epic;
pub mod process;
pub mod steam;
pub mod ubisoft;

use crate::models::Game;

pub fn scan_all_platforms() -> Vec<Game> {
    let mut all_games = Vec::new();

    // 1. Scan Steam
    all_games.extend(steam::SteamScanner::scan_installed_games());

    // 2. Scan Epic Games
    all_games.extend(epic::EpicScanner::scan_installed_games());

    // 3. Scan Ubisoft Connect
    all_games.extend(ubisoft::UbisoftScanner::scan_installed_games());

    // 4. Scan EA App / Origin
    all_games.extend(ea::EaScanner::scan_installed_games());

    // 5. Scan Blizzard / Battle.net
    all_games.extend(battlenet::BattleNetScanner::scan_installed_games());

    // 6. Scan Emulators & ROMs
    all_games.extend(emulator::EmulatorScanner::scan_installed_games());

    // Index all cached covers from disk once for blazing fast O(1) matching
    let cached_covers_map = crate::artwork::get_all_cached_covers();
    for game in &mut all_games {
        if game.cover_image_path.is_none() {
            let safe_id = crate::artwork::sanitize_filename(&game.id);
            if let Some(path) = cached_covers_map
                .get(&safe_id)
                .or_else(|| cached_covers_map.get(&game.id))
            {
                game.set_cover_path(Some(path.clone()));
            }
        }
        if game.cover_uri.is_none() && game.cover_image_path.is_some() {
            if let Some(p) = &game.cover_image_path {
                game.cover_uri = Some(format!("file://{}", p.to_string_lossy().replace('\\', "/")));
            }
        }
    }

    // Sort all games alphabetically
    all_games.sort_by_key(|a| a.title.to_lowercase());

    all_games
}
