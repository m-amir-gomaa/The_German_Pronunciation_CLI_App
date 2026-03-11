use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub editor: EditorConfig,
    pub tts: TtsConfig,
    pub whisper: WhisperConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Enable full Vim modal editing (Normal / Insert / Visual)
    pub vim_mode: bool,
    /// Which mode to start in when vim_mode = true
    pub vim_default_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TtsConfig {
    /// Edge TTS neural voice name
    pub preferred_voice: String,
    /// "piper" or "espeak" — used when offline
    pub fallback: String,
    /// Path to the Piper .onnx model file
    pub piper_model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperConfig {
    /// "tiny" | "base" | "small"
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_ipa: bool,
    pub autocomplete_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig {
                vim_mode: false,
                vim_default_mode: "insert".to_string(),
            },
            tts: TtsConfig {
                preferred_voice: "de-DE-KatjaNeural".to_string(),
                fallback: "piper".to_string(),
                piper_model: "~/.local/share/pronouncer/de_DE-thorsten-high.onnx".to_string(),
            },
            whisper: WhisperConfig {
                model: "base".to_string(),
            },
            ui: UiConfig {
                show_ipa: true,
                autocomplete_limit: 10,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            let default = Config::default();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_str = toml::to_string_pretty(&default)?;
            fs::write(&path, &toml_str)?;
            println!("Created default config at {}", path.display());
            return Ok(default);
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;
        Ok(config)
    }
}

fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(home).join(".config")
        });
    base.join("pronouncer").join("config.toml")
}
