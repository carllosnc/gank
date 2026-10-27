use crate::models::Platform;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Returns the persistent local directory for cached game cover posters.
pub fn get_covers_cache_dir() -> PathBuf {
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
        .map(|h| PathBuf::from(h).join(".cache"))
        .unwrap_or_else(|_| PathBuf::from("."));

    let dir = base.join("kiro").join("covers");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Sanitizes a string for use as a safe filename.
pub fn sanitize_filename(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

/// Checks if a cached cover image already exists on disk for the given ID.
pub fn get_cached_cover(id: &str) -> Option<PathBuf> {
    let safe_id = sanitize_filename(id);
    let cache_dir = get_covers_cache_dir();
    let file = cache_dir.join(format!("{}.jpg", safe_id));
    if file.exists() && file.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
        Some(file)
    } else {
        None
    }
}

/// Fetches and caches a cover image for a game using Epic Store Content CDN or Steam Community Search.
pub fn fetch_and_cache_cover(id: &str, title: &str, platform: Platform) -> Option<PathBuf> {
    // 1. Check local cache first
    if let Some(cached) = get_cached_cover(id) {
        return Some(cached);
    }

    let cache_dir = get_covers_cache_dir();
    let safe_id = sanitize_filename(id);
    let target_file = cache_dir.join(format!("{}.jpg", safe_id));

    // 2. If it's Epic Games, try Epic Store Content CDN first
    if platform == Platform::EpicGames {
        if let Some(bytes) = try_fetch_epic_cover(title) {
            if fs::write(&target_file, &bytes).is_ok() {
                return Some(target_file);
            }
        }
    }

    // 3. Fallback to Steam Community Search (matches 99% of PC games)
    if let Some(bytes) = try_fetch_steam_cover_by_search(title) {
        if fs::write(&target_file, &bytes).is_ok() {
            return Some(target_file);
        }
    }

    None
}

/// Attempts to fetch vertical cover art from Epic Games Store static content CDN.
fn try_fetch_epic_cover(title: &str) -> Option<Vec<u8>> {
    let slugs = generate_epic_slugs(title);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();

    for slug in slugs {
        let api_url = format!(
            "https://store-content-ipv4.ak.epicgames.com/api/en-US/content/products/{}",
            slug
        );

        if let Ok(resp) = agent.get(&api_url).call() {
            if resp.status() == 200 {
                if let Ok(json_val) = resp.into_json::<serde_json::Value>() {
                    // Try to find portraitBackgroundImageUrl or 1200x1600 images
                    if let Some(image_url) = extract_epic_portrait_url(&json_val) {
                        if let Ok(img_resp) = agent.get(&image_url).call() {
                            if img_resp.status() == 200 {
                                let mut bytes = Vec::new();
                                if std::io::copy(&mut img_resp.into_reader(), &mut bytes).is_ok() {
                                    if bytes.len() > 1000 {
                                        return Some(bytes);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Generates reasonable Epic store product slugs from game display titles.
fn generate_epic_slugs(title: &str) -> Vec<String> {
    let mut slugs = Vec::new();
    let lower = title.to_lowercase();

    // 1. Exact cleaned title
    let clean: String = lower
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect();
    let base_slug = clean.split_whitespace().collect::<Vec<_>>().join("-");
    if !base_slug.is_empty() {
        slugs.push(base_slug.clone());
    }

    // 2. Strip common editions / suffixes ("Enhanced", "Remastered", "Edition", "Special Edition")
    let stripped = lower
        .replace("enhanced", "")
        .replace("special edition", "")
        .replace("definitive edition", "")
        .replace("complete edition", "")
        .replace("remastered", "")
        .replace("standard edition", "");

    let clean_stripped: String = stripped
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect();
    let stripped_slug = clean_stripped.split_whitespace().collect::<Vec<_>>().join("-");
    if !stripped_slug.is_empty() && stripped_slug != base_slug {
        slugs.push(stripped_slug);
    }

    slugs
}

/// Extracts a portrait cover URL from Epic store product JSON metadata.
fn extract_epic_portrait_url(json: &serde_json::Value) -> Option<String> {
    // 1. Check pages[0].data.hero.portraitBackgroundImageUrl
    if let Some(pages) = json.get("pages").and_then(|p| p.as_array()) {
        for page in pages {
            if let Some(hero) = page.get("data").and_then(|d| d.get("hero")) {
                if let Some(url) = hero.get("portraitBackgroundImageUrl").and_then(|u| u.as_str()) {
                    if !url.is_empty() {
                        return Some(url.to_string());
                    }
                }
            }
        }
    }

    // 2. Fallback: check _images_ array for any image containing "1200x1600" or "s2"
    if let Some(pages) = json.get("pages").and_then(|p| p.as_array()) {
        for page in pages {
            if let Some(images) = page.get("_images_").and_then(|i| i.as_array()) {
                for img in images {
                    if let Some(url) = img.as_str() {
                        if url.contains("1200x1600") || url.contains("s2") {
                            return Some(url.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Fallback: Searches Steam Community and downloads `library_600x900.jpg` for the top matching appid.
fn try_fetch_steam_cover_by_search(title: &str) -> Option<Vec<u8>> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();

    // Clean title for search (remove special edition markers if needed)
    let clean_title = title
        .replace("Enhanced", "")
        .replace("Standard Edition", "")
        .trim()
        .to_string();

    let search_url = format!(
        "https://steamcommunity.com/actions/SearchApps/{}",
        urlencoding(&clean_title)
    );

    let resp = agent.get(&search_url).call().ok()?;
    if resp.status() != 200 {
        return None;
    }

    let search_results = resp.into_json::<serde_json::Value>().ok()?;
    let items = search_results.as_array()?;
    let first_item = items.first()?;
    let appid = first_item.get("appid")?.as_str()?;

    // Modern Steam vertical poster URL
    let poster_url = format!(
        "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
        appid
    );

    let img_resp = agent.get(&poster_url).call().ok()?;
    if img_resp.status() != 200 {
        return None;
    }

    let mut bytes = Vec::new();
    if std::io::copy(&mut img_resp.into_reader(), &mut bytes).is_ok() && bytes.len() > 1000 {
        Some(bytes)
    } else {
        None
    }
}

/// Simple percent-encoding for query params.
fn urlencoding(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_epic_chivalry() {
        let path = fetch_and_cache_cover("Peppermint", "Chivalry 2", Platform::EpicGames);
        assert!(path.is_some(), "Chivalry 2 cover should be fetched and cached");
        let p = path.unwrap();
        assert!(p.exists());
        assert!(p.metadata().unwrap().len() > 1000);
    }

    #[test]
    fn test_fetch_epic_gtav() {
        let path = fetch_and_cache_cover("8769e24080ea413b8ebca3f1b8c50951", "Grand Theft Auto V Enhanced", Platform::EpicGames);
        assert!(path.is_some(), "GTA V cover should be fetched and cached");
        let p = path.unwrap();
        assert!(p.exists());
        assert!(p.metadata().unwrap().len() > 1000);
    }
}

