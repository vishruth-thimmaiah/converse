use std::{env::var, fs, path::PathBuf, process::exit};

use clap::Parser;
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
    pub openai: ConfigOpenAI,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ConfigOpenAI {
    pub api: String,
    pub use_model: u32,
    pub conversation_input: serde_json::Value,
    pub model: String,
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

impl Default for ConfigOpenAI {
    fn default() -> Self {
        Self {
            api: {
                if let Ok(key) = var("OPENAI_API_KEY") {
                    key
                } else {
                    String::new()
                }
            },
            use_model: 1,
            conversation_input: json!([]),
            model: "gpt-3.5-turbo".to_string(),
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
            openai: ConfigOpenAI::default(),
        }
    }
}

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    /// Specify config file path
    config: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Config {
        let args = Args::parse();

        let toml_str = if let Some(path) = args.config {
            fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("Error reading file: {}", e);
                exit(1)
            })
        } else {
            fs::read_to_string(format!("{}/.config/converse/config.toml", env!("HOME")))
                .unwrap_or_else(|e| {
                    eprintln!("Error reading file: {}; using default values.", e);
                    String::new()
                })
        };

        let config_file: Config = toml::from_str(&toml_str).unwrap_or_else(|e| {
            eprintln!("Error deserializing the file: {}", e);
            exit(1)
        });

        if config_file.gemini.use_model != 0 && config_file.gemini.api.is_empty() {
            eprintln!("Please set gemini api key in config.toml");
        }
        if config_file.cohere.use_model != 0 && config_file.cohere.api.is_empty() {
            eprintln!("Please set cohere api key in config.toml");
        }
        if config_file.claude.use_model != 0 && config_file.claude.api.is_empty() {
            eprintln!("Please set claude api key in config.toml");
        }
        if config_file.openai.use_model != 0 && config_file.openai.api.is_empty() {
            eprintln!("Please set openai api key in config.toml");
        }

        config_file
    }
}
