//! Settings page — view current backend configuration.

use dioxus::prelude::*;

use crate::api;
use crate::components::header::Header;
use crate::components::icons::{IconCpu, IconDatabase, IconSettings as IconSettingsIcon};

#[component]
pub fn Settings() -> Element {
    let config = use_resource(|| async { api::fetch_config().await });
    let health = use_resource(|| async { api::fetch_health().await });
    let ollama_status = use_resource(|| async { api::fetch_ollama_status().await });
    let ollama_models = use_resource(|| async { api::fetch_ollama_models().await });

    rsx! {
        Header { title: "Settings".to_string() }
        div { class: "page-content",
            h2 { class: "page-title",
                IconSettingsIcon { size: 24 }
                "Configuration"
            }

            div { class: "settings-grid",
                // LLM card
                div { class: "setting-card",
                    h3 {
                        IconCpu { size: 16 }
                        "LLM Provider"
                    }
                    match &*config.read() {
                        Some(Ok(cfg)) => rsx! {
                            div { class: "setting-row",
                                span { class: "label", "Provider" }
                                span { class: "value", "{cfg.llm_provider}" }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Model" }
                                span { class: "value", "{cfg.model_name}" }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "form-hint", "Failed to load: {e}" }
                        },
                        None => rsx! {
                            p { class: "form-hint", "Loading..." }
                        },
                    }
                }

                // Ollama status card
                div { class: "setting-card",
                    h3 {
                        IconCpu { size: 16 }
                        "Ollama Server"
                    }
                    match &*ollama_status.read() {
                        Some(Ok(st)) => rsx! {
                            div { class: "setting-row",
                                span { class: "label", "Running" }
                                span { class: "value",
                                    if st.running {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Base URL" }
                                span { class: "value", "{st.base_url}" }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Active Model" }
                                span { class: "value", "{st.model}" }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Models Available" }
                                span { class: "value", "{st.models_available}" }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "form-hint", "Ollama not available: {e}" }
                        },
                        None => rsx! {
                            p { class: "form-hint", "Loading..." }
                        },
                    }
                }

                // Vector store card
                div { class: "setting-card",
                    h3 {
                        IconDatabase { size: 16 }
                        "Vector Store"
                    }
                    match &*config.read() {
                        Some(Ok(cfg)) => rsx! {
                            div { class: "setting-row",
                                span { class: "label", "Backend" }
                                span { class: "value", "{cfg.vector_store}" }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Embedding Model" }
                                span { class: "value", "{cfg.embedding_model}" }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "form-hint", "Failed to load: {e}" }
                        },
                        None => rsx! {
                            p { class: "form-hint", "Loading..." }
                        },
                    }
                }

                // Health card
                div { class: "setting-card",
                    h3 {
                        IconCpu { size: 16 }
                        "Service Health"
                    }
                    match &*health.read() {
                        Some(Ok(h)) => rsx! {
                            div { class: "setting-row",
                                span { class: "label", "Status" }
                                span { class: "value", "{h.status}" }
                            }
                            div { class: "setting-row",
                                span { class: "label", "LLM Connected" }
                                span { class: "value",
                                    if h.llm_connected {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Detail" }
                                span { class: "value", "{h.detail}" }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "form-hint", "Failed to load: {e}" }
                        },
                        None => rsx! {
                            p { class: "form-hint", "Loading..." }
                        },
                    }
                }

                // Ollama models card
                div { class: "setting-card",
                    h3 {
                        IconDatabase { size: 16 }
                        "Available Models"
                    }
                    match &*ollama_models.read() {
                        Some(Ok(m)) => rsx! {
                            if m.models.is_empty() {
                                p { class: "form-hint", "No models found." }
                            } else {
                                for model in m.models.iter() {
                                    div { class: "setting-row",
                                        span { class: "label", "{model.name}" }
                                        span { class: "value", {format_model_size(model.size)} }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "form-hint", "Could not list models: {e}" }
                        },
                        None => rsx! {
                            p { class: "form-hint", "Loading..." }
                        },
                    }
                }
            }
        }
    }
}

/// Human-readable file size.
fn format_model_size(bytes: Option<u64>) -> String {
    match bytes {
        Some(b) if b >= 1_000_000_000 => format!("{:.1} GB", b as f64 / 1_000_000_000.0),
        Some(b) if b >= 1_000_000 => format!("{:.1} MB", b as f64 / 1_000_000.0),
        Some(b) => format!("{b} B"),
        None => "—".to_string(),
    }
}
