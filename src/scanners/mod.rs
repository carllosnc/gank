pub mod steam;

use crate::models::Game;

pub fn scan_all_platforms() -> Vec<Game> {
    let mut all_games = Vec::new();

    // 1. Scan Steam
    all_games.extend(steam::SteamScanner::scan_installed_games());

    // In future iterations:
    // all_games.extend(epic::EpicScanner::scan_installed_games());
    // all_games.extend(gog::GogScanner::scan_installed_games());
    // all_games.extend(xbox::XboxScanner::scan_installed_games());

    all_games
}
