use std::fs;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub general: General,
    #[serde(default)]
    pub theming: Theming,
    pub gemini: ConfigGemini,
    pub cohere: ConfigCohere,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Theming {
    pub quote_indicator: String,
    pub quote_foreground: String,
    pub code_background: String,
    pub code_foreground: String,

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

impl Default for Theming {
    fn default() -> Self {
        Self {
            quote_indicator: String::from("#ccc"),
            quote_foreground: String::from("#aaa"),
            code_background: String::from("#111"),
            code_foreground: String::from("#bbb"),
        }
    }
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
