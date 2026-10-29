use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub rawg_api_key: Option<String>,
    pub steamgriddb_api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            rawg_api_key: std::env::var("RAWG_API_KEY").ok(),
            steamgriddb_api_key: std::env::var("STEAMGRIDDB_API_KEY").ok(),
        }
    }
}

pub fn get_config_dir() -> PathBuf {
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
        .map(|h| PathBuf::from(h).join(".config"))
        .unwrap_or_else(|_| PathBuf::from("."));

    let dir = base.join("gank");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

impl AppConfig {
    pub fn load() -> Self {
        let path = get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }

        let default_cfg = AppConfig::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = get_config_path();
        let content = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, content)
    }
}
