//! Settings page — view current backend configuration.

use dioxus::prelude::*;

use crate::api;
use crate::components::icons::{IconCpu, IconDatabase, IconSettings as IconSettingsIcon, IconCheckCircle, IconAlertCircle};

#[component]
pub fn Settings() -> Element {
    let config = use_resource(|| async { api::fetch_config().await });
    let health = use_resource(|| async { api::fetch_health().await });
    let ollama_status = use_resource(|| async { api::fetch_ollama_status().await });
    let ollama_models = use_resource(|| async { api::fetch_ollama_models().await });

    rsx! {
        div { class: "settings-page",
            div { class: "settings-header",
                h1 { class: "settings-title",
                    IconSettingsIcon { size: 24 }
                    "Settings"
                }
                // Status indicators
                div { class: "settings-status",
                    match &*health.read() {
                        Some(Ok(h)) if h.status == "ok" => rsx! {
                            span { class: "status-pill status-pill--ok",
                                IconCheckCircle { size: 14 }
                                "System Healthy"
                            }
                        },
                        Some(Ok(_)) => rsx! {
                            span { class: "status-pill status-pill--warn",
                                IconAlertCircle { size: 14 }
                                "Degraded"
                            }
                        },
                        Some(Err(_)) => rsx! {
                            span { class: "status-pill status-pill--error",
                                IconAlertCircle { size: 14 }
                                "Unreachable"
                            }
                        },
                        None => rsx! {
                            span { class: "status-pill status-pill--loading", "Checking..." }
                        },
                    }
                }
            }

            div { class: "settings-grid",
                // LLM card
                div { class: "setting-card",
                    div { class: "setting-card-icon setting-card-icon--blue",
                        IconCpu { size: 20 }
                    }
                    h3 { "LLM Provider" }
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
                            p { class: "setting-error", "Failed to load: {e}" }
                        },
                        None => rsx! {
                            div { class: "setting-skeleton" }
                            div { class: "setting-skeleton" }
                        },
                    }
                }

                // Ollama status card
                div { class: "setting-card",
                    div { class: "setting-card-icon setting-card-icon--green",
                        IconCpu { size: 20 }
                    }
                    h3 { "Ollama Server" }
                    match &*ollama_status.read() {
                        Some(Ok(st)) => rsx! {
                            div { class: "setting-row",
                                span { class: "label", "Running" }
                                span { class: if st.running { "value value--ok" } else { "value value--error" },
                                    if st.running {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Base URL" }
                                span { class: "value value--mono", "{st.base_url}" }
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
                            p { class: "setting-error", "Ollama not available: {e}" }
                        },
                        None => rsx! {
                            div { class: "setting-skeleton" }
                            div { class: "setting-skeleton" }
                        },
                    }
                }

                // Vector store card
                div { class: "setting-card",
                    div { class: "setting-card-icon setting-card-icon--purple",
                        IconDatabase { size: 20 }
                    }
                    h3 { "Vector Store" }
                    match &*config.read() {
                        Some(Ok(cfg)) => rsx! {
                            div { class: "setting-row",
                                span { class: "label", "Backend" }
                                span { class: "value", "{cfg.vector_store}" }
                            }
                            div { class: "setting-row",
                                span { class: "label", "Embedding Model" }
                                span { class: "value value--mono", "{cfg.embedding_model}" }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "setting-error", "Failed to load: {e}" }
                        },
                        None => rsx! {
                            div { class: "setting-skeleton" }
                        },
                    }
                }

                // Health card
                div { class: "setting-card",
                    div { class: "setting-card-icon setting-card-icon--amber",
                        IconCpu { size: 20 }
                    }
                    h3 { "Service Health" }
                    match &*health.read() {
                        Some(Ok(h)) => rsx! {
                            div { class: "setting-row",
                                span { class: "label", "Status" }
                                span { class: if h.status == "ok" { "value value--ok" } else { "value value--error" }, "{h.status}" }
                            }
                            div { class: "setting-row",
                                span { class: "label", "LLM Connected" }
                                span { class: if h.llm_connected { "value value--ok" } else { "value value--error" },
                                    if h.llm_connected {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                }
                            }
                            if !h.detail.is_empty() {
                                div { class: "setting-row",
                                    span { class: "label", "Detail" }
                                    span { class: "value", "{h.detail}" }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "setting-error", "Failed to load: {e}" }
                        },
                        None => rsx! {
                            div { class: "setting-skeleton" }
                        },
                    }
                }

                // Ollama models card (wider)
                div { class: "setting-card setting-card--wide",
                    div { class: "setting-card-icon setting-card-icon--blue",
                        IconDatabase { size: 20 }
                    }
                    h3 { "Available Models" }
                    match &*ollama_models.read() {
                        Some(Ok(m)) => rsx! {
                            if m.models.is_empty() {
                                p { class: "setting-empty", "No models found on Ollama server." }
                            } else {
                                div { class: "models-table",
                                    div { class: "models-table-header",
                                        span { "Name" }
                                        span { "Size" }
                                    }
                                    for model in m.models.iter() {
                                        div { class: "models-table-row",
                                            span { class: "model-name", "{model.name}" }
                                            span { class: "model-size", {format_model_size(model.size)} }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "setting-error", "Could not list models: {e}" }
                        },
                        None => rsx! {
                            div { class: "setting-skeleton" }
                            div { class: "setting-skeleton" }
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
