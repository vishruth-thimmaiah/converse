use reqwest::{Client, Error, StatusCode};
use serde_json::json;

use crate::parser::{cache::Cache, config::ConfigOpenAI};

use super::ChatContent;

pub struct OpenAI {}

const URL: &str = "https://api.openai.com/v1/chat/completions";

impl OpenAI {
    pub async fn request(query: &str, config: ConfigOpenAI) -> Result<ChatContent, Error> {
        let mut conversation = Self::create_query(config.conversation_input, config.model);
        conversation["messages"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "role": "user", "content": query }));

        let (response, status) = Self::send_request(URL, config.api, &conversation).await?;
        let result = Self::process_response(query, &response, status)?;

        Ok(result)
    }

    async fn send_request(
        url: &str,
        api: String,
        data: &serde_json::Value,
    ) -> Result<(String, StatusCode), Error> {
        let response = Client::new()
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api))
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
            .pointer("/choices/0/message/content")
            .and_then(|val| val.as_str())
            .unwrap_or("");
        let result = ChatContent {
            question: query.to_string(),
            answer: answer.to_string(),
            status,
        };
        if status.is_success() {
            Cache::update_conversation(&result, "OpenAI");
        }
        Ok(result)
    }

    fn create_query(conversation_input: serde_json::Value, model: String) -> serde_json::Value {
        let mut template = json!({"model": model, "messages": []});

        for item in conversation_input.as_array().unwrap() {
            template["messages"]
                .as_array_mut()
                .unwrap()
                .push(json!({"role": item["role"].as_str().unwrap().replace("model", "assistant"), "content": item["text"] }));
        }

        for item in Cache::read()["chat"].as_array().unwrap() {
            template["messages"]
                .as_array_mut()
                .unwrap()
                .push(json!({"role": item["role"].as_str().unwrap().replace("model", "assistant"), "content": item["text"] }));
        }

        template
    }
}
