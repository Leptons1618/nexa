//! HTTP client for the Nexa Support backend API.

use crate::models::{
    ChatRequest, ChatResponse, ConfigResponse, HealthResponse, IngestRequest, IngestResponse,
    OllamaModelsResponse, OllamaStatusResponse,
};

const BASE: &str = "http://localhost:8000/api";

/// Send a chat message and return the response.
pub async fn send_chat(message: &str) -> Result<ChatResponse, String> {
    let client = reqwest::Client::new();
    let body = ChatRequest {
        message: message.to_string(),
        session_id: None,
    };
    let resp = client
        .post(format!("{BASE}/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<ChatResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Ingest documents at the given paths.
pub async fn ingest_documents(
    paths: Vec<String>,
    tags: Option<Vec<String>>,
    version: Option<String>,
) -> Result<IngestResponse, String> {
    let client = reqwest::Client::new();
    let body = IngestRequest {
        paths,
        tags,
        version,
    };
    let resp = client
        .post(format!("{BASE}/ingest"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<IngestResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Fetch service health.
pub async fn fetch_health() -> Result<HealthResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/health"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    resp.json::<HealthResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Fetch service configuration.
pub async fn fetch_config() -> Result<ConfigResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/config"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    resp.json::<ConfigResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// List models available on the Ollama server.
pub async fn fetch_ollama_models() -> Result<OllamaModelsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/ollama/models"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<OllamaModelsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Fetch Ollama server status.
pub async fn fetch_ollama_status() -> Result<OllamaStatusResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/ollama/status"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<OllamaStatusResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}
