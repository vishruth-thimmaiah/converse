use std::{fs, path::Path};

use serde_json::json;

use crate::models::ChatContent;

pub struct Cache {}

impl Cache {
    pub fn read() -> serde_json::Value {
        let path: String = format!("{}/.cache/converse/chat_history", env!("HOME"));
        if !Path::new(&path).exists() {
            let _ = fs::create_dir(format!("{}/.cache/converse", env!("HOME")));
            let _ = fs::File::create(path.clone());
        };
        let cache_file = fs::read_to_string(path).expect("Could not read.");

        let response: serde_json::Value =
            serde_json::from_str(&cache_file).unwrap_or(json!({"chat": []}));
        response
    }

    fn write(response: serde_json::Value) {
        let cache_file = serde_json::to_string(&response).expect("Could not Serialize");
        fs::write(
            format!("{}/.cache/converse/chat_history", env!("HOME")),
            cache_file,
        )
        .expect("Could not write.");
    }
    pub fn update_conversation(response: &ChatContent, model: &str) {
        let new_question = json!(
        {
            "role": "user",
            "text": response.question
        });
        let new_answer = json!(
        {
            "role": "model",
            "text": response.answer
        });

        let mut conversation = Self::read();
        conversation
            .as_object_mut()
            .unwrap()
            .entry("model")
            .or_insert(json!(model));
        conversation["chat"]
            .as_array_mut()
            .unwrap()
            .push(new_question);
        conversation["chat"]
            .as_array_mut()
            .unwrap()
            .push(new_answer);

        Self::write(conversation);
    }
    pub fn truncate() {
        let path: String = format!("{}/.cache/converse/chat_history", env!("HOME"));
        let file = fs::File::create(path).unwrap();
        let _ = file.set_len(0);
    }
}
