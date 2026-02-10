//! Settings page — comprehensive configuration management.

use dioxus::prelude::*;

use crate::api;
use crate::components::icons::{
    IconAlertCircle, IconCheckCircle, IconCpu, IconDatabase, IconEdit, IconFolder, IconHardDrive,
    IconMessageCircle, IconRefreshCw, IconSave, IconSettings as IconSettingsIcon, IconSliders,
    IconTrash,
};
use crate::models::{LLMSettingsUpdateRequest, PromptsUpdateRequest};

#[component]
pub fn Settings() -> Element {
    // ── Data resources ──────────────────────────────────
    let health = use_resource(|| async { api::fetch_health().await });
    let ollama_status = use_resource(|| async { api::fetch_ollama_status().await });
    let ollama_models = use_resource(|| async { api::fetch_ollama_models().await });
    let prompts_res = use_resource(|| async { api::fetch_prompts().await });
    let uploads_res = use_resource(|| async { api::fetch_uploaded_files().await });
    let mut index_res = use_resource(|| async { api::fetch_index_stats().await });
    let llm_settings_res = use_resource(|| async { api::fetch_llm_settings().await });
    let sessions_res = use_resource(|| async { api::fetch_sessions().await });
    let config_res = use_resource(|| async { api::fetch_config().await });

    // ── Editable state ──────────────────────────────────
    let mut system_prompt = use_signal(|| String::new());
    let mut rag_prompt = use_signal(|| String::new());
    let mut prompts_loaded = use_signal(|| false);
    let mut prompts_saving = use_signal(|| false);
    let mut prompts_msg = use_signal(|| Option::<(bool, String)>::None);

    let mut temperature = use_signal(|| String::from("0.2"));
    let mut top_p = use_signal(|| String::from("0.9"));
    let mut max_tokens = use_signal(|| String::from("512"));
    let mut chunk_size = use_signal(|| String::from("400"));
    let mut chunk_overlap = use_signal(|| String::from("80"));
    let mut top_k = use_signal(|| String::from("4"));
    let mut similarity_threshold = use_signal(|| String::from("0.35"));
    let mut llm_loaded = use_signal(|| false);
    let mut llm_saving = use_signal(|| false);
    let mut llm_msg = use_signal(|| Option::<(bool, String)>::None);

    let mut switching_model = use_signal(|| false);
    let mut model_msg = use_signal(|| Option::<(bool, String)>::None);

    let mut action_msg = use_signal(|| Option::<(bool, String)>::None);

    // ── Hydrate editable fields from API data ───────────
    if !*prompts_loaded.read() {
        if let Some(Ok(p)) = &*prompts_res.read() {
            system_prompt.set(p.system_prompt.clone());
            rag_prompt.set(p.rag_addon_prompt.clone());
            prompts_loaded.set(true);
        }
    }
    if !*llm_loaded.read() {
        if let Some(Ok(s)) = &*llm_settings_res.read() {
            temperature.set(format!("{}", s.temperature));
            top_p.set(format!("{}", s.top_p));
            max_tokens.set(format!("{}", s.max_tokens));
            chunk_size.set(format!("{}", s.chunk_size));
            chunk_overlap.set(format!("{}", s.chunk_overlap));
            top_k.set(format!("{}", s.top_k));
            similarity_threshold.set(format!("{}", s.similarity_threshold));
            llm_loaded.set(true);
        }
    }

    // ── Handlers ────────────────────────────────────────
    let save_prompts = move |_| {
        let sys = system_prompt.read().clone();
        let rag = rag_prompt.read().clone();
        prompts_saving.set(true);
        prompts_msg.set(None);
        spawn(async move {
            let req = PromptsUpdateRequest {
                system_prompt: Some(sys),
                rag_addon_prompt: Some(rag),
            };
            match api::update_prompts(&req).await {
                Ok(_) => prompts_msg.set(Some((true, "Prompts saved.".into()))),
                Err(e) => prompts_msg.set(Some((false, format!("Failed: {e}")))),
            }
            prompts_saving.set(false);
        });
    };

    let save_llm_settings = move |_| {
        let temp = temperature.read().parse::<f64>().ok();
        let tp = top_p.read().parse::<f64>().ok();
        let mt = max_tokens.read().parse::<u64>().ok();
        let cs = chunk_size.read().parse::<u64>().ok();
        let co = chunk_overlap.read().parse::<u64>().ok();
        let tk = top_k.read().parse::<u64>().ok();
        let st = similarity_threshold.read().parse::<f64>().ok();
        llm_saving.set(true);
        llm_msg.set(None);
        spawn(async move {
            let req = LLMSettingsUpdateRequest {
                temperature: temp,
                top_p: tp,
                max_tokens: mt,
                chunk_size: cs,
                chunk_overlap: co,
                top_k: tk,
                similarity_threshold: st,
            };
            match api::update_llm_settings(&req).await {
                Ok(_) => llm_msg.set(Some((true, "Settings saved.".into()))),
                Err(e) => llm_msg.set(Some((false, format!("Failed: {e}")))),
            }
            llm_saving.set(false);
        });
    };

    let on_switch_model = move |evt: Event<FormData>| {
        let new_model = evt.value();
        if new_model.is_empty() {
            return;
        }
        switching_model.set(true);
        model_msg.set(None);
        spawn(async move {
            match api::switch_model(&new_model).await {
                Ok(resp) => {
                    model_msg.set(Some((true, format!("Switched to {}", resp.current_model))));
                }
                Err(e) => {
                    model_msg.set(Some((false, format!("Failed: {e}"))));
                }
            }
            switching_model.set(false);
        });
    };

    let clear_history = move |_| {
        action_msg.set(None);
        spawn(async move {
            match api::clear_all_sessions().await {
                Ok(n) => {
                    action_msg.set(Some((true, format!("Cleared {n} sessions."))));
                }
                Err(e) => action_msg.set(Some((false, format!("Failed: {e}")))),
            }
        });
    };

    let rebuild_idx = move |_| {
        action_msg.set(None);
        spawn(async move {
            match api::rebuild_index().await {
                Ok(_) => {
                    action_msg.set(Some((
                        true,
                        "Index cleared. Re-ingest documents to rebuild.".into(),
                    )));
                    index_res.restart();
                }
                Err(e) => action_msg.set(Some((false, format!("Failed: {e}")))),
            }
        });
    };

    let uploads_state = uploads_res.read().clone();
    let delete_upload = move |name: String| {
        let mut uploads_res = uploads_res.clone();
        let mut action_msg = action_msg.clone();
        spawn(async move {
            match api::delete_uploaded_file(&name).await {
                Ok(_) => uploads_res.restart(),
                Err(e) => action_msg.set(Some((false, format!("Delete failed: {e}")))),
            }
        });
    };

    let uploads_view = match &uploads_state {
        Some(Ok(u)) => rsx! {
            if u.files.is_empty() {
                p { class: "setting-empty", "No uploaded files." }
            } else {
                div { class: "setting-file-list",
                    for file in u.files.iter().cloned() {
                        div { class: "setting-file-item",
                            div { class: "setting-file-info",
                                span { class: "setting-file-name", "{file.name}" }
                                span { class: "setting-file-meta", "{format_size(file.size)}" }
                            }
                            button {
                                class: "btn-icon btn-icon--danger",
                                title: "Delete file",
                                onclick: move |_| delete_upload(file.name.clone()),
                                IconTrash { size: 14 }
                            }
                        }
                    }
                }
                p { class: "setting-field-hint", "{u.files.len()} file(s) total" }
            }
        },
        Some(Err(e)) => rsx! {
            p { class: "setting-inline-msg setting-inline-msg--error", "Failed to load files: {e}" }
        },
        None => rsx! {
            div { class: "setting-skeleton" }
        },
    };

    let prompts_feedback = match prompts_msg.read().as_ref() {
        Some((ok, msg)) => {
            let class_name = if *ok {
                "setting-inline-msg setting-inline-msg--ok"
            } else {
                "setting-inline-msg setting-inline-msg--error"
            };
            rsx! {
                span { class: class_name, {msg.clone()} }
            }
        }
        None => rsx! {},
    };

    rsx! {
        div { class: "settings-page",
            // ── Header ──────────────────────────────────
            div { class: "settings-header",
                h1 { class: "settings-title",
                    IconSettingsIcon { size: 24 }
                    "Settings"
                }
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

            // ── Global action message ───────────────────
            {
                let action_val = action_msg.read().clone();
                match action_val {
                    Some((true, ref msg)) => rsx! {
                        div { class: "settings-alert settings-alert--ok",
                            IconCheckCircle { size: 14 }
                            "{msg}"
                        }
                    },
                    Some((false, ref msg)) => rsx! {
                        div { class: "settings-alert settings-alert--error",
                            IconAlertCircle { size: 14 }
                            "{msg}"
                        }
                    },
                    None => rsx! {},
                }
            }

            div { class: "settings-sections",

                // ══════════════════════════════════════════
                // 1. LLM Configuration
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--blue",
                            IconCpu { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "LLM Configuration" }
                            p { class: "settings-section-desc",
                                "Provider, model selection, and generation parameters."
                            }
                        }
                    }

                    div { class: "settings-section-body",
                        // Provider + Status row
                        div { class: "settings-row-group",
                            match &*config_res.read() {
                                Some(Ok(cfg)) => rsx! {
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Provider" }
                                        div { class: "setting-field-value", "{cfg.llm_provider}" }
                                    }
                                },
                                _ => rsx! {
                                    div { class: "setting-skeleton" }
                                },
                            }
                            match &*ollama_status.read() {
                                Some(Ok(st)) => {
                                    let status_text = if st.running { "Running" } else { "Offline" };
                                    rsx! {
                                        div { class: "setting-field setting-field--readonly",
                                            label { class: "setting-field-label", "Server Status" }
                                            div { class: if st.running { "setting-field-value setting-field-value--ok" } else { "setting-field-value setting-field-value--error" },
                                                span { "{status_text}" }
                                                span { class: "setting-field-hint", " ({st.base_url})" }
                                            }
                                        }
                                    }
                                }
                                _ => rsx! {
                                    div { class: "setting-skeleton" }
                                },
                            }
                        }

                        // Active model selector — uses Ollama status for the real model
                        div { class: "setting-field",
                            label { class: "setting-field-label", "Active Model" }
                            match (&*ollama_models.read(), &*ollama_status.read()) {
                                (Some(Ok(m)), Some(Ok(st))) => rsx! {
                                    div { class: "setting-model-row",
                                        select {
                                            class: "setting-select",
                                            disabled: *switching_model.read(),
                                            value: "{st.model}",
                                            onchange: on_switch_model,
                                            for model in m.models.iter() {
                                                option { value: "{model.name}", selected: model.name == st.model,
                                                    "{model.name}"
                                                    {
                                                        if let Some(sz) = model.size {
                                                            format!(" ({})", format_size(sz))
                                                        } else {
                                                            String::new()
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if *switching_model.read() {
                                            span { class: "setting-field-hint setting-field-hint--warn", "Switching..." }
                                        }
                                    }
                                    if let Some((ok, msg)) = model_msg.read().as_ref() {
                                        p { class: if *ok { "setting-inline-msg setting-inline-msg--ok" } else { "setting-inline-msg setting-inline-msg--error" },
                                            "{msg}"
                                        }
                                    }
                                },
                                (Some(Err(e)), _) => rsx! {
                                    p { class: "setting-inline-msg setting-inline-msg--error", "Could not load models: {e}" }
                                },
                                _ => rsx! {
                                    div { class: "setting-skeleton" }
                                },
                            }
                        }

                        // Generation parameters
                        div { class: "setting-field",
                            label { class: "setting-field-label", "Generation Parameters" }
                            div { class: "settings-param-grid",
                                div { class: "setting-param",
                                    label { "Temperature" }
                                    input {
                                        class: "setting-input setting-input--sm",
                                        r#type: "number",
                                        step: "0.05",
                                        min: "0",
                                        max: "2",
                                        value: "{temperature}",
                                        oninput: move |e| temperature.set(e.value()),
                                    }
                                }
                                div { class: "setting-param",
                                    label { "Top P" }
                                    input {
                                        class: "setting-input setting-input--sm",
                                        r#type: "number",
                                        step: "0.05",
                                        min: "0",
                                        max: "1",
                                        value: "{top_p}",
                                        oninput: move |e| top_p.set(e.value()),
                                    }
                                }
                                div { class: "setting-param",
                                    label { "Max Tokens" }
                                    input {
                                        class: "setting-input setting-input--sm",
                                        r#type: "number",
                                        step: "64",
                                        min: "1",
                                        value: "{max_tokens}",
                                        oninput: move |e| max_tokens.set(e.value()),
                                    }
                                }
                            }
                        }

                        div { class: "settings-section-actions",
                            button {
                                class: "btn btn--primary btn--sm",
                                disabled: *llm_saving.read(),
                                onclick: save_llm_settings,
                                IconSave { size: 14 }
                                if *llm_saving.read() {
                                    "Saving..."
                                } else {
                                    "Save LLM Settings"
                                }
                            }
                            if let Some((ok, msg)) = llm_msg.read().as_ref() {
                                span { class: if *ok { "setting-inline-msg setting-inline-msg--ok" } else { "setting-inline-msg setting-inline-msg--error" },
                                    "{msg}"
                                }
                            }
                        }
                    }
                }

                // ══════════════════════════════════════════
                // 2. RAG Configuration
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--purple",
                            IconSliders { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "RAG Configuration" }
                            p { class: "settings-section-desc",
                                "Retrieval-augmented generation tuning parameters."
                            }
                        }
                    }
                    div { class: "settings-section-body",
                        match &*config_res.read() {
                            Some(Ok(cfg)) => rsx! {
                                div { class: "settings-row-group",
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Vector Store" }
                                        div { class: "setting-field-value", "{cfg.vector_store}" }
                                    }
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Embedding Model" }
                                        div { class: "setting-field-value setting-field-value--mono", "{cfg.embedding_model}" }
                                    }
                                }
                            },
                            _ => rsx! {
                                div { class: "setting-skeleton" }
                            },
                        }

                        div { class: "settings-param-grid",
                            div { class: "setting-param",
                                label { "Chunk Size" }
                                input {
                                    class: "setting-input setting-input--sm",
                                    r#type: "number",
                                    step: "50",
                                    min: "100",
                                    value: "{chunk_size}",
                                    oninput: move |e| chunk_size.set(e.value()),
                                }
                            }
                            div { class: "setting-param",
                                label { "Chunk Overlap" }
                                input {
                                    class: "setting-input setting-input--sm",
                                    r#type: "number",
                                    step: "10",
                                    min: "0",
                                    value: "{chunk_overlap}",
                                    oninput: move |e| chunk_overlap.set(e.value()),
                                }
                            }
                            div { class: "setting-param",
                                label { "Top K" }
                                input {
                                    class: "setting-input setting-input--sm",
                                    r#type: "number",
                                    step: "1",
                                    min: "1",
                                    max: "20",
                                    value: "{top_k}",
                                    oninput: move |e| top_k.set(e.value()),
                                }
                            }
                            div { class: "setting-param",
                                label { "Similarity Threshold" }
                                input {
                                    class: "setting-input setting-input--sm",
                                    r#type: "number",
                                    step: "0.05",
                                    min: "0",
                                    max: "1",
                                    value: "{similarity_threshold}",
                                    oninput: move |e| similarity_threshold.set(e.value()),
                                }
                            }
                        }

                        div { class: "settings-section-actions",
                            button {
                                class: "btn btn--primary btn--sm",
                                disabled: *llm_saving.read(),
                                onclick: save_llm_settings,
                                IconSave { size: 14 }
                                if *llm_saving.read() {
                                    "Saving..."
                                } else {
                                    "Save RAG Settings"
                                }
                            }
                        }
                    }
                }

                // ══════════════════════════════════════════
                // 3. Prompts
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--green",
                            IconEdit { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "Prompts" }
                            p { class: "settings-section-desc",
                                "Edit the system prompt and RAG context addon."
                            }
                        }
                    }
                    div { class: "settings-section-body",
                        if let Some(Err(e)) = &*prompts_res.read() {
                            p { class: "setting-inline-msg setting-inline-msg--error",
                                "Failed to load prompts: {e}"
                            }
                        }
                        div { class: "setting-field",
                            label { class: "setting-field-label", "System Prompt" }
                            textarea {
                                class: "setting-textarea",
                                rows: "6",
                                value: "{system_prompt}",
                                oninput: move |e| system_prompt.set(e.value()),
                            }
                        }
                        div { class: "setting-field",
                            label { class: "setting-field-label", "RAG Context Addon" }
                            textarea {
                                class: "setting-textarea",
                                rows: "4",
                                value: "{rag_prompt}",
                                oninput: move |e| rag_prompt.set(e.value()),
                            }
                        }
                        div { class: "settings-section-actions",
                            button {
                                class: "btn btn--primary btn--sm",
                                disabled: *prompts_saving.read(),
                                onclick: save_prompts,
                                IconSave { size: 14 }
                                if *prompts_saving.read() {
                                    "Saving..."
                                } else {
                                    "Save Prompts"
                                }
                            }
                            {prompts_feedback}
                        }
                    }
                }

                // ══════════════════════════════════════════
                // 4. Uploaded Files
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--amber",
                            IconFolder { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "Uploaded Files" }
                            p { class: "settings-section-desc",
                                "Manage documents uploaded to the system."
                            }
                        }
                    }
                    div { class: "settings-section-body", {uploads_view} }
                }

                // ══════════════════════════════════════════
                // 5. Chat History
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--blue",
                            IconMessageCircle { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "Chat History" }
                            p { class: "settings-section-desc",
                                "View and manage saved chat sessions."
                            }
                        }
                    }
                    div { class: "settings-section-body",
                        match &*sessions_res.read() {
                            Some(Ok(s)) => rsx! {
                                div { class: "settings-row-group",
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Total Sessions" }
                                        div { class: "setting-field-value", "{s.sessions.len()}" }
                                    }
                                }
                                if !s.sessions.is_empty() {
                                    div { class: "setting-session-list",
                                        for sess in s.sessions.iter().take(10) {
                                            div { class: "setting-session-item",
                                                div { class: "setting-session-info",
                                                    span { class: "setting-session-title", "{sess.title}" }
                                                    span { class: "setting-session-meta",
                                                        "{sess.message_count} msgs · {sess.created_at}"
                                                    }
                                                }
                                            }
                                        }
                                        if s.sessions.len() > 10 {
                                            p { class: "setting-field-hint", "... and {s.sessions.len() - 10} more sessions." }
                                        }
                                    }
                                }
                                div { class: "settings-section-actions",
                                    button {
                                        class: "btn btn--ghost btn--sm btn--danger-text",
                                        onclick: clear_history,
                                        IconTrash { size: 14 }
                                        "Clear All History"
                                    }
                                }
                            },
                            Some(Err(e)) => rsx! {
                                p { class: "setting-inline-msg setting-inline-msg--error", "Failed: {e}" }
                            },
                            None => rsx! {
                                div { class: "setting-skeleton" }
                            },
                        }
                    }
                }

                // ══════════════════════════════════════════
                // 6. Vector Index
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--purple",
                            IconHardDrive { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "Vector Index" }
                            p { class: "settings-section-desc",
                                "FAISS index statistics and management."
                            }
                        }
                    }
                    div { class: "settings-section-body",
                        match &*index_res.read() {
                            Some(Ok(idx)) => rsx! {
                                div { class: "settings-row-group",
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Total Vectors" }
                                        div { class: "setting-field-value", "{idx.total_vectors}" }
                                    }
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Metadata Entries" }
                                        div { class: "setting-field-value", "{idx.total_metadata}" }
                                    }
                                }
                                div { class: "settings-row-group",
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Index Size" }
                                        div { class: "setting-field-value", "{format_size(idx.index_size_bytes)}" }
                                    }
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Metadata Size" }
                                        div { class: "setting-field-value", "{format_size(idx.metadata_size_bytes)}" }
                                    }
                                }
                                div { class: "settings-row-group",
                                    div { class: "setting-field setting-field--readonly",
                                        label { class: "setting-field-label", "Index Path" }
                                        div { class: "setting-field-value setting-field-value--mono", "{idx.index_path}" }
                                    }
                                }
                                div { class: "settings-section-actions",
                                    button {
                                        class: "btn btn--ghost btn--sm btn--danger-text",
                                        onclick: rebuild_idx,
                                        IconRefreshCw { size: 14 }
                                        "Rebuild Index"
                                    }
                                }
                            },
                            Some(Err(e)) => rsx! {
                                p { class: "setting-inline-msg setting-inline-msg--error", "Failed: {e}" }
                            },
                            None => rsx! {
                                div { class: "setting-skeleton" }
                                div { class: "setting-skeleton" }
                            },
                        }
                    }
                }

                // ══════════════════════════════════════════
                // 7. Available Models
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--blue",
                            IconDatabase { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "Available Models" }
                            p { class: "settings-section-desc",
                                "Models installed on the Ollama server."
                            }
                        }
                    }
                    div { class: "settings-section-body",
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
                                                span { class: "model-size", {format_size(model.size.unwrap_or(0))} }
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Err(e)) => rsx! {
                                p { class: "setting-inline-msg setting-inline-msg--error", "Could not list models: {e}" }
                            },
                            None => rsx! {
                                div { class: "setting-skeleton" }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Human-readable file size.
fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} B")
    }
}
