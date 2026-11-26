use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

pub fn data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot find home dir"))?;
    let dir = home.join(".hindsight");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn settings_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("settings.json"))
}

pub fn load() -> Result<Settings> {
    load_from(&settings_path()?)
}

pub fn load_or_default() -> Settings {
    load().unwrap_or_default()
}

pub fn load_from(path: &Path) -> Result<Settings> {
    if !path.exists() {
        return Ok(Settings::default());
    }

    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

pub fn save(settings: &Settings) -> Result<Settings> {
    save_to(&settings_path()?, settings)
}

pub fn save_to(path: &Path, settings: &Settings) -> Result<Settings> {
    if let Some(days) = settings.retention_days {
        if !matches!(days, 7 | 30 | 90 | 365) {
            anyhow::bail!("unsupported retention period: {} days", days);
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, serde_json::to_vec_pretty(settings)?)?;
    Ok(settings.clone())
}

#[cfg(test)]
mod tests {
    use super::{load_from, save_to, Settings};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_settings_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("hindsight-test-{name}-{nanos}"))
            .join("settings.json")
    }

    #[test]
    fn missing_settings_returns_defaults() {
        let path = temp_settings_path("missing");
        let loaded = load_from(&path).expect("load default settings");
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let path = temp_settings_path("roundtrip");
        let settings = Settings {
            recording_enabled: false,
            retention_days: Some(30),
        };
        save_to(&path, &settings).expect("save settings");
        assert_eq!(load_from(&path).expect("reload settings"), settings);
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn save_rejects_unsupported_retention_days() {
        let path = temp_settings_path("invalid");
        let settings = Settings {
            recording_enabled: true,
            retention_days: Some(14),
        };
        assert!(save_to(&path, &settings).is_err());
    }
}
