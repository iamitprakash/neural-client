use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    pub response: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

pub async fn generate_summary(text: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&OllamaRequest {
            model: "llama3".to_string(), // Default to llama3
            prompt: format!("Summarize this email concisely:\n\n{}", text),
            stream: false,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(json["response"].as_str().unwrap_or("No response").to_string())
}
