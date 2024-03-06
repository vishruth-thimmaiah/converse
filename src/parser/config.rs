use std::fs;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub general: General,
    pub gemini: ConfigGemini,
    pub cohere: ConfigCohere,
}

#[derive(Debug, Clone, Deserialize)]
pub struct General {
    #[serde(default)]
    pub use_gtk_layer: bool,
    pub layer_margin_top: i32,
    pub layer_margin_bottom: i32,
    pub layer_margin_left: i32,
    pub layer_margin_right: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigGemini {
    #[serde(default)]
    pub api: String,
    #[serde(default)]
    pub use_model: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigCohere {
    #[serde(default)]
    pub api: String,
    #[serde(default)]
    pub use_model: u32,
    #[serde(default)]
    pub web_search: bool,
}

impl Config {
    pub fn new() -> Config {
        let toml_str =
            fs::read_to_string(format!("{}/.config/converse/config.toml", env!("HOME")))
                .unwrap_or_default();

        let config_file: Config =
            toml::from_str(&toml_str).expect("Failed to deserialize config.toml");

        config_file
    }
}
