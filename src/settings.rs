use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub recording_enabled: bool,
    pub retention_days: Option<u32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            recording_enabled: true,
            retention_days: None,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".hindsight").join("settings.json"))
}

pub fn load() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };

    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}
