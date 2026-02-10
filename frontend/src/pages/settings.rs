//! Settings page — comprehensive configuration management with API keys,
//! confirmation modals, cloud model fetching, connection testing,
//! multi-profile support, and toast notifications.

use dioxus::prelude::*;

use crate::api;
use crate::components::confirm_modal::ConfirmModal;
use crate::components::icons::{
    IconAlertCircle, IconCheckCircle, IconCpu, IconDatabase, IconEdit, IconFolder, IconHardDrive,
    IconKey, IconMessageCircle, IconRefreshCw, IconSave, IconSettings as IconSettingsIcon,
    IconSliders, IconTrash,
};
use crate::components::notifications::use_notifications;
use crate::models::{
    ApiKeysUpdateRequest, ApiProfileCreate, ConnectionTestRequest, LLMSettingsUpdateRequest,
    PromptsUpdateRequest,
};

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
    let api_keys_res = use_resource(|| async { api::fetch_api_keys().await });

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

    // API Keys state
    let mut api_provider = use_signal(|| String::from("ollama"));
    let mut api_key_input = use_signal(|| String::new());
    let mut api_base_url = use_signal(|| String::from("https://api.openai.com/v1"));
    let mut api_cloud_model = use_signal(|| String::from("gpt-4"));
    let mut api_keys_loaded = use_signal(|| false);
    let mut api_keys_saving = use_signal(|| false);
    let mut api_keys_msg = use_signal(|| Option::<(bool, String)>::None);

    // Cloud model fetching state
    let mut cloud_models = use_signal(Vec::<String>::new);
    let mut cloud_models_loading = use_signal(|| false);

    // Connection test state
    let mut connection_testing = use_signal(|| false);

    // API Profiles state
    let mut profiles_res = use_resource(|| async { api::fetch_profiles().await });
    let mut new_profile_name = use_signal(|| String::new());
    let mut profile_saving = use_signal(|| false);
    let mut show_confirm_delete_profile = use_signal(|| false);
    let mut pending_delete_profile_id = use_signal(|| String::new());

    // Notification handle
    let mut toasts = use_notifications();

    // Confirmation modal state
    let mut show_confirm_clear_history = use_signal(|| false);
    let mut show_confirm_rebuild = use_signal(|| false);
    let mut show_confirm_clear_idx = use_signal(|| false);
    let mut show_confirm_delete_file = use_signal(|| false);
    let mut pending_delete_filename = use_signal(|| String::new());

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
    if !*api_keys_loaded.read() {
        if let Some(Ok(k)) = &*api_keys_res.read() {
            api_provider.set(k.llm_provider.clone());
            api_base_url.set(k.cloud_base_url.clone());
            api_cloud_model.set(k.cloud_model.clone());
            api_keys_loaded.set(true);
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

    let save_api_keys = move |_| {
        api_keys_saving.set(true);
        api_keys_msg.set(None);
        let provider = api_provider.read().clone();
        let key = api_key_input.read().clone();
        let url = api_base_url.read().clone();
        let model = api_cloud_model.read().clone();
        spawn(async move {
            let req = ApiKeysUpdateRequest {
                llm_provider: Some(provider),
                cloud_api_key: if key.is_empty() { None } else { Some(key) },
                cloud_base_url: Some(url),
                cloud_model: Some(model),
            };
            match api::update_api_keys(&req).await {
                Ok(_) => {
                    api_keys_msg.set(Some((true, "API configuration saved.".into())));
                    api_key_input.set(String::new());
                    toasts.success("Saved", "API configuration updated.");
                }
                Err(e) => {
                    api_keys_msg.set(Some((false, format!("Failed: {e}"))));
                    toasts.error("Error", format!("Save failed: {e}"));
                }
            }
            api_keys_saving.set(false);
        });
    };

    // Fetch cloud models from the provider
    let fetch_cloud_models = move |_| {
        cloud_models_loading.set(true);
        spawn(async move {
            match api::fetch_cloud_models().await {
                Ok(resp) => {
                    let ids: Vec<String> = resp.models.iter().map(|m| m.id.clone()).collect();
                    let count = ids.len();
                    cloud_models.set(ids);
                    toasts.success("Models Loaded", format!("{count} model(s) fetched from provider."));
                }
                Err(e) => {
                    toasts.error("Fetch Failed", format!("Could not load models: {e}"));
                }
            }
            cloud_models_loading.set(false);
        });
    };

    // Test connection
    let test_connection = move |_| {
        connection_testing.set(true);
        let url = api_base_url.read().clone();
        let key = api_key_input.read().clone();
        spawn(async move {
            let req = ConnectionTestRequest {
                base_url: Some(url),
                api_key: if key.is_empty() { None } else { Some(key) },
            };
            match api::test_cloud_connection(&req).await {
                Ok(resp) => {
                    if resp.success {
                        toasts.success("Connected", resp.message);
                    } else {
                        toasts.warning("Connection Failed", resp.message);
                    }
                }
                Err(e) => {
                    toasts.error("Test Error", format!("Request failed: {e}"));
                }
            }
            connection_testing.set(false);
        });
    };

    // Save current config as a new profile
    let save_as_profile = move |_| {
        let name = new_profile_name.read().clone();
        if name.trim().is_empty() {
            toasts.warning("Missing Name", "Enter a profile name first.");
            return;
        }
        profile_saving.set(true);
        let provider = api_provider.read().clone();
        let key = api_key_input.read().clone();
        let url = api_base_url.read().clone();
        let model = api_cloud_model.read().clone();
        spawn(async move {
            let profile = ApiProfileCreate {
                id: String::new(), // server will assign
                name: name.clone(),
                llm_provider: provider,
                cloud_api_key: if key.is_empty() { None } else { Some(key) },
                cloud_base_url: url,
                cloud_model: model,
            };
            match api::create_profile(&profile).await {
                Ok(_) => {
                    toasts.success("Profile Saved", format!("'{name}' created."));
                    new_profile_name.set(String::new());
                    profiles_res.restart();
                }
                Err(e) => {
                    toasts.error("Error", format!("Could not save profile: {e}"));
                }
            }
            profile_saving.set(false);
        });
    };

    // Activate a profile
    let activate_profile = move |id: String, name: String| {
        spawn(async move {
            match api::activate_profile(&id).await {
                Ok(_) => {
                    toasts.success("Activated", format!("Profile '{name}' is now active."));
                    profiles_res.restart();
                    // Refresh the API keys view
                    api_keys_loaded.set(false);
                }
                Err(e) => {
                    toasts.error("Error", format!("Activation failed: {e}"));
                }
            }
        });
    };

    // Delete a profile (triggered after confirm)
    let do_delete_profile = move |_: ()| {
        let pid = pending_delete_profile_id.read().clone();
        if pid.is_empty() {
            return;
        }
        spawn(async move {
            match api::delete_profile(&pid).await {
                Ok(_) => {
                    toasts.success("Deleted", "Profile removed.");
                    profiles_res.restart();
                }
                Err(e) => {
                    toasts.error("Error", format!("Delete failed: {e}"));
                }
            }
        });
    };

    // Confirmation-guarded actions
    let do_clear_history = move |_: ()| {
        action_msg.set(None);
        spawn(async move {
            match api::clear_all_sessions().await {
                Ok(n) => {
                    action_msg.set(Some((true, format!("Cleared {n} sessions."))));
                    toasts.success("History Cleared", format!("{n} session(s) deleted."));
                }
                Err(e) => {
                    action_msg.set(Some((false, format!("Failed: {e}"))));
                    toasts.error("Error", format!("Clear failed: {e}"));
                }
            }
        });
    };

    let do_rebuild_idx = move |_: ()| {
        action_msg.set(None);
        spawn(async move {
            match api::rebuild_index().await {
                Ok(_) => {
                    action_msg.set(Some((
                        true,
                        "Index rebuilt. Re-ingest documents to populate.".into(),
                    )));
                    index_res.restart();
                    toasts.success("Index Rebuilt", "Re-ingest documents to populate.");
                }
                Err(e) => {
                    action_msg.set(Some((false, format!("Failed: {e}"))));
                    toasts.error("Error", format!("Rebuild failed: {e}"));
                }
            }
        });
    };

    let do_clear_index = move |_: ()| {
        action_msg.set(None);
        spawn(async move {
            match api::clear_index().await {
                Ok(_) => {
                    action_msg.set(Some((true, "Vector index cleared.".into())));
                    index_res.restart();
                    toasts.success("Index Cleared", "All vectors and metadata removed.");
                }
                Err(e) => {
                    action_msg.set(Some((false, format!("Failed: {e}"))));
                    toasts.error("Error", format!("Clear failed: {e}"));
                }
            }
        });
    };

    let uploads_state = uploads_res.read().clone();
    let do_delete_file = move |_: ()| {
        let name = pending_delete_filename.read().clone();
        if name.is_empty() {
            return;
        }
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
                                onclick: move |_| {
                                    pending_delete_filename.set(file.name.clone());
                                    show_confirm_delete_file.set(true);
                                },
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
                            span { class: "status-pill status-pill--loading",
                                span { class: "spinner" }
                                "Checking..."
                            }
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
                // 0. Cloud API & Provider Configuration
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--purple",
                            IconKey { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "API Keys & Provider" }
                            p { class: "settings-section-desc",
                                "Configure the LLM provider and cloud API credentials."
                            }
                        }
                    }
                    div { class: "settings-section-body",
                        div { class: "setting-field",
                            label { class: "setting-field-label", "LLM Provider" }
                            select {
                                class: "setting-select",
                                value: "{api_provider}",
                                onchange: move |e: Event<FormData>| api_provider.set(e.value()),
                                option { value: "ollama", "Ollama (Local)" }
                                option { value: "cloud", "Cloud API (OpenAI-compatible)" }
                            }
                        }

                        {
                            let is_cloud = api_provider.read().as_str() == "cloud";
                            let api_key_set = api_keys_res
                                .read()
                                .as_ref()
                                .and_then(|r| r.as_ref().ok())
                                .map(|k| k.cloud_api_key_set)
                                .unwrap_or(false);
                            let placeholder_text = if api_key_set {
                                "Key is set — enter new value to replace"
                            } else {
                                "Enter your API key"
                            };
                            let fetched_models = cloud_models.read().clone();
                            let has_fetched = !fetched_models.is_empty();
                            if is_cloud {
                                rsx! {
                                    div { class: "setting-field",
                                        label { class: "setting-field-label", "API Key" }
                                        input {
                                            class: "setting-input",
                                            r#type: "password",
                                            placeholder: "{placeholder_text}",
                                            value: "{api_key_input}",
                                            oninput: move |e| api_key_input.set(e.value()),
                                        }
                                        p { class: "setting-field-hint", "Stored in server memory only. Never sent to the frontend." }
                                    }
                                    div { class: "settings-row-group",
                                        div { class: "setting-field",
                                            label { class: "setting-field-label", "Base URL" }
                                            input {
                                                class: "setting-input",
                                                r#type: "text",
                                                placeholder: "https://api.openai.com/v1",
                                                value: "{api_base_url}",
                                                oninput: move |e| api_base_url.set(e.value()),
                                            }
                                            p { class: "setting-field-hint", "OpenAI, Groq, Together, or any compatible endpoint." }
                                        }
                                        div { class: "setting-field",
                                            label { class: "setting-field-label", "Cloud Model" }
                                            if has_fetched {
                                                select {
                                                    class: "setting-select",
                                                    value: "{api_cloud_model}",
                                                    onchange: move |e: Event<FormData>| api_cloud_model.set(e.value()),
                                                    for model_id in fetched_models.iter() {
                                                        option { value: "{model_id}", "{model_id}" }
                                                    }
                                                }
                                            } else {
                                                input {
                                                    class: "setting-input",
                                                    r#type: "text",
                                                    placeholder: "gpt-4",
                                                    value: "{api_cloud_model}",
                                                    oninput: move |e| api_cloud_model.set(e.value()),
                                                }
                                            }
                                        }
                                    }
                                    div { class: "settings-section-actions",
                                        button {
                                            class: "btn btn--ghost btn--sm",
                                            disabled: *cloud_models_loading.read(),
                                            onclick: fetch_cloud_models,
                                            IconRefreshCw { size: 14 }
                                            if *cloud_models_loading.read() {
                                                "Fetching…"
                                            } else {
                                                "Fetch Models"
                                            }
                                        }
                                        button {
                                            class: "btn btn--ghost btn--sm",
                                            disabled: *connection_testing.read(),
                                            onclick: test_connection,
                                            IconCheckCircle { size: 14 }
                                            if *connection_testing.read() {
                                                "Testing…"
                                            } else {
                                                "Test Connection"
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }

                        div { class: "settings-section-actions",
                            button {
                                class: "btn btn--primary btn--sm",
                                disabled: *api_keys_saving.read(),
                                onclick: save_api_keys,
                                IconSave { size: 14 }
                                if *api_keys_saving.read() {
                                    "Saving…"
                                } else {
                                    "Save Provider Config"
                                }
                            }
                            if let Some((ok, msg)) = api_keys_msg.read().as_ref() {
                                span { class: if *ok { "setting-inline-msg setting-inline-msg--ok" } else { "setting-inline-msg setting-inline-msg--error" },
                                    "{msg}"
                                }
                            }
                        }
                    }
                }

                // ══════════════════════════════════════════
                // 0b. API Profiles
                // ══════════════════════════════════════════
                div { class: "settings-section",
                    div { class: "settings-section-header",
                        div { class: "settings-section-icon settings-section-icon--green",
                            IconFolder { size: 20 }
                        }
                        div {
                            h2 { class: "settings-section-title", "API Profiles" }
                            p { class: "settings-section-desc",
                                "Save and switch between multiple API configurations."
                            }
                        }
                    }
                    div { class: "settings-section-body",
                        // Saved profiles list
                        match &*profiles_res.read() {
                            Some(Ok(data)) => {
                                let active_id = data.active_profile_id.clone().unwrap_or_default();
                                rsx! {
                                    if data.profiles.is_empty() {
                                        p { class: "setting-empty", "No saved profiles yet." }
                                    } else {
                                        div { class: "setting-profile-list",
                                            for profile in data.profiles.iter().cloned() {
                                                {
                                                    let is_active = profile.id == active_id;
                                                    let pid = profile.id.clone();
                                                    let pname = profile.name.clone();
                                                    rsx! {
                                                        div { class: if is_active { "setting-profile-item setting-profile-item--active" } else { "setting-profile-item" },
                                                            div { class: "setting-profile-info",
                                                                span { class: "setting-profile-name",
                                                                    "{profile.name}"
                                                                    if is_active {
                                                                        span { class: "profile-badge", "Active" }
                                                                    }
                                                                }
                                                                span { class: "setting-profile-meta", "{profile.llm_provider} · {profile.cloud_model}" }
                                                            }
                                                            div { class: "setting-profile-actions",
                                                                if !is_active {
                                                                    button {
                                                                        class: "btn-icon",
                                                                        title: "Activate",
                                                                        onclick: move |_| activate_profile(pid.clone(), pname.clone()),
                                                                        IconCheckCircle { size: 14 }
                                                                    }
                                                                }
                                                                button {
                                                                    class: "btn-icon btn-icon--danger",
                                                                    title: "Delete profile",
                                                                    onclick: move |_| {
                                                                        pending_delete_profile_id.set(profile.id.clone());
                                                                        show_confirm_delete_profile.set(true);
                                                                    },
                                                                    IconTrash { size: 14 }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => rsx! {
                                p { class: "setting-inline-msg setting-inline-msg--error", "Failed: {e}" }
                            },
                            None => rsx! {
                                div { class: "setting-skeleton" }
                            },
                        }

                        // Save current as new profile
                        div { class: "settings-row-group",
                            div { class: "setting-field",
                                label { class: "setting-field-label", "Save Current as Profile" }
                                div { class: "setting-input-group",
                                    input {
                                        class: "setting-input",
                                        r#type: "text",
                                        placeholder: "Profile name (e.g. 'OpenAI GPT-4')",
                                        value: "{new_profile_name}",
                                        oninput: move |e| new_profile_name.set(e.value()),
                                    }
                                    button {
                                        class: "btn btn--primary btn--sm",
                                        disabled: *profile_saving.read(),
                                        onclick: save_as_profile,
                                        IconSave { size: 14 }
                                        if *profile_saving.read() {
                                            "Saving…"
                                        } else {
                                            "Save Profile"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

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
                                            span { class: "setting-field-hint setting-field-hint--warn",
                                                span { class: "spinner" }
                                                "Switching..."
                                            }
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
                                            p { class: "setting-field-hint", "… and {s.sessions.len() - 10} more sessions." }
                                        }
                                    }
                                }
                                div { class: "settings-section-actions",
                                    button {
                                        class: "btn btn--ghost btn--sm btn--danger-text",
                                        onclick: move |_| show_confirm_clear_history.set(true),
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
                                        onclick: move |_| show_confirm_clear_idx.set(true),
                                        IconTrash { size: 14 }
                                        "Clear Index"
                                    }
                                    button {
                                        class: "btn btn--ghost btn--sm",
                                        onclick: move |_| show_confirm_rebuild.set(true),
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

            // ── Confirmation Modals ─────────────────────
            {
                let delete_msg = {
                    let name = pending_delete_filename.read().clone();
                    if name.is_empty() {
                        "Are you sure you want to delete this file?".to_string()
                    } else {
                        format!("Delete '{name}'? This action cannot be undone.")
                    }
                };
                rsx! {
                    ConfirmModal {
                        show: show_confirm_clear_history,
                        title: "Clear Chat History".to_string(),
                        message: "This will permanently delete all chat sessions. This action cannot be undone."
                            .to_string(),
                        confirm_text: Some("Clear All".to_string()),
                        danger: Some(true),
                        on_confirm: do_clear_history,
                    }
                    ConfirmModal {
                        show: show_confirm_rebuild,
                        title: "Rebuild Index".to_string(),
                        message: "This will delete the current vector index. You will need to re-ingest documents afterward."
                            .to_string(),
                        confirm_text: Some("Rebuild".to_string()),
                        danger: Some(true),
                        on_confirm: do_rebuild_idx,
                    }
                    ConfirmModal {
                        show: show_confirm_clear_idx,
                        title: "Clear Vector Index".to_string(),
                        message: "This will permanently delete all indexed vectors and metadata.".to_string(),
                        confirm_text: Some("Clear Index".to_string()),
                        danger: Some(true),
                        on_confirm: do_clear_index,
                    }
                    ConfirmModal {
                        show: show_confirm_delete_file,
                        title: "Delete File".to_string(),
                        message: delete_msg,
                        confirm_text: Some("Delete".to_string()),
                        danger: Some(true),
                        on_confirm: do_delete_file,
                    }
                    ConfirmModal {
                        show: show_confirm_delete_profile,
                        title: "Delete Profile".to_string(),
                        message: "Are you sure you want to delete this API profile?".to_string(),
                        confirm_text: Some("Delete".to_string()),
                        danger: Some(true),
                        on_confirm: do_delete_profile,
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
