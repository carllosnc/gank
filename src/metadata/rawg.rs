use super::SystemRequirements;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgSearchResponse {
    results: Vec<RawgGameSummary>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgGameSummary {
    id: u64,
    slug: String,
    name: String,
    metacritic: Option<u8>,
    released: Option<String>,
    background_image: Option<String>,
    short_screenshots: Option<Vec<RawgScreenshot>>,
    genres: Option<Vec<RawgNamedItem>>,
    platforms: Option<Vec<RawgPlatformWrapper>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgScreenshot {
    id: u64,
    image: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgNamedItem {
    name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgPlatformWrapper {
    platform: RawgPlatformDetails,
    requirements: Option<RawgRequirements>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgPlatformDetails {
    id: u64,
    name: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgRequirements {
    minimum: Option<String>,
    recommended: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawgGameDetail {
    id: u64,
    description_raw: Option<String>,
    developers: Option<Vec<RawgNamedItem>>,
    publishers: Option<Vec<RawgNamedItem>>,
}

/// Helper client for querying RAWG Video Games Database API.
pub struct RawgClient {
    api_key: Option<String>,
}

impl RawgClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    /// Searches RAWG for a game title and returns its structured metadata.
    pub fn fetch_game_info(&self, title: &str) -> Option<super::GameMetadata> {
        let api_key = self.api_key.as_deref()?;
        let encoded_title = urlencoding(title);
        let url = format!(
            "https://api.rawg.io/api/games?key={}&search={}&page_size=1&search_precise=true",
            api_key, encoded_title
        );

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(6))
            .build();

        let response: RawgSearchResponse = agent
            .get(&url)
            .set("User-Agent", "GankLauncher/0.1.0")
            .call()
            .ok()?
            .into_json()
            .ok()?;

        let item = response.results.into_iter().next()?;
        let mut meta = super::GameMetadata::new(title, &item.name);
        meta.rating = item.metacritic;
        meta.release_date = item.released;
        meta.hero_banner_url = item.background_image;

        if let Some(genres) = item.genres {
            meta.genres = genres.into_iter().map(|g| g.name).collect();
        }

        if let Some(shots) = item.short_screenshots {
            meta.screenshots = shots.into_iter().map(|s| s.image).collect();
        }

        // Extract PC requirements if present
        if let Some(plats) = item.platforms {
            for p in plats {
                if p.platform.slug == "pc" {
                    if let Some(reqs) = p.requirements {
                        if let Some(min_str) = reqs.minimum {
                            meta.req_minimum = Some(parse_requirements_text(&min_str));
                        }
                        if let Some(rec_str) = reqs.recommended {
                            meta.req_recommended = Some(parse_requirements_text(&rec_str));
                        }
                    }
                    break;
                }
            }
        }

        // Fetch deep details (developer, publisher, synopsis) using the game ID
        let detail_url = format!("https://api.rawg.io/api/games/{}?key={}", item.id, api_key);

        if let Ok(detail_resp) = agent
            .get(&detail_url)
            .set("User-Agent", "GankLauncher/0.1.0")
            .call()
        {
            if let Ok(detail) = detail_resp.into_json::<RawgGameDetail>() {
                meta.summary = detail.description_raw;
                if let Some(devs) = detail.developers {
                    meta.developer = devs.into_iter().map(|d| d.name).next();
                }
                if let Some(pubs) = detail.publishers {
                    meta.publisher = pubs.into_iter().map(|p| p.name).next();
                }
            }
        }

        Some(meta)
    }
}

/// Parses unstructured minimum / recommended text into SystemRequirements fields.
fn parse_requirements_text(text: &str) -> SystemRequirements {
    let mut req = SystemRequirements::default();
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("os:") || lower.contains("sistema:") {
            req.os = extract_after_colon(line);
        } else if lower.contains("processor:")
            || lower.contains("processador:")
            || lower.contains("cpu:")
        {
            req.processor = extract_after_colon(line);
        } else if lower.contains("memory:") || lower.contains("memória:") || lower.contains("ram:")
        {
            req.memory = extract_after_colon(line);
        } else if lower.contains("graphics:") || lower.contains("vídeo:") || lower.contains("gpu:")
        {
            req.graphics = extract_after_colon(line);
        } else if lower.contains("storage:")
            || lower.contains("armazenamento:")
            || lower.contains("disco:")
        {
            req.storage = extract_after_colon(line);
        } else if lower.contains("directx:") {
            req.directx = extract_after_colon(line);
        }
    }
    req
}

fn extract_after_colon(s: &str) -> Option<String> {
    s.split_once(':').map(|(_, val)| val.trim().to_string())
}

fn urlencoding(input: &str) -> String {
    let mut result = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char)
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", b));
            }
        }
    }
    result
}
