use reqwest::{Client, Error, StatusCode};
use serde_json::json;

use crate::parser::{cache::Cache, config::ConfigCohere};

use super::ChatContent;

pub struct Cohere {}

const URL: &'static str = "https://api.cohere.ai/v1/chat";

impl Cohere {
    pub async fn request(query: &str, config: ConfigCohere) -> Result<ChatContent, Error> {
        let url = format!("{}", URL);
        let mut conversation = Self::create_query();
        conversation.as_object_mut().unwrap().insert("message".to_string(), serde_json::Value::String(query.to_string()));

        println!("{conversation}");

        let (response, status) = Self::send_request(&url, &conversation, config.api).await?;
        let result = Self::process_response(query, &response, status)?;

        Ok(result)
    }
    async fn send_request(
        url: &str,
        data: &serde_json::Value,
        api: String,
    ) -> Result<(String, StatusCode), Error> {
        println!("{data}");
        let response = Client::new()
            .post(url)
            .header("Authorization", format!("Bearer {}", api))
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
            .pointer("/text")
            .and_then(|val| val.as_str())
            .unwrap_or("");
        let result = ChatContent {
            question: query.to_string(),
            answer: answer.to_string(),
            status,
        };
        if status.is_success() {
            Cache::update_conversation(&result, "cohere");
        }
        Ok(result)
    }

    fn create_query() -> serde_json::Value {
        let mut template = json!({"chat_history": [], "connectors": [{"id": "web-search"}]});

        for item in Cache::read()["chat"].as_array().unwrap() {
            template["chat_history"].as_array_mut().unwrap().push(json!(
                {
                    "role": item["role"],
                    "message": item["text"]
                }
            ))
        }

        template
    }
}
