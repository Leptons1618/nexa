//! Documents page — ingest files into the vector store.

use dioxus::prelude::*;

use crate::api;
use crate::components::header::Header;
use crate::components::icons::{IconAlertCircle, IconCheckCircle, IconFileText, IconUpload};

#[component]
pub fn Documents() -> Element {
    let mut paths_input = use_signal(|| String::new());
    let mut tags_input = use_signal(|| String::new());
    let mut version_input = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut result_msg = use_signal(|| Option::<(bool, String)>::None);

    let on_submit = move |_| {
        let raw_paths = paths_input.read().clone();
        let raw_tags = tags_input.read().clone();
        let raw_version = version_input.read().clone();

        let paths: Vec<String> = raw_paths
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if paths.is_empty() {
            result_msg.set(Some((false, "Please enter at least one path.".into())));
            return;
        }

        let tags: Option<Vec<String>> = {
            let t: Vec<String> = raw_tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if t.is_empty() { None } else { Some(t) }
        };

        let version = if raw_version.trim().is_empty() {
            None
        } else {
            Some(raw_version.trim().to_string())
        };

        loading.set(true);
        result_msg.set(None);

        spawn(async move {
            match api::ingest_documents(paths, tags, version).await {
                Ok(resp) => {
                    result_msg.set(Some((
                        true,
                        format!("Successfully indexed {} chunks.", resp.chunks_indexed),
                    )));
                    paths_input.set(String::new());
                    tags_input.set(String::new());
                    version_input.set(String::new());
                }
                Err(err) => {
                    result_msg.set(Some((false, format!("Ingestion failed: {err}"))));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        Header { title: "Documents".to_string() }
        div { class: "page-content",
            h2 { class: "page-title",
                IconFileText { size: 24 }
                "Ingest Documents"
            }

            div { class: "form-card",
                div { class: "form-group",
                    label { class: "form-label", "Paths" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "docs/, README.md, data/guide.pdf",
                        value: "{paths_input}",
                        oninput: move |e| paths_input.set(e.value()),
                    }
                    p { class: "form-hint", "Comma-separated file or directory paths." }
                }

                div { class: "form-group",
                    label { class: "form-label", "Tags (optional)" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "release, v2, internal",
                        value: "{tags_input}",
                        oninput: move |e| tags_input.set(e.value()),
                    }
                    p { class: "form-hint", "Comma-separated metadata tags." }
                }

                div { class: "form-group",
                    label { class: "form-label", "Version (optional)" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "1.0.0",
                        value: "{version_input}",
                        oninput: move |e| version_input.set(e.value()),
                    }
                }

                button {
                    class: "btn btn--primary",
                    disabled: *loading.read(),
                    onclick: on_submit,
                    IconUpload { size: 16 }
                    if *loading.read() {
                        "Ingesting..."
                    } else {
                        "Ingest"
                    }
                }

                if let Some((ok, msg)) = result_msg.read().as_ref() {
                    div { class: if *ok { "alert alert--success" } else { "alert alert--error" },
                        if *ok {
                            IconCheckCircle { size: 16 }
                        } else {
                            IconAlertCircle { size: 16 }
                        }
                        "{msg}"
                    }
                }
            }
        }
    }
}
