use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: General,
    pub hyprscroll: HyprScroll,
    pub volume: Volume,
    pub appscroll: AppScroll,
    pub zoom: Zoom,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct General {
    pub overlay_timeout_ms: u64,
    pub mode_order: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct HyprScroll {
    pub pixels_per_tick: i32,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Volume {
    pub step_percent: u32,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AppScroll {
    pub speed_multiplier: i32,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Zoom {
    pub step: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General::default(),
            hyprscroll: HyprScroll::default(),
            volume: Volume::default(),
            appscroll: AppScroll::default(),
            zoom: Zoom::default(),
        }
    }
}

impl Default for General {
    fn default() -> Self {
        Self {
            overlay_timeout_ms: 1500,
            mode_order: vec![
                "volume".into(),
                "zoom".into(),
                "appscroll".into(),
                "hyprscroll".into(),
            ],
        }
    }
}

impl Default for HyprScroll {
    fn default() -> Self {
        Self {
            pixels_per_tick: 0, // 0 = column mode (animated), >0 = pixel mode
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self { step_percent: 2 }
    }
}

impl Default for AppScroll {
    fn default() -> Self {
        Self {
            speed_multiplier: 1,
        }
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self { step: 0.5 }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => {
                        log::info!("Loaded config from {}", path.display());
                        return config;
                    }
                    Err(e) => log::warn!("Failed to parse config: {e}, using defaults"),
                },
                Err(e) => log::warn!("Failed to read config: {e}, using defaults"),
            }
        } else {
            log::info!("No config file found, using defaults");
        }
        Self::default()
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("surface-dial")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }
}
