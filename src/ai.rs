use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Serialize, Deserialize)]
struct OllamaOptions {
    num_ctx: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

pub async fn generate_summary(text: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&OllamaRequest {
            model: "llama3.1:latest".to_string(), // use latest explicitly as seen in curl
            prompt: format!("Summarize this email concisely:\n\n{}", text),
            stream: false,
            options: None,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(json["response"].as_str().unwrap_or("No response").to_string())
}

pub async fn chat_with_emails(question: &str, emails_context: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&OllamaRequest {
            model: "llama3.1:latest".to_string(),
            prompt: format!("You are an AI assistant helping with an email inbox. Using the following emails context, answer the user's question.\n\nContext:\n{}\n\nQuestion: {}", emails_context, question),
            stream: false,
            options: Some(OllamaOptions { num_ctx: 32768 }), // Increase context window
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(json["response"].as_str().unwrap_or("No response").to_string())
}
