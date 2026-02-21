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

pub async fn generate_reply(email_text: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&OllamaRequest {
            model: "llama3.1:latest".to_string(),
            prompt: format!(
                "You are an AI assistant tasked with writing a highly professional, concise reply to the following email. \
                Do not include conversational filler like 'Here is your reply:' or 'Certainly!'. Draft only the final text \
                of the response suitable for hitting send immediately.\n\nOriginal Email:\n{}",
                email_text
            ),
            stream: false,
            options: None,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(json["response"].as_str().unwrap_or("No response").to_string())
}

pub async fn categorize_email(subject: &str, body: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&OllamaRequest {
            model: "llama3.1:latest".to_string(),
            prompt: format!(
                "Categorize the following email into exactly one of these labels: [Inbox, Work, Finance, Social, Promotions]. \
                Respond with only the label name and nothing else. Output 'Inbox' if unsure.\n\nSubject: {}\n\nBody preview: {}",
                subject,
                if body.len() > 200 { &body[..200] } else { body }
            ),
            stream: false,
            options: None,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let response = json["response"].as_str().unwrap_or("Inbox").trim().to_string();
    
    let valid_labels = ["Inbox", "Work", "Finance", "Social", "Promotions"];
    for label in valid_labels {
        if response.to_lowercase().contains(&label.to_lowercase()) {
            return Ok(label.to_string());
        }
    }
    Ok("Inbox".to_string())
}
