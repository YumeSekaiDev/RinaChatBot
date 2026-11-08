use crate::state::ChatMessage;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";

#[derive(Deserialize)]
struct OllamaTagItem {
    name: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaTagItem>,
}

#[derive(Serialize)]
struct OllamaChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OllamaChatOptions {
    temperature: f32,
}

#[derive(Serialize)]
struct OllamaChatRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaChatMessage<'a>>,
    options: OllamaChatOptions,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaChatAssistantMessage {
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaChatAssistantMessage>,
}

#[derive(Serialize)]
struct OllamaPullRequest<'a> {
    name: &'a str,
    stream: bool,
}

pub async fn check_ollama_status() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    client
        .get(format!("{}/api/tags", OLLAMA_BASE_URL))
        .send()
        .await
        .map(|res| res.status().is_success())
        .unwrap_or(false)
}

pub async fn fetch_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Client creation error: {}", e))?;

    let res = client
        .get(format!("{}/api/tags", OLLAMA_BASE_URL))
        .send()
        .await
        .map_err(|e| {
            format!(
                "Cannot connect to Ollama at {}. Is 'ollama serve' running? ({})",
                OLLAMA_BASE_URL, e
            )
        })?;

    if !res.status().is_success() {
        return Err(format!("Ollama server error: HTTP {}", res.status()));
    }

    let parsed: OllamaTagsResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse models list from Ollama: {}", e))?;

    let names: Vec<String> = parsed.models.into_iter().map(|m| m.name).collect();
    Ok(names)
}

pub async fn pull_model(model_name: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1800)) // 30 minutes for large model download
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let payload = OllamaPullRequest {
        name: model_name,
        stream: false,
    };

    let res = client
        .post(format!("{}/api/pull", OLLAMA_BASE_URL))
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Cannot connect to Ollama at {}. Is 'ollama serve' running? ({})",
                OLLAMA_BASE_URL, e
            )
        })?;

    if res.status().is_success() {
        Ok(format!("Model '{}' pulled successfully!", model_name))
    } else {
        Err(format!(
            "Failed to pull model '{}': HTTP {}",
            model_name,
            res.status()
        ))
    }
}

pub async fn chat_completion(
    messages: &[ChatMessage],
    model: &str,
    system_prompt: &str,
    temperature: f32,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let mut api_messages = Vec::new();

    // 1. System prompt
    if !system_prompt.trim().is_empty() {
        api_messages.push(OllamaChatMessage {
            role: "system",
            content: system_prompt,
        });
    }

    // 2. Chat history messages (keep last 15 messages to stay within local context window)
    let history_slice = if messages.len() > 15 {
        &messages[messages.len() - 15..]
    } else {
        messages
    };

    for msg in history_slice {
        api_messages.push(OllamaChatMessage {
            role: &msg.role,
            content: &msg.content,
        });
    }

    let payload = OllamaChatRequest {
        model,
        messages: api_messages,
        options: OllamaChatOptions { temperature },
        stream: false,
    };

    let res = client
        .post(format!("{}/api/chat", OLLAMA_BASE_URL))
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Cannot connect to Ollama at {}. Is 'ollama serve' running? ({})",
                OLLAMA_BASE_URL, e
            )
        })?;

    let status = res.status();
    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        if status.as_u16() == 404 || err_text.contains("not found") {
            return Err(format!(
                "Model '{}' was not found in Ollama. Please click 'Pull Model' to download it.",
                model
            ));
        }
        return Err(format!("Ollama error (HTTP {}): {}", status, err_text));
    }

    let chat_res: OllamaChatResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse response from Ollama: {}", e))?;

    match chat_res.message {
        Some(msg) => Ok(msg.content.trim().to_string()),
        None => Err("Ollama returned an empty response message.".to_string()),
    }
}
