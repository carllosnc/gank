use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub cover_image_path: Option<PathBuf>,
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
            cover_image_path: None,
        }
    }
}
