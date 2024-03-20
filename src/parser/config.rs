use std::{env::var, fs};

use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: General,
    pub theming: Theming,
    pub gemini: ConfigGemini,
    pub cohere: ConfigCohere,
    pub claude: ConfigClaude,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Theming {
    pub quote_indicator: String,
    pub quote_foreground: String,
    pub code_background: String,
    pub code_foreground: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct General {
    pub use_gtk_layer: bool,
    pub layer_margin_top: i32,
    pub layer_margin_bottom: i32,
    pub layer_margin_left: i32,
    pub layer_margin_right: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ConfigGemini {
    pub api: String,
    pub use_model: u32,
    pub conversation_input: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ConfigCohere {
    pub api: String,
    pub use_model: u32,
    pub conversation_input: serde_json::Value,
    pub web_search: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ConfigClaude {
    pub api: String,
    pub use_model: u32,
    pub conversation_input: serde_json::Value,
    pub max_tokens: u32,
    pub model: String,
    pub anthropic_version: String,
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

impl Default for General {
    fn default() -> Self {
        Self {
            use_gtk_layer: false,
            layer_margin_top: 0,
            layer_margin_bottom: 0,
            layer_margin_left: 0,
            layer_margin_right: 0,
        }
    }
}

impl Default for ConfigGemini {
    fn default() -> Self {
        Self {
            api: {
                if let Ok(key) = var("GEMINI_API_KEY") {
                    key
                } else {
                    String::new()
                }
            },
            use_model: 2,
            conversation_input: json!([]),
        }
    }
}

impl Default for ConfigCohere {
    fn default() -> Self {
        Self {
            api: {
                if let Ok(key) = var("COHERE_API_KEY") {
                    key
                } else {
                    String::new()
                }
            },
            use_model: 1,
            conversation_input: json!([]),
            web_search: false,
        }
    }
}

impl Default for ConfigClaude {
    fn default() -> Self {
        Self {
            api: {
                if let Ok(key) = var("CLAUDE_API_KEY") {
                    key
                } else {
                    String::new()
                }
            },
            use_model: 1,
            conversation_input: json!([]),
            max_tokens: 1024,
            model: "claude-3-haiku-20240307".to_string(),
            anthropic_version: "2023-06-01".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General::default(),
            theming: Theming::default(),
            gemini: ConfigGemini::default(),
            cohere: ConfigCohere::default(),
            claude: ConfigClaude::default(),
        }
    }
}

impl Config {
    pub fn new() -> Config {
        let toml_str = fs::read_to_string(format!("{}/.config/converse/config.toml", env!("HOME")))
            .unwrap_or_default();

        let config_file: Config =
            toml::from_str(&toml_str).expect("Failed to deserialize config.toml");

        if config_file.gemini.use_model != 0 && config_file.gemini.api.is_empty() {
            println!("Please set gemini api key in config.toml");
        }
        if config_file.cohere.use_model != 0 && config_file.cohere.api.is_empty() {
            println!("Please set cohere api key in config.toml");
        }

        config_file
    }
}
