use std::{
    fs::{self},
    path::PathBuf,
};

use serde_json::json;

use crate::models::ChatContent;

pub struct Cache {}

impl Cache {
    pub fn read(path: &PathBuf) -> serde_json::Value {
        if let Ok(cache_file) = fs::read_to_string(path) {
            let response: serde_json::Value =
                serde_json::from_str(&cache_file).unwrap_or(json!({"chat": []}));
            response
        } else {
            json!({"chat": []})
        }
    }

    fn write(path: PathBuf, response: serde_json::Value) {
        let cache_file = serde_json::to_string(&response).expect("Could not Serialize");
        fs::write(path, cache_file).expect("Could not write.");
    }
    pub fn update_conversation(file: PathBuf, response: &ChatContent, model: &str) {
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

        let mut conversation = Self::read(&file);
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

        Self::write(file, conversation);
    }

    pub fn read_all(dir_path: PathBuf) -> Vec<PathBuf> {
        fs::create_dir(&dir_path).ok();
        let mut dir_files = Vec::new();
        if let Ok(files) = fs::read_dir(dir_path) {
            for file in files {
                dir_files.push(file.expect("Error reading file").path())
            }
        }
        dir_files.sort();
        dir_files
    }
}
