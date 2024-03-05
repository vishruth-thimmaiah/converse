pub mod gemini;

use reqwest::Error;

use crate::parser::config::Config;

use self::gemini::{ChatContent, Gemini};

pub async fn select_model(
    combobox_selection: &str,
    entry_text: &str,
    config: Config,
) -> Result<ChatContent, Error> {
    match combobox_selection {
        "Gemini" => Gemini::request(&entry_text, config.gemini).await,
        "Cohere" => unimplemented!(),
        _ => unreachable!(),
    }
}
