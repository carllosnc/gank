use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SteamGridDbResponse<T> {
    success: bool,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SgdbGameItem {
    id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SgdbImageItem {
    id: u64,
    url: String,
    thumb: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

/// Helper client for SteamGridDB API v2.
pub struct SteamGridDbClient {
    api_key: Option<String>,
}

pub struct SgdbArtBundle {
    pub poster_url: Option<String>,
    pub hero_url: Option<String>,
    pub logo_url: Option<String>,
}

impl SteamGridDbClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    /// Fetches covers, heroes and logos for a game by title or steam app id.
    pub fn fetch_art(&self, title: &str, steam_id: Option<u64>) -> Option<SgdbArtBundle> {
        let api_key = self.api_key.as_deref()?;
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(6))
            .build();

        // 1. Resolve SGDB Game ID
        let sgdb_game_id: u64 = if let Some(sid) = steam_id {
            let url = format!("https://www.steamgriddb.com/api/v2/games/steam/{}", sid);
            if let Ok(resp) = agent
                .get(&url)
                .set("Authorization", &format!("Bearer {}", api_key))
                .call()
            {
                resp.into_json::<SteamGridDbResponse<SgdbGameItem>>()
                    .ok()
                    .and_then(|r| r.data)
                    .map(|d| d.id)?
            } else {
                self.search_game_id(&agent, api_key, title)?
            }
        } else {
            self.search_game_id(&agent, api_key, title)?
        };

        // 2. Fetch vertical grid poster (600x900)
        let poster_url = self.fetch_top_image(
            &agent,
            api_key,
            &format!(
                "https://www.steamgriddb.com/api/v2/grids/game/{}?dimensions=600x900",
                sgdb_game_id
            ),
        );

        // 3. Fetch hero banner (background)
        let hero_url = self.fetch_top_image(
            &agent,
            api_key,
            &format!(
                "https://www.steamgriddb.com/api/v2/heroes/game/{}",
                sgdb_game_id
            ),
        );

        // 4. Fetch transparent logo
        let logo_url = self.fetch_top_image(
            &agent,
            api_key,
            &format!(
                "https://www.steamgriddb.com/api/v2/logos/game/{}",
                sgdb_game_id
            ),
        );

        Some(SgdbArtBundle {
            poster_url,
            hero_url,
            logo_url,
        })
    }

    fn search_game_id(&self, agent: &ureq::Agent, api_key: &str, title: &str) -> Option<u64> {
        let url = format!(
            "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
            urlencoding(title)
        );
        let resp = agent
            .get(&url)
            .set("Authorization", &format!("Bearer {}", api_key))
            .call()
            .ok()?
            .into_json::<SteamGridDbResponse<Vec<SgdbGameItem>>>()
            .ok()?;

        resp.data?.into_iter().next().map(|g| g.id)
    }

    fn fetch_top_image(
        &self,
        agent: &ureq::Agent,
        api_key: &str,
        endpoint: &str,
    ) -> Option<String> {
        let resp = agent
            .get(endpoint)
            .set("Authorization", &format!("Bearer {}", api_key))
            .call()
            .ok()?
            .into_json::<SteamGridDbResponse<Vec<SgdbImageItem>>>()
            .ok()?;

        resp.data?.into_iter().next().map(|i| i.url)
    }
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
