pub mod gemini;
pub mod cohere;

use reqwest::{Error, StatusCode};

use crate::parser::config::Config;

use self::{cohere::Cohere, gemini::Gemini};

pub struct ChatContent {
    pub question: String,
    pub answer: String,
    pub status: StatusCode,
}

pub async fn select_model(
    combobox_selection: &str,
    entry_text: &str,
    config: Config,
) -> Result<ChatContent, Error> {
    match combobox_selection {
        "Gemini" => Gemini::request(&entry_text, config.gemini).await,
        "Cohere" => Cohere::request(&entry_text, config.cohere).await,
        _ => unreachable!(),
    }
}
