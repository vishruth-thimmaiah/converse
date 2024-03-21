pub mod claude;
pub mod cohere;
pub mod gemini;
pub mod openai;

use reqwest::{Error, StatusCode};

use crate::parser::config::Config;

use self::{claude::Claude, cohere::Cohere, gemini::Gemini, openai::OpenAI};

pub struct ChatContent {
    pub question: String,
    pub answer: String,
    pub status: StatusCode,
}

pub fn get_models(config: &Config) -> Vec<String> {
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
    config: Config,
) -> Result<ChatContent, Error> {
    match combobox_selection {
        "Gemini" => Gemini::request(&entry_text, config.gemini).await,
        "Cohere" => Cohere::request(&entry_text, config.cohere).await,
        "Claude" => Claude::request(&entry_text, config.claude).await,
        "OpenAI" => OpenAI::request(&entry_text, config.openai).await,
        _ => unreachable!(),
    }
}
