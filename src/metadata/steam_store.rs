use std::time::Duration;
use crate::models::SystemRequirements;

#[derive(serde::Deserialize)]
struct SteamAppDetailsResponse {
    success: bool,
    data: Option<SteamAppDetailsData>,
}

#[derive(serde::Deserialize)]
struct SteamAppDetailsData {
    short_description: Option<String>,
    header_image: Option<String>,
    developers: Option<Vec<String>>,
    publishers: Option<Vec<String>>,
    genres: Option<Vec<SteamGenre>>,
    metacritic: Option<SteamMetacritic>,
    pc_requirements: Option<serde_json::Value>,
    screenshots: Option<Vec<SteamScreenshot>>,
    movies: Option<Vec<SteamMovie>>,
    release_date: Option<SteamReleaseDate>,
}

#[derive(serde::Deserialize)]
struct SteamGenre {
    description: String,
}

#[derive(serde::Deserialize)]
struct SteamMetacritic {
    score: u8,
}

#[derive(serde::Deserialize)]
struct SteamScreenshot {
    path_full: String,
}

#[derive(serde::Deserialize)]
struct SteamMovie {
    mp4: Option<SteamMovieFormat>,
    webm: Option<SteamMovieFormat>,
}

#[derive(serde::Deserialize)]
struct SteamMovieFormat {
    max: Option<String>,
    #[serde(rename = "480")]
    sd: Option<String>,
}

#[derive(serde::Deserialize)]
struct SteamReleaseDate {
    date: String,
}

/// Fetches official metadata directly from Steam Storefront API (Zero API key required!).
pub fn fetch_steam_store_metadata(app_id: &str, title: &str) -> Option<crate::metadata::GameMetadata> {
    let url = format!(
        "https://store.steampowered.com/api/appdetails?appids={}&l=brazilian",
        app_id
    );

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .build();

    let json_val: serde_json::Value = agent
        .get(&url)
        .set("User-Agent", "Mozilla/5.0")
        .call()
        .ok()?
        .into_json()
        .ok()?;

    let root_obj = json_val.as_object()?;
    let app_val = root_obj.get(app_id)?;
    let resp: SteamAppDetailsResponse = serde_json::from_value(app_val.clone()).ok()?;

    if !resp.success {
        return None;
    }

    let data = resp.data?;
    let mut meta = crate::metadata::GameMetadata::new(app_id, title);
    meta.summary = data.short_description;
    meta.hero_banner_url = data.header_image;
    meta.rating = data.metacritic.map(|m| m.score);
    meta.release_date = data.release_date.map(|r| r.date);

    if let Some(devs) = data.developers {
        meta.developer = devs.into_iter().next();
    }
    if let Some(pubs) = data.publishers {
        meta.publisher = pubs.into_iter().next();
    }
    if let Some(genres) = data.genres {
        meta.genres = genres.into_iter().map(|g| g.description).collect();
    }
    if let Some(shots) = data.screenshots {
        meta.screenshots = shots.into_iter().map(|s| s.path_full).collect();
    }
    if let Some(movies) = data.movies {
        if let Some(first) = movies.into_iter().next() {
            meta.trailer_url = first.mp4.and_then(|m| m.max.or(m.sd))
                .or_else(|| first.webm.and_then(|w| w.max.or(w.sd)));
        }
    }

    // Parse PC requirements if available
    if let Some(req_val) = data.pc_requirements {
        if let Some(obj) = req_val.as_object() {
            if let Some(min_html) = obj.get("minimum").and_then(|v| v.as_str()) {
                meta.req_minimum = Some(parse_html_requirements(min_html));
            }
            if let Some(rec_html) = obj.get("recommended").and_then(|v| v.as_str()) {
                meta.req_recommended = Some(parse_html_requirements(rec_html));
            }
        }
    }

    Some(meta)
}

fn parse_html_requirements(html: &str) -> SystemRequirements {
    let clean = html.replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<li>", "\n")
        .replace("</li>", "");

    // Strip remaining tags
    let mut text = String::new();
    let mut in_tag = false;
    for c in clean.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            text.push(c);
        }
    }

    let mut req = SystemRequirements::default();
    for line in text.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("so:") || lower.starts_with("os:") || lower.starts_with("sistema:") {
            req.os = extract_after_colon(trimmed);
        } else if lower.starts_with("processador:") || lower.starts_with("processor:") || lower.starts_with("cpu:") {
            req.processor = extract_after_colon(trimmed);
        } else if lower.starts_with("memória:") || lower.starts_with("memory:") || lower.starts_with("ram:") {
            req.memory = extract_after_colon(trimmed);
        } else if lower.starts_with("placa de vídeo:") || lower.starts_with("graphics:") || lower.starts_with("vídeo:") {
            req.graphics = extract_after_colon(trimmed);
        } else if lower.starts_with("armazenamento:") || lower.starts_with("storage:") || lower.starts_with("espaço no disco:") {
            req.storage = extract_after_colon(trimmed);
        } else if lower.starts_with("directx:") {
            req.directx = extract_after_colon(trimmed);
        }
    }
    req
}

fn extract_after_colon(s: &str) -> Option<String> {
    s.split_once(':').map(|(_, val)| val.trim().to_string())
}
