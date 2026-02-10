//! HTTP client for the Nexa Support backend API.

use crate::models::{
    ApiKeysResponse, ApiKeysUpdateRequest,
    ApiProfileCreate, ApiProfileListResponse, ApiProfileSummary,
    ChatRequest, ChatResponse, CloudModelsResponse, ConfigResponse,
    ConnectionTestRequest, ConnectionTestResponse,
    HealthResponse, IngestRequest, IngestResponse,
    IndexStatsResponse, LLMSettingsResponse, LLMSettingsUpdateRequest,
    OllamaModelsResponse, OllamaStatusResponse, PromptsResponse, PromptsUpdateRequest,
    SessionDetailResponse, SessionListResponse,
    SourceContext, SwitchModelRequest, SwitchModelResponse, UploadResponse,
    UploadedFilesListResponse,
};

const BASE: &str = "http://localhost:8000/api";

/// Send a chat message and return the response.
#[allow(dead_code)]
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

/// SSE streaming event from `/api/chat/stream`.
#[derive(Clone, Debug)]
pub enum StreamEvent {
    Sources(Vec<String>),
    Contexts(Vec<SourceContext>),
    Token(String),
    Done,
    Error(String),
}

/// Stream chat tokens via SSE. Calls the callback with each event.
/// Returns when the stream finishes or errors.
pub async fn stream_chat(
    message: &str,
    mut on_event: impl FnMut(StreamEvent),
) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use web_sys::{Request, RequestInit, Response};

    let body = serde_json::json!({ "message": message });
    let body_str = body.to_string();

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body_str));

    let request = Request::new_with_str_and_init(&format!("{BASE}/chat/stream"), &opts)
        .map_err(|e| format!("Request creation failed: {:?}", e))?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header set failed: {:?}", e))?;

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response cast failed".to_string())?;

    if !resp.ok() {
        let text = wasm_bindgen_futures::JsFuture::from(
            resp.text().map_err(|_| "Failed to read error body".to_string())?,
        )
        .await
        .map_err(|_| "Failed to await error body".to_string())?;
        let body = text.as_string().unwrap_or_default();
        return Err(format!("Server error ({}): {}", resp.status(), body));
    }

    // Read SSE stream via ReadableStream
    let body_stream = resp.body().ok_or("No response body")?;
    let reader = body_stream
        .get_reader()
        .dyn_into::<web_sys::ReadableStreamDefaultReader>()
        .map_err(|_| "Failed to get reader".to_string())?;

    let mut buffer = String::new();

    loop {
        let chunk_promise = reader.read();
        let chunk_result = wasm_bindgen_futures::JsFuture::from(chunk_promise)
            .await
            .map_err(|e| format!("Read failed: {:?}", e))?;

        let done = js_sys::Reflect::get(&chunk_result, &"done".into())
            .unwrap_or(wasm_bindgen::JsValue::TRUE)
            .as_bool()
            .unwrap_or(true);

        if !done {
            if let Ok(value) = js_sys::Reflect::get(&chunk_result, &"value".into()) {
                let arr: js_sys::Uint8Array = js_sys::Uint8Array::new(&value);
                let mut buf = vec![0u8; arr.length() as usize];
                arr.copy_to(&mut buf);
                let decoded = String::from_utf8_lossy(&buf).to_string();
                buffer.push_str(&decoded);

                // Parse SSE events from buffer
                while let Some(event) = parse_next_sse_event(&mut buffer) {
                    on_event(event);
                }
            }
        }

        if done {
            // Process any remaining buffer
            while let Some(event) = parse_next_sse_event(&mut buffer) {
                on_event(event);
            }
            break;
        }
    }

    Ok(())
}

/// Parse a single SSE event from the buffer, consuming it.
fn parse_next_sse_event(buffer: &mut String) -> Option<StreamEvent> {
    // SSE events are separated by double newline
    let end_idx = buffer.find("\n\n")?;
    let event_text: String = buffer.drain(..end_idx + 2).collect();

    let mut event_type = String::new();
    let mut data = String::new();

    for line in event_text.lines() {
        if let Some(rest) = line.strip_prefix("event: ") {
            event_type = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("data: ") {
            data = rest.to_string();
        }
    }

    match event_type.as_str() {
        "sources" => {
            let sources: Vec<String> = serde_json::from_str(&data).unwrap_or_default();
            Some(StreamEvent::Sources(sources))
        }
        "contexts" => {
            let contexts: Vec<SourceContext> = serde_json::from_str(&data).unwrap_or_default();
            Some(StreamEvent::Contexts(contexts))
        }
        "token" => {
            let token: String = serde_json::from_str(&data).unwrap_or_default();
            Some(StreamEvent::Token(token))
        }
        "done" => Some(StreamEvent::Done),
        "error" => {
            let msg: String = serde_json::from_str(&data).unwrap_or(data);
            Some(StreamEvent::Error(msg))
        }
        _ => None,
    }
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

/// Switch the active Ollama model.
pub async fn switch_model(model: &str) -> Result<SwitchModelResponse, String> {
    let client = reqwest::Client::new();
    let body = SwitchModelRequest {
        model: model.to_string(),
    };
    let resp = client
        .put(format!("{BASE}/ollama/model"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<SwitchModelResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Upload files via multipart form data, auto-ingest on the backend.
/// Uses web_sys FormData + fetch for true browser file upload.
pub async fn upload_files(
    file_data: Vec<(String, Vec<u8>)>,
    tags: Option<String>,
    version: Option<String>,
) -> Result<UploadResponse, String> {
    use js_sys::{Array, Uint8Array};
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, FormData, Request, RequestInit, Response};

    let form = FormData::new().map_err(|e| format!("FormData::new failed: {:?}", e))?;

    for (name, bytes) in &file_data {
        // Create Blob from bytes
        let uint8 = Uint8Array::from(bytes.as_slice());
        let parts = Array::new();
        parts.push(&uint8);
        let bag = BlobPropertyBag::new();
        bag.set_type("application/octet-stream");
        let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &bag)
            .map_err(|e| format!("Blob creation failed: {:?}", e))?;

        form.append_with_blob_and_filename("files", &blob, name)
            .map_err(|e| format!("FormData append failed: {:?}", e))?;
    }

    if let Some(t) = &tags {
        form.append_with_str("tags", t)
            .map_err(|e| format!("FormData append tags failed: {:?}", e))?;
    }
    if let Some(v) = &version {
        form.append_with_str("version", v)
            .map_err(|e| format!("FormData append version failed: {:?}", e))?;
    }

    // Use fetch API directly (reqwest doesn't support FormData with Blob in WASM)
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&form);

    let url = format!("{BASE}/upload");
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Request creation failed: {:?}", e))?;

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response cast failed".to_string())?;

    if !resp.ok() {
        let text = wasm_bindgen_futures::JsFuture::from(
            resp.text().map_err(|_| "Failed to read error body".to_string())?,
        )
        .await
        .map_err(|_| "Failed to await error body".to_string())?;
        let body = text.as_string().unwrap_or_default();
        return Err(format!("Server error ({}): {}", resp.status(), body));
    }

    let json_val = wasm_bindgen_futures::JsFuture::from(
        resp.json().map_err(|_| "Failed to parse JSON".to_string())?,
    )
    .await
    .map_err(|_| "Failed to await JSON".to_string())?;

    let upload_resp: UploadResponse = serde_wasm_bindgen::from_value(json_val)
        .map_err(|e| format!("Deserialization failed: {:?}", e))?;

    Ok(upload_resp)
}

// ── Session history API ─────────────────────────────────

/// Fetch all session summaries from the server.
pub async fn fetch_sessions() -> Result<SessionListResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/sessions"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<SessionListResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Fetch a full session with messages from the server.
pub async fn fetch_session(session_id: &str) -> Result<SessionDetailResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/sessions/{session_id}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<SessionDetailResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Save (create or update) a session on the server.
pub async fn save_session(
    id: &str,
    title: &str,
    messages: &[crate::models::ChatMessage],
    documents: &[String],
) -> Result<(), String> {
    let msg_payload: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| !m.streaming)
        .map(|m| {
            serde_json::json!({
                "role": match m.role {
                    crate::models::Role::User => "user",
                    crate::models::Role::Assistant => "assistant",
                },
                "text": m.text,
                "sources": m.sources,
            })
        })
        .collect();

    let body = serde_json::json!({
        "id": id,
        "title": title,
        "messages": msg_payload,
        "documents": documents,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE}/sessions"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    Ok(())
}

/// Delete a session on the server.
pub async fn delete_session(session_id: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .delete(format!("{BASE}/sessions/{session_id}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    Ok(())
}

/// Clear all chat sessions on the server.
pub async fn clear_all_sessions() -> Result<u64, String> {
    let client = reqwest::Client::new();
    let resp = client
        .delete(format!("{BASE}/sessions"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    let val: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {e}"))?;
    Ok(val.get("deleted").and_then(|v| v.as_u64()).unwrap_or(0))
}

// ── Prompts API ─────────────────────────────────────────

/// Fetch current system and RAG prompts.
pub async fn fetch_prompts() -> Result<PromptsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/prompts"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<PromptsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Update system and/or RAG prompts.
pub async fn update_prompts(req: &PromptsUpdateRequest) -> Result<PromptsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{BASE}/prompts"))
        .json(req)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<PromptsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

// ── Uploaded files API ──────────────────────────────────

/// List uploaded files.
pub async fn fetch_uploaded_files() -> Result<UploadedFilesListResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/uploads"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<UploadedFilesListResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Delete an uploaded file.
pub async fn delete_uploaded_file(filename: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .delete(format!("{BASE}/uploads/{filename}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    Ok(())
}

// ── Index API ───────────────────────────────────────────

/// Fetch vector index statistics.
pub async fn fetch_index_stats() -> Result<IndexStatsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/index/stats"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<IndexStatsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Rebuild (clear) the vector index.
pub async fn rebuild_index() -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE}/index/rebuild"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    Ok(())
}

// ── LLM Settings API ───────────────────────────────────

/// Fetch LLM/RAG tuning parameters.
pub async fn fetch_llm_settings() -> Result<LLMSettingsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/settings/llm"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<LLMSettingsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Update LLM/RAG tuning parameters.
pub async fn update_llm_settings(req: &LLMSettingsUpdateRequest) -> Result<LLMSettingsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{BASE}/settings/llm"))
        .json(req)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<LLMSettingsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

// ── API Keys ─────────────────────────────────────────────

/// Fetch current API key configuration (key value is never returned).
pub async fn fetch_api_keys() -> Result<ApiKeysResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/settings/api-keys"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<ApiKeysResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Update API key configuration.
pub async fn update_api_keys(req: &ApiKeysUpdateRequest) -> Result<ApiKeysResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{BASE}/settings/api-keys"))
        .json(req)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<ApiKeysResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Clear the vector index completely.
pub async fn clear_index() -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE}/index/clear"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    Ok(())
}

// ── Cloud Models & Connection Test ───────────────────────

/// Fetch available models from the configured cloud provider.
pub async fn fetch_cloud_models() -> Result<CloudModelsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/cloud/models"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<CloudModelsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Test the cloud API connection.
pub async fn test_cloud_connection(req: &ConnectionTestRequest) -> Result<ConnectionTestResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE}/cloud/test-connection"))
        .json(req)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<ConnectionTestResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

// ── API Profiles ─────────────────────────────────────────

/// List all saved API profiles.
pub async fn fetch_profiles() -> Result<ApiProfileListResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{BASE}/profiles"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<ApiProfileListResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Create a new API profile.
pub async fn create_profile(profile: &ApiProfileCreate) -> Result<ApiProfileSummary, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE}/profiles"))
        .json(profile)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    resp.json::<ApiProfileSummary>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Delete an API profile.
pub async fn delete_profile(profile_id: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .delete(format!("{BASE}/profiles/{profile_id}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    Ok(())
}

/// Activate a saved API profile.
pub async fn activate_profile(profile_id: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{BASE}/profiles/{profile_id}/activate"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Server error ({status}): {text}"));
    }
    Ok(())
}
