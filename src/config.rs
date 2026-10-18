use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default = "default_fps")]
    pub fps: u32,

    #[serde(default = "default_brightness")]
    pub default_brightness: u8,

    #[serde(default = "default_mode")]
    pub default_mode: String,

    #[serde(default)]
    pub wave: WaveConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveConfig {
    #[serde(default = "default_palette")]
    pub palette: String,

    #[serde(default = "default_speed")]
    pub speed: f32,

    #[serde(default)]
    pub reverse: bool,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            palette: default_palette(),
            speed: default_speed(),
            reverse: false,
        }
    }
}

fn default_fps() -> u32 { 4 }
fn default_brightness() -> u8 { 100 }
fn default_mode() -> String { "wave".to_string() }
fn default_palette() -> String { "cyberpunk".to_string() }
fn default_speed() -> f32 { 0.5 }

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            fps: default_fps(),
            default_brightness: default_brightness(),
            default_mode: default_mode(),
            wave: WaveConfig::default(),
        }
    }
}

impl DaemonConfig {
    pub fn get_default_config_path() -> PathBuf {
        if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_dir).join("omen-ry").join("config.toml")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("omen-ry").join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn load_or_default<P: AsRef<Path>>(path: Option<P>) -> Self {
        let p = path.map(|p| p.as_ref().to_path_buf()).unwrap_or_else(Self::get_default_config_path);
        if p.exists() {
            if let Ok(content) = fs::read_to_string(&p) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }
}
