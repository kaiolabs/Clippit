use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub hotkeys: HotkeyConfig,
    pub ui: UiConfig,
    pub search: SearchConfig,
    pub features: FeaturesConfig,
    pub privacy: PrivacyConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_max_history")]
    pub max_history_items: usize,
    
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    
    #[serde(default = "default_max_text_size")]
    pub max_text_size: usize,
    
    #[serde(default = "default_max_image_size")]
    pub max_image_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_hotkey_modifier")]
    pub show_history_modifier: String,
    
    #[serde(default = "default_hotkey_key")]
    pub show_history_key: String,
    
    pub show_history_alt_modifier: Option<String>,
    pub show_history_alt_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    
    #[serde(default = "default_language")]
    pub language: String,
    
    #[serde(default = "default_font_family")]
    pub font_family: String,
    
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    
    pub colors: UiColors,
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub dark: ThemeColors,
    pub light: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub selection: String,
    pub border: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: u32,
    
    #[serde(default = "default_window_height")]
    pub max_height: u32,
    
    #[serde(default = "default_window_position")]
    pub position: String,
    
    #[serde(default = "default_window_opacity")]
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_enable_suggestions")]
    pub enable_suggestions: bool,
    
    #[serde(default = "default_max_suggestions")]
    pub max_suggestions: usize,
    
    #[serde(default = "default_focus_search_modifier")]
    pub focus_search_modifier: String,
    
    #[serde(default = "default_focus_search_key")]
    pub focus_search_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default = "default_true")]
    pub capture_text: bool,
    
    #[serde(default = "default_true")]
    pub capture_images: bool,
    
    #[serde(default = "default_false")]
    pub capture_files: bool,
    
    #[serde(default = "default_false")]
    pub sync_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    #[serde(default = "default_true")]
    pub ignore_sensitive_apps: bool,
    
    #[serde(default)]
    pub ignored_apps: Vec<String>,
    
    #[serde(default = "default_false")]
    pub clear_on_exit: bool,
    
    #[serde(default = "default_enable_image_capture")]
    pub enable_image_capture: bool,
    
    #[serde(default = "default_max_image_size_mb")]
    pub max_image_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    pub database_path: Option<String>,
    pub ipc_socket: Option<String>,
}

// Default functions
fn default_max_history() -> usize { 100 }
fn default_poll_interval() -> u64 { 200 }
fn default_max_text_size() -> usize { 10 * 1024 * 1024 }
fn default_max_image_size() -> usize { 50 * 1024 * 1024 }
fn default_hotkey_modifier() -> String { "super".to_string() }
fn default_hotkey_key() -> String { "v".to_string() }
fn default_theme() -> String { "system".to_string() }
fn default_language() -> String { "en".to_string() }
fn default_font_family() -> String { "Nunito".to_string() }
fn default_font_size() -> u32 { 14 }
fn default_window_width() -> u32 { 600 }
fn default_window_height() -> u32 { 400 }
fn default_window_position() -> String { "center".to_string() }
fn default_window_opacity() -> f32 { 0.95 }
fn default_log_level() -> String { "info".to_string() }
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_enable_image_capture() -> bool { true }
fn default_max_image_size_mb() -> u32 { 10 }
fn default_enable_suggestions() -> bool { true }
fn default_max_suggestions() -> usize { 3 }
fn default_focus_search_modifier() -> String { "ctrl".to_string() }
fn default_focus_search_key() -> String { "p".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                max_history_items: default_max_history(),
                poll_interval_ms: default_poll_interval(),
                max_text_size: default_max_text_size(),
                max_image_size: default_max_image_size(),
            },
            hotkeys: HotkeyConfig {
                show_history_modifier: default_hotkey_modifier(),
                show_history_key: default_hotkey_key(),
                show_history_alt_modifier: None,
                show_history_alt_key: None,
            },
            ui: UiConfig {
                theme: default_theme(),
                language: default_language(),
                font_family: default_font_family(),
                font_size: default_font_size(),
                colors: UiColors {
                    dark: ThemeColors {
                        background: "#1e1e1e".to_string(),
                        foreground: "#ffffff".to_string(),
                        selection: "#264f78".to_string(),
                        border: "#454545".to_string(),
                    },
                    light: ThemeColors {
                        background: "#ffffff".to_string(),
                        foreground: "#000000".to_string(),
                        selection: "#0078d4".to_string(),
                        border: "#cccccc".to_string(),
                    },
                },
                window: WindowConfig {
                    width: default_window_width(),
                    max_height: default_window_height(),
                    position: default_window_position(),
                    opacity: default_window_opacity(),
                },
            },
            search: SearchConfig {
                enable_suggestions: default_enable_suggestions(),
                max_suggestions: default_max_suggestions(),
                focus_search_modifier: default_focus_search_modifier(),
                focus_search_key: default_focus_search_key(),
            },
            features: FeaturesConfig {
                capture_text: true,
                capture_images: true,
                capture_files: false,
                sync_enabled: false,
            },
            privacy: PrivacyConfig {
                ignore_sensitive_apps: true,
                ignored_apps: vec![
                    "keepassxc".to_string(),
                    "bitwarden".to_string(),
                    "1password".to_string(),
                ],
                clear_on_exit: false,
                enable_image_capture: default_enable_image_capture(),
                max_image_size_mb: default_max_image_size_mb(),
            },
            advanced: AdvancedConfig {
                log_level: default_log_level(),
                database_path: None,
                ipc_socket: None,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if !config_path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        std::fs::write(&config_path, contents)
            .context("Failed to write config file")?;
        
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        path.push("clippit");
        path.push("config.toml");
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.max_history_items, 100);
        assert_eq!(config.hotkeys.show_history_modifier, "super");
        assert_eq!(config.hotkeys.show_history_key, "v");
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.general.max_history_items, 100);
    }
}
