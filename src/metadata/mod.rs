pub mod rawg;
pub mod steam_store;
pub mod steamgrid;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::AppConfig;
use crate::models::Platform;
pub use crate::models::SystemRequirements;

/// High-level function that enriches a game's metadata using the best available source.
/// 1. If already cached locally in %LOCALAPPDATA%/gank/metadata/<id>.json, returns that immediately.
/// 2. If it's Steam, queries Steam Storefront API (returns specs, rating, genres, screens, trailers without API key).
/// 3. If RAWG key is configured, queries RAWG for specs, description, genres, ratings.
/// 4. If SteamGridDB key is configured, queries SGDB for posters, logos and background hero art.
/// 5. Automatically caches the combined result to disk for subsequent launches.
pub fn enrich_and_cache_game(
    game_id: &str,
    title: &str,
    platform: Platform,
    config: &AppConfig,
) -> Option<GameMetadata> {
    // 1. Check local cache first
    if let Some(cached) = load_cached_metadata(game_id) {
        return Some(cached);
    }

    let mut result_meta: Option<GameMetadata> = None;

    // 2. Try Steam Storefront API first if it's Steam or has a Steam App ID
    if platform == Platform::Steam {
        if let Some(steam_meta) = steam_store::fetch_steam_store_metadata(game_id, title) {
            result_meta = Some(steam_meta);
        }
    }

    // 3. If still missing full metadata or if we want RAWG enrichment
    if result_meta.is_none() && config.rawg_api_key.is_some() {
        let rawg = rawg::RawgClient::new(config.rawg_api_key.clone());
        if let Some(rawg_meta) = rawg.fetch_game_info(title) {
            result_meta = Some(rawg_meta);
        }
    }

    // 4. SteamGridDB fallback/enrichment for logos & hero banners
    if let Some(ref mut meta) = result_meta {
        if config.steamgriddb_api_key.is_some()
            && (meta.hero_banner_url.is_none() || meta.logo_url.is_none())
        {
            let sgdb = steamgrid::SteamGridDbClient::new(config.steamgriddb_api_key.clone());
            let steam_id = if platform == Platform::Steam {
                game_id.parse::<u64>().ok()
            } else {
                None
            };
            if let Some(bundle) = sgdb.fetch_art(title, steam_id) {
                if meta.hero_banner_url.is_none() {
                    meta.hero_banner_url = bundle.hero_url;
                }
                if meta.logo_url.is_none() {
                    meta.logo_url = bundle.logo_url;
                }
            }
        }
    }

    // 5. Persist to local disk cache
    if let Some(ref mut meta) = result_meta {
        meta.game_id = game_id.to_string();
        let _ = save_cached_metadata(meta);
    }

    result_meta
}

/// Rich metadata for a game fetched from RAWG, SteamGridDB, or local cache.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetadata {
    /// Associated game identifier (e.g. Steam App ID or Epic App Name)
    pub game_id: String,
    /// Canonical title
    pub title: String,
    /// Detailed or short synopsis
    pub summary: Option<String>,
    /// Metacritic or community rating score (0 - 100)
    pub rating: Option<u8>,
    /// Release date string (e.g., "2020-12-10")
    pub release_date: Option<String>,
    /// Developer / Studio
    pub developer: Option<String>,
    /// Publisher
    pub publisher: Option<String>,
    /// Genre tags (e.g. ["Action", "RPG", "Open World"])
    pub genres: Vec<String>,
    /// Minimum PC requirements
    pub req_minimum: Option<SystemRequirements>,
    /// Recommended PC requirements
    pub req_recommended: Option<SystemRequirements>,
    /// Screenshot URLs or paths
    pub screenshots: Vec<String>,
    /// Hero/Banner art URL or local path
    pub hero_banner_url: Option<String>,
    /// Logo transparent PNG URL
    pub logo_url: Option<String>,
    /// Video trailer URL (e.g. mp4, webm or youtube link)
    pub trailer_url: Option<String>,
    /// Timestamp when this metadata was cached
    pub cached_at: u64,
}

impl GameMetadata {
    pub fn new(game_id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            game_id: game_id.into(),
            title: title.into(),
            summary: None,
            rating: None,
            release_date: None,
            developer: None,
            publisher: None,
            genres: Vec::new(),
            req_minimum: None,
            req_recommended: None,
            screenshots: Vec::new(),
            hero_banner_url: None,
            logo_url: None,
            trailer_url: None,
            cached_at: now,
        }
    }
}

/// Helper directory for storing game metadata JSON files (%LOCALAPPDATA%/gank/metadata).
pub fn get_metadata_cache_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("USERPROFILE")
                .map(|p| PathBuf::from(p).join("AppData").join("Local"))
                .unwrap_or_else(|_| PathBuf::from("."))
        });

    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let dir = base.join("gank").join("metadata");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Sanitizes an ID for filesystem safety.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Path to the metadata JSON file for a given game ID.
pub fn get_metadata_path(game_id: &str) -> PathBuf {
    let safe_id = sanitize_id(game_id);
    get_metadata_cache_dir().join(format!("{}.json", safe_id))
}

/// Checks if metadata exists on disk and is non-empty.
pub fn has_cached_metadata(game_id: &str) -> bool {
    let path = get_metadata_path(game_id);
    path.is_file() && path.metadata().map(|m| m.len() > 10).unwrap_or(false)
}

/// Loads cached metadata from disk if available.
pub fn load_cached_metadata(game_id: &str) -> Option<GameMetadata> {
    let path = get_metadata_path(game_id);
    if !path.is_file() {
        return None;
    }

    let contents = fs::read_to_string(&path).ok()?;
    serde_json::from_str::<GameMetadata>(&contents).ok()
}

/// Saves metadata to disk.
pub fn save_cached_metadata(meta: &GameMetadata) -> Result<(), std::io::Error> {
    let path = get_metadata_path(&meta.game_id);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(meta).map_err(std::io::Error::other)?;
    fs::write(path, json)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_store_metadata_fetching() {
        // App ID 894020 is Death's Door
        let meta = steam_store::fetch_steam_store_metadata("894020", "Death's Door");
        assert!(meta.is_some(), "Deveria encontrar dados na Steam Store API");
        let m = meta.unwrap();
        println!("Metadata fetched para Death's Door:");
        println!(
            "- Resumo: {:?}",
            m.summary.as_ref().map(|s| &s[..s.len().min(60)])
        );
        println!("- Gêneros: {:?}", m.genres);
        println!("- Metacritic: {:?}", m.rating);
        println!("- Release Date: {:?}", m.release_date);
        println!("- Screenshots: {} fotos", m.screenshots.len());
        println!("- Minimum Specs: {:?}", m.req_minimum);

        assert!(!m.genres.is_empty(), "Deveria conter gêneros");
        assert!(!m.screenshots.is_empty(), "Deveria ter screenshots");
    }
}
