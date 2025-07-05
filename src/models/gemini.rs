use reqwest::{Client, Error, StatusCode};
use serde_json::json;

use crate::parser::config::ConfigGemini;

use super::ChatContent;

pub struct Gemini {}

const URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/MODEL:generateContent?key=";

impl Gemini {
    pub async fn request(
        query: &str,
        config: &ConfigGemini,
        init_input: &serde_json::Value,
    ) -> Result<ChatContent, Error> {
        let url = format!("{}{}", URL.replace("MODEL", &config.model), config.api);
        let mut conversation = Self::create_query(&config.conversation_input, &init_input);
        conversation["contents"].as_array_mut().unwrap().push(json!(
            {
            "role": "user",
            "parts": [{
                "text": query
            }]
        }));

        let (response, status) = Self::send_request(&url, &conversation).await?;
        let result = Self::process_response(query, &response, status)?;

        Ok(result)
    }

    async fn send_request(
        url: &str,
        data: &serde_json::Value,
    ) -> Result<(String, StatusCode), Error> {
        let response = Client::new()
            .post(url)
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await?;
        let status = response.status();
        let response = response.text().await?;
        Ok((response, status))
    }

    fn process_response(
        query: &str,
        response: &str,
        status: StatusCode,
    ) -> Result<ChatContent, Error> {
        let response_content: serde_json::Value = serde_json::from_str(response).unwrap();
        let answer = response_content
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(|val| val.as_str())
            .unwrap_or("");
        let result = ChatContent {
            question: query.to_string(),
            answer: answer.to_string(),
            status,
        };
        Ok(result)
    }

    fn create_query(
        conversation_input: &serde_json::Value,
        init_input: &serde_json::Value,
    ) -> serde_json::Value {
        let mut template = json!({"contents": []});

        for item in conversation_input.as_array().unwrap() {
            template["contents"]
                .as_array_mut()
                .unwrap()
                .push(json!({"role": item["role"], "parts": [{"text": item["text"]}]}));
        }

        for item in init_input.as_array().unwrap() {
            template["contents"]
                .as_array_mut()
                .unwrap()
                .push(json!({"parts": [{"text": item["text"]}], "role": item["role"]}));
        }

        template
    }
}
