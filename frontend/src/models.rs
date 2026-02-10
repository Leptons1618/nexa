//! Shared data models (mirroring backend schemas).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[allow(dead_code)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct HealthResponse {
    pub status: String,
    pub llm_connected: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub digest: Option<String>,
    #[allow(dead_code)]
    pub modified_at: Option<String>,
}

/// Response from `/api/ollama/models`.
#[derive(Clone, Debug, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModelEntry>,
}

/// Response from `/api/ollama/status`.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct OllamaStatusResponse {
    pub running: bool,
    pub base_url: String,
    pub model: String,
    pub models_available: u32,
}

/// Request to switch the Ollama model.
#[derive(Clone, Debug, Serialize)]
pub struct SwitchModelRequest {
    pub model: String,
}

/// Response from switching the Ollama model.
#[derive(Clone, Debug, Deserialize)]
pub struct SwitchModelResponse {
    #[allow(dead_code)]
    pub previous_model: String,
    pub current_model: String,
}

/// Local chat message for display.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ChatMessage {
    pub role: Role,
    pub text: String,
    pub sources: Vec<String>,
    /// Rich source contexts with document snippets for hover preview.
    #[serde(skip)]
    pub source_contexts: Vec<SourceContext>,
    /// Whether this message is still being streamed.
    #[serde(skip)]
    pub streaming: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// Source context for hover preview — a retrieved chunk.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SourceContext {
    pub document: String,
    pub chunk_id: String,
    pub text: String,
    pub score: f64,
}

/// A single chat session for the history sidebar.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    #[serde(skip)]
    pub messages: Vec<ChatMessage>,
    pub documents: Vec<String>,
    pub created_at: String,
}

/// Server-side session summary (from GET /api/sessions).
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub message_count: u32,
    pub document_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionSummary>,
}

/// Server-side session detail with messages (from GET /api/sessions/{id}).
#[derive(Clone, Debug, Deserialize)]
pub struct ServerMessage {
    pub role: String,
    pub text: String,
    #[serde(default)]
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct SessionDetailResponse {
    pub id: String,
    pub title: String,
    pub messages: Vec<ServerMessage>,
    #[serde(default)]
    pub documents: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A document that has been uploaded in the current session.
#[derive(Clone, Debug, PartialEq)]
pub struct UploadedDoc {
    pub name: String,
    pub path: String,
    pub chunks: u64,
    pub size: u64,
    pub tags: Vec<String>,
}

/// Backend response from `/api/upload`.
#[derive(Clone, Debug, Deserialize)]
pub struct UploadedFileInfo {
    pub original_name: String,
    pub saved_path: String,
    pub size: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UploadResponse {
    pub files: Vec<UploadedFileInfo>,
    pub chunks_indexed: u64,
}

// ── New models for settings management ─────────────────

/// Prompts read/write.
#[derive(Clone, Debug, Deserialize)]
pub struct PromptsResponse {
    pub system_prompt: String,
    pub rag_addon_prompt: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PromptsUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_addon_prompt: Option<String>,
}

/// Uploaded file entry from `/api/uploads`.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct UploadedFileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UploadedFilesListResponse {
    pub files: Vec<UploadedFileEntry>,
}

/// Index statistics from `/api/index/stats`.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct IndexStatsResponse {
    pub total_vectors: u64,
    pub total_metadata: u64,
    pub index_path: String,
    pub metadata_path: String,
    pub index_size_bytes: u64,
    pub metadata_size_bytes: u64,
}

/// LLM/RAG tuning settings.
#[derive(Clone, Debug, Deserialize)]
pub struct LLMSettingsResponse {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: u64,
    pub chunk_size: u64,
    pub chunk_overlap: u64,
    pub top_k: u64,
    pub similarity_threshold: f64,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct LLMSettingsUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_overlap: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_threshold: Option<f64>,
}

// ── API Keys ─────────────────────────────────────────────

/// API keys configuration (GET response — key is never exposed).
#[derive(Clone, Debug, Deserialize)]
pub struct ApiKeysResponse {
    pub llm_provider: String,
    pub cloud_api_key_set: bool,
    pub cloud_base_url: String,
    pub cloud_model: String,
}

/// API keys update request.
#[derive(Clone, Debug, Serialize, Default)]
pub struct ApiKeysUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_model: Option<String>,
}

// ── Cloud Model Listing ──────────────────────────────────

/// A model entry from the cloud provider.
#[derive(Clone, Debug, Deserialize)]
pub struct CloudModelEntry {
    pub id: String,
    #[allow(dead_code)]
    pub owned_by: String,
}

/// Response from `/api/cloud/models`.
#[derive(Clone, Debug, Deserialize)]
pub struct CloudModelsResponse {
    pub models: Vec<CloudModelEntry>,
}

// ── Connection Test ──────────────────────────────────────

/// Request to test a cloud connection (optionally overriding credentials).
#[derive(Clone, Debug, Serialize, Default)]
pub struct ConnectionTestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Response from `/api/cloud/test-connection`.
#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionTestResponse {
    pub success: bool,
    pub message: String,
    #[allow(dead_code)]
    pub models_count: Option<u64>,
}

// ── API Profiles ─────────────────────────────────────────

/// An API profile for saving multiple configurations.
#[derive(Clone, Debug, Serialize)]
pub struct ApiProfileCreate {
    pub id: String,
    pub name: String,
    pub llm_provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_api_key: Option<String>,
    pub cloud_base_url: String,
    pub cloud_model: String,
}

/// Profile summary returned from the server (no key exposed).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ApiProfileSummary {
    pub id: String,
    pub name: String,
    pub llm_provider: String,
    pub cloud_api_key_set: bool,
    pub cloud_base_url: String,
    pub cloud_model: String,
}

/// Response from `/api/profiles`.
#[derive(Clone, Debug, Deserialize)]
pub struct ApiProfileListResponse {
    pub profiles: Vec<ApiProfileSummary>,
    pub active_profile_id: Option<String>,
}
