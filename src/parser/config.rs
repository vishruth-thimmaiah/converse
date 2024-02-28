use std::fs;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub gemini: ConfigGemini,
    pub general: General,
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
