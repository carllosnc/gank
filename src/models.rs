use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Steam,
    EpicGames,
    Gog,
    Xbox,
    Custom,
}

impl Platform {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::EpicGames => "Epic Games",
            Self::Gog => "GOG Galaxy",
            Self::Xbox => "Xbox / MS Store",
            Self::Custom => "Personalizado",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub platform: Platform,
    pub launch_uri: String,
    pub install_dir: Option<PathBuf>,
    pub executable_path: Option<PathBuf>,
    pub cover_image_path: Option<PathBuf>,
    pub install_size: Option<u64>,
    pub last_played: Option<u64>,
    pub version: Option<String>,
}

impl Game {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        platform: Platform,
        launch_uri: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            platform,
            launch_uri: launch_uri.into(),
            install_dir: None,
            executable_path: None,
            cover_image_path: None,
            install_size: None,
            last_played: None,
            version: None,
        }
    }

    pub fn formatted_size(&self) -> Option<String> {
        self.install_size.map(|bytes| {
            const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
            const MIB: f64 = 1024.0 * 1024.0;
            let b = bytes as f64;
            if b >= GIB {
                format!("{:.1} GB", b / GIB)
            } else if b >= MIB {
                format!("{:.0} MB", b / MIB)
            } else {
                format!("{} KB", bytes / 1024)
            }
        })
    }

    pub fn formatted_last_played(&self) -> String {
        match self.last_played {
            None | Some(0) => "Nunca jogado".to_string(),
            Some(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if ts > now {
                    return "Recentemente".to_string();
                }
                let diff_secs = now - ts;
                let days = diff_secs / 86400;
                if days == 0 {
                    "Jogado hoje".to_string()
                } else if days == 1 {
                    "Jogado ontem".to_string()
                } else if days < 30 {
                    format!("Há {} dias", days)
                } else if days < 365 {
                    format!("Há {} meses", days / 30)
                } else {
                    format!("Há {} anos", days / 365)
                }
            }
        }
    }
}
