pub mod claude;
pub mod cohere;
pub mod gemini;
pub mod openai;

use std::{path::PathBuf, sync::Arc};

use reqwest::{Error, StatusCode};

use crate::parser::{cache::Cache, config::Config};

use self::{claude::Claude, cohere::Cohere, gemini::Gemini, openai::OpenAI};

pub struct ChatContent {
    pub question: String,
    pub answer: String,
    pub status: StatusCode,
}

pub fn get_models(config: &Arc<Config>) -> Vec<String> {
    let mut models = Vec::new();
    models.push((config.gemini.use_model, "Gemini"));
    models.push((config.cohere.use_model, "Cohere"));
    models.push((config.claude.use_model, "Claude"));
    models.push((config.openai.use_model, "OpenAI"));

    // sort models by use_model and return as Vec<String>, where higher use_model is first.
    models.sort_by(|a, b| b.0.cmp(&a.0));
    let sorted_models: Vec<String> = models
        .iter()
        .filter_map(|(priority, model)| {
            if priority != &0 {
                Some(model.to_string())
            } else {
                None
            }
        })
        .collect();

    sorted_models
}

pub async fn select_model(
    combobox_selection: &str,
    entry_text: &str,
    config: Arc<Config>,
    file: PathBuf,
) -> Result<ChatContent, Error> {
    let init_input = Cache::read(&file);
    let result = match combobox_selection {
        "Gemini" => Gemini::request(&entry_text, &config.gemini, &init_input["chat"]).await,
        "Cohere" => Cohere::request(&entry_text, &config.cohere, &init_input["chat"]).await,
        "Claude" => Claude::request(&entry_text, &config.claude, &init_input["chat"]).await,
        "OpenAI" => OpenAI::request(&entry_text, &config.openai, &init_input["chat"]).await,
        _ => unreachable!(),
    };
    if let Ok(output) = &result {
        if output.status.is_success() {
            Cache::update_conversation(file, output, combobox_selection);
        }
    }
    result
}
