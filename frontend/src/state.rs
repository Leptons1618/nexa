//! Global application state shared across all pages via context.
//!
//! Provides cached data that persists across page navigations,
//! preventing redundant API calls when switching between Chat and Settings.

use dioxus::prelude::*;

use crate::models::{ChatMessage, ChatSession, OllamaModelEntry, UploadedDoc};

/// Shared application state — provided at the root, read by any page.
#[derive(Clone, Copy)]
pub struct AppState {
    /// Cached list of available Ollama models.
    pub models: Signal<Vec<OllamaModelEntry>>,
    /// Currently selected / active model name.
    pub selected_model: Signal<String>,
    /// Whether the initial data load has completed.
    pub loaded: Signal<bool>,
}

/// Chat-specific state — lives at the root, survives page navigation.
#[derive(Clone, Copy)]
pub struct ChatState {
    /// Messages for the active conversation.
    pub messages: Signal<Vec<ChatMessage>>,
    /// All known chat sessions.
    pub sessions: Signal<Vec<ChatSession>>,
    /// ID of the active session.
    pub active_session_id: Signal<String>,
    /// Uploaded docs for the active session.
    pub uploaded_docs: Signal<Vec<UploadedDoc>>,
    /// Whether chat data has been loaded from the server.
    pub chat_loaded: Signal<bool>,
}
