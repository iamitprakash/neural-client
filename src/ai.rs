use serde::{Deserialize, Serialize};
use reqwest;
use std::env;
use tracing::{debug, warn, error};

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

async fn post_to_ollama(request: OllamaRequest) -> Result<String, String> {
    let endpoint = env::var("OLLAMA_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string());
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| {
            error!("Failed to create reqwest client: {}", e);
            format!("Failed to create client: {}", e)
        })?;

    let mut last_error = String::new();
    for attempt in 1..=3 {
        debug!("Ollama request attempt {} to {}", attempt, endpoint);
        match client.post(&endpoint)
            .json(&request)
            .send()
            .await 
        {
            Ok(res) => {
                let json: serde_json::Value = res.json().await.map_err(|e| {
                    error!("Ollama JSON parse error: {}", e);
                    format!("Invalid JSON response: {}", e)
                })?;
                debug!("Ollama request successful");
                return Ok(json["response"].as_str().unwrap_or("No response").to_string());
            }
            Err(e) => {
                warn!("Ollama request attempt {} failed: {}", attempt, e);
                last_error = format!("Attempt {}: {}", attempt, e);
                if attempt < 3 {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempt)).await;
                }
            }
        }
    }
    error!("Ollama request failed after 3 attempts: {}", last_error);
    Err(format!("AI Service unavailable after 3 attempts. Last error: {}", last_error))
}

pub async fn generate_summary(text: &str) -> Result<String, String> {
    post_to_ollama(OllamaRequest {
        model: "llama3.1:latest".to_string(),
        prompt: format!("Summarize this email concisely:\n\n{}", text),
        stream: false,
        options: None,
    }).await
}

pub async fn chat_with_emails(question: &str, emails_context: &str) -> Result<String, String> {
    post_to_ollama(OllamaRequest {
        model: "llama3.1:latest".to_string(),
        prompt: format!("You are an AI assistant helping with an email inbox. Using the following emails context, answer the user's question.\n\nContext:\n{}\n\nQuestion: {}", emails_context, question),
        stream: false,
        options: Some(OllamaOptions { num_ctx: 8192 }), // Optimized context window
    }).await
}

pub async fn generate_reply(email_text: &str) -> Result<String, String> {
    post_to_ollama(OllamaRequest {
        model: "llama3.1:latest".to_string(),
        prompt: format!(
            "You are an AI assistant tasked with writing a highly professional, concise reply to the following email. \
            Do not include conversational filler like 'Here is your reply:' or 'Certainly!'. Draft only the final text \
            of the response suitable for hitting send immediately.\n\nOriginal Email:\n{}",
            email_text
        ),
        stream: false,
        options: None,
    }).await
}

pub async fn categorize_email(subject: &str, body: &str) -> Result<String, String> {
    let result = post_to_ollama(OllamaRequest {
        model: "llama3.1:latest".to_string(),
        prompt: format!(
            "Categorize the following email into exactly one of these labels: [Inbox, Work, Finance, Social, Promotions]. \
            Respond with only the label name and nothing else. Output 'Inbox' if unsure.\n\nSubject: {}\n\nBody preview: {}",
            subject,
            if body.len() > 200 { &body[..200] } else { body }
        ),
        stream: false,
        options: None,
    }).await;

    match result {
        Ok(response) => {
            let response = response.trim();
            let valid_labels = ["Inbox", "Work", "Finance", "Social", "Promotions"];
            for label in valid_labels {
                if response.to_lowercase().contains(&label.to_lowercase()) {
                    return Ok(label.to_string());
                }
            }
            Ok("Inbox".to_string())
        }
        Err(e) => Err(e),
    }
}
