use reqwest::{Client, Error, StatusCode};
use serde_json::json;

use crate::parser::config::ConfigClaude;

use super::ChatContent;

pub struct Claude {}

const URL: &'static str = "https://api.anthropic.com/v1/messages";

impl Claude {
    pub async fn request(
        query: &str,
        config: &ConfigClaude,
        init_input: &serde_json::Value,
    ) -> Result<ChatContent, Error> {
        let url = format!("{}", URL);
        let mut conversation = Self::create_query(
            config.max_tokens,
            &config.model,
            &config.conversation_input,
            init_input,
        );

        conversation["messages"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "role": "user", "content": query }));

        let (response, status) =
            Self::send_request(&url, &conversation, &config.api, &config.anthropic_version).await?;
        let result = Self::process_response(query, &response, status)?;

        Ok(result)
    }
    async fn send_request(
        url: &str,
        data: &serde_json::Value,
        api: &str,
        anthropic_version: &str,
    ) -> Result<(String, StatusCode), Error> {
        let response = Client::new()
            .post(url)
            .header("x-api-key", api)
            .header("anthropic-version", anthropic_version)
            .header("Content-Type", "application/json")
            .json(&data)
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
            .pointer("/content/0/text")
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
        tokens: u32,
        model: &str,
        conversation_input: &serde_json::Value,
        init_input: &serde_json::Value,
    ) -> serde_json::Value {
        let mut template = json!({"model": model, "max_tokens": tokens, "messages": []});

        for item in conversation_input.as_array().unwrap() {
            template["messages"]
                .as_array_mut()
                .unwrap()
                .push(json!({ "role": item["role"].as_str().unwrap().replace("model", "assistant"), "content": item["text"]}))
        }

        for item in init_input.as_array().unwrap() {
            template["messages"]
                .as_array_mut()
                .unwrap()
                .push(json!({ "role": item["role"].as_str().unwrap().replace("model", "assistant"), "content": item["text"]}))
        }

        template
    }
}
