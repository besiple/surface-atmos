use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use log::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_transition_ms")]
    pub transition_duration_ms: u64,

    #[serde(default = "default_step_interval_ms")]
    pub step_interval_ms: u64,

    #[serde(default = "default_notifications")]
    pub enable_notifications: bool,

    #[serde(default = "default_sink_name")]
    pub sink_name: String,
}

fn default_transition_ms() -> u64 {
    200
}

fn default_step_interval_ms() -> u64 {
    20
}

fn default_notifications() -> bool {
    false
}

fn default_sink_name() -> String {
    "surface_stage_compensator".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            transition_duration_ms: default_transition_ms(),
            step_interval_ms: default_step_interval_ms(),
            enable_notifications: default_notifications(),
            sink_name: default_sink_name(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/surface-atmos/config.toml")
    }

    pub fn load_or_default() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                    debug!("Loaded configuration from {}", path.display());
                    return cfg;
                } else {
                    warn!("Failed to parse config at {}, using defaults", path.display());
                }
            }
        }

        let default = Self::default();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
            if let Ok(toml_str) = toml::to_string_pretty(&default) {
                let _ = fs::write(&path, toml_str);
            }
        }
        default
    }
}
