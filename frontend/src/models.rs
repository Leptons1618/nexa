//! Shared data models (mirroring backend schemas).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChatResponse {
    pub answer: String,
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct IngestRequest {
    pub paths: Vec<String>,
    pub tags: Option<Vec<String>>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IngestResponse {
    pub chunks_indexed: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub llm_connected: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigResponse {
    pub llm_provider: String,
    pub model_name: String,
    pub vector_store: String,
    pub embedding_model: String,
}

/// Ollama model entry returned by `/api/ollama/models`.
#[derive(Clone, Debug, Deserialize)]
pub struct OllamaModelEntry {
    pub name: String,
    pub size: Option<u64>,
    pub digest: Option<String>,
    pub modified_at: Option<String>,
}

/// Response from `/api/ollama/models`.
#[derive(Clone, Debug, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModelEntry>,
}

/// Response from `/api/ollama/status`.
#[derive(Clone, Debug, Deserialize)]
pub struct OllamaStatusResponse {
    pub running: bool,
    pub base_url: String,
    pub model: String,
    pub models_available: u32,
}

/// Local chat message for display.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub role: Role,
    pub text: String,
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    User,
    Assistant,
}
