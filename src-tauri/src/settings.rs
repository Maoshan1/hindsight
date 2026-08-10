use anyhow::Result;
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

fn settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot find home dir"))?;
    let dir = home.join(".hindsight");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("settings.json"))
}

pub fn load() -> Result<Settings> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(Settings::default());
    }

    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

pub fn save(settings: &Settings) -> Result<Settings> {
    if let Some(days) = settings.retention_days {
        if !matches!(days, 7 | 30 | 90 | 365) {
            anyhow::bail!("unsupported retention period: {} days", days);
        }
    }

    let path = settings_path()?;
    std::fs::write(&path, serde_json::to_vec_pretty(settings)?)?;
    Ok(settings.clone())
}
