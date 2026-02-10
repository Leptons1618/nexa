//! Global application state shared across all pages via context.
//!
//! Provides cached data that persists across page navigations,
//! preventing redundant API calls when switching between Chat and Settings.

use dioxus::prelude::*;

use crate::models::OllamaModelEntry;

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
