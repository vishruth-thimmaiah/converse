use reqwest::{Client, Error, StatusCode};
use serde_json::json;

use crate::parser::{cache::Cache, config::ConfigGemini};

use super::ChatContent;

pub struct Gemini {}

const URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=";

impl Gemini {
    pub async fn request(query: &str, config: ConfigGemini) -> Result<ChatContent, Error> {
        let url = format!("{}{}", URL, config.api);
        let mut conversation = Self::create_query();
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
        if status.is_success() {
            Cache::update_conversation(&result, "gemini");
        }
        Ok(result)
    }

    fn create_query() -> serde_json::Value {
        let mut template = json!({"contents": []});

        for item in Cache::read()["chat"].as_array().unwrap() {
            template["contents"].as_array_mut().unwrap().push(json!(
                {
                    "parts": [{
                        "text": item["text"]
                    }],
                    "role": item["role"]
                }
            ))
        }

        template
    }
}
