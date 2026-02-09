//! Upload modal — real browser file upload with drag-and-drop,
//! multipart upload to backend, and auto-ingestion.

use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use crate::api;
use crate::components::icons::{IconFileText, IconUpload, IconTrash};
use crate::models::UploadedDoc;

/// Allowed file extensions (must match backend).
const ALLOWED_EXTENSIONS: &[&str] = &[
    ".pdf", ".txt", ".md", ".doc", ".docx",
    ".html", ".htm", ".json", ".xml", ".csv", ".rst",
];

/// Maximum single file size (50 MB).
const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// A file selected in the browser, with bytes loaded via FileReader.
#[derive(Clone, Debug)]
struct PendingFile {
    name: String,
    size: u64,
    data: Vec<u8>,
    error: Option<String>,
}

/// Format bytes into a human-readable size string.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let kb = bytes as f64 / 1024.0;
    if kb < 1024.0 {
        return format!("{kb:.1} KB");
    }
    let mb = kb / 1024.0;
    format!("{mb:.1} MB")
}

/// Check if a filename has an allowed extension.
fn is_valid_extension(name: &str) -> bool {
    let lower = name.to_lowercase();
    ALLOWED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Read a browser File into bytes using FileReader + oneshot channel.
async fn read_file_bytes(file: web_sys::File) -> Result<Vec<u8>, String> {
    let (tx, rx) = futures_channel::oneshot::channel::<Result<Vec<u8>, String>>();

    let reader = web_sys::FileReader::new()
        .map_err(|e| format!("FileReader::new failed: {:?}", e))?;

    let reader_clone = reader.clone();
    let onload = wasm_bindgen::closure::Closure::once(move |_event: web_sys::Event| {
        let result = reader_clone.result();
        match result {
            Ok(val) => {
                let array = js_sys::Uint8Array::new(&val);
                let mut bytes = vec![0u8; array.length() as usize];
                array.copy_to(&mut bytes);
                let _ = tx.send(Ok(bytes));
            }
            Err(e) => {
                let _ = tx.send(Err(format!("FileReader result error: {:?}", e)));
            }
        }
    });

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    reader
        .read_as_array_buffer(&file)
        .map_err(|e| format!("read_as_array_buffer failed: {:?}", e))?;
    onload.forget();

    rx.await.map_err(|_| "FileReader channel cancelled".to_string())?
}

#[component]
pub fn UploadModal(
    show: Signal<bool>,
    uploaded_docs: Signal<Vec<UploadedDoc>>,
) -> Element {
    let mut pending_files = use_signal(Vec::<PendingFile>::new);
    let mut tags_input = use_signal(|| String::new());
    let mut version_input = use_signal(|| String::new());
    let mut uploading = use_signal(|| false);
    let mut result_msg = use_signal(|| Option::<String>::None);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // If not visible, render nothing
    if !*show.read() {
        return rsx! {};
    }

    // Handle file input change — reads files via FileReader
    let on_file_input = move |_evt: Event<FormData>| {
        spawn(async move {
            // Get the file input element by ID
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };
            let doc = match window.document() {
                Some(d) => d,
                None => return,
            };
            let elem: web_sys::HtmlInputElement = match doc
                .get_element_by_id("upload-file-input")
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                Some(e) => e,
                None => return,
            };

            let file_list = match elem.files() {
                Some(fl) => fl,
                None => return,
            };

            let count = file_list.length();
            for i in 0..count {
                if let Some(file) = file_list.get(i) {
                    let name = file.name();
                    let size = file.size() as u64;

                    // Validate extension
                    if !is_valid_extension(&name) {
                        pending_files.write().push(PendingFile {
                            name,
                            size,
                            data: vec![],
                            error: Some("Unsupported file type".to_string()),
                        });
                        continue;
                    }

                    // Validate size
                    if size > MAX_FILE_SIZE {
                        pending_files.write().push(PendingFile {
                            name,
                            size,
                            data: vec![],
                            error: Some("File too large (max 50 MB)".to_string()),
                        });
                        continue;
                    }

                    // Read bytes
                    match read_file_bytes(file).await {
                        Ok(bytes) => {
                            pending_files.write().push(PendingFile {
                                name,
                                size,
                                data: bytes,
                                error: None,
                            });
                        }
                        Err(e) => {
                            pending_files.write().push(PendingFile {
                                name,
                                size,
                                data: vec![],
                                error: Some(format!("Read error: {e}")),
                            });
                        }
                    }
                }
            }

            // Reset the input so the same files can be re-selected
            elem.set_value("");
        });
    };

    // Upload handler
    let on_upload = move |_: MouseEvent| {
        let valid_files: Vec<(String, Vec<u8>)> = pending_files
            .read()
            .iter()
            .filter(|f| f.error.is_none() && !f.data.is_empty())
            .map(|f| (f.name.clone(), f.data.clone()))
            .collect();

        if valid_files.is_empty() {
            error_msg.set(Some("No valid files to upload".to_string()));
            return;
        }

        let tags = tags_input.read().clone();
        let version = version_input.read().clone();
        let tags_opt = if tags.trim().is_empty() {
            None
        } else {
            Some(tags.trim().to_string())
        };
        let version_opt = if version.trim().is_empty() {
            None
        } else {
            Some(version.trim().to_string())
        };

        uploading.set(true);
        error_msg.set(None);
        result_msg.set(None);

        spawn(async move {
            match api::upload_files(valid_files, tags_opt.clone(), version_opt).await {
                Ok(resp) => {
                    let tag_list: Vec<String> = tags_opt
                        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                        .unwrap_or_default();

                    // Add uploaded files to session documents
                    let mut docs = uploaded_docs.write();
                    for file_info in &resp.files {
                        docs.push(UploadedDoc {
                            name: file_info.original_name.clone(),
                            path: file_info.saved_path.clone(),
                            chunks: resp.chunks_indexed / resp.files.len().max(1) as u64,
                            size: file_info.size,
                            tags: tag_list.clone(),
                        });
                    }

                    result_msg.set(Some(format!(
                        "Uploaded {} file(s) — {} chunks indexed",
                        resp.files.len(),
                        resp.chunks_indexed,
                    )));
                    pending_files.set(vec![]);
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
            uploading.set(false);
        });
    };

    // Remove a pending file
    let mut remove_file = move |idx: usize| {
        pending_files.write().remove(idx);
    };

    // Close modal
    let on_close = move |_: MouseEvent| {
        show.set(false);
        result_msg.set(None);
        error_msg.set(None);
    };

    let has_valid = pending_files
        .read()
        .iter()
        .any(|f| f.error.is_none() && !f.data.is_empty());

    rsx! {
        div { class: "modal-backdrop", onclick: on_close,
            div {
                class: "modal modal--upload",
                onclick: move |evt: MouseEvent| evt.stop_propagation(),

                // Header
                div { class: "modal-header",
                    h3 {
                        IconUpload { size: 20 }
                        "Upload Documents"
                    }
                    button { class: "modal-close", onclick: on_close, "×" }
                }

                // Body
                div { class: "modal-body",
                    // Dropzone / file picker
                    div { class: "upload-dropzone",
                        input {
                            r#type: "file",
                            id: "upload-file-input",
                            class: "upload-input",
                            multiple: true,
                            accept: ".pdf,.txt,.md,.doc,.docx,.html,.htm,.json,.xml,.csv,.rst",
                            onchange: on_file_input,
                        }
                        label { r#for: "upload-file-input", class: "upload-label",
                            IconUpload { size: 36 }
                            span { class: "upload-label-text", "Choose files or drag & drop" }
                            span { class: "upload-label-hint",
                                "PDF, TXT, MD, DOC, DOCX, HTML, JSON, XML, CSV, RST — max 50 MB each"
                            }
                        }
                    }

                    // File preview list
                    if !pending_files.read().is_empty() {
                        div { class: "file-preview-list",
                            div { class: "file-preview-title",
                                "Selected Files ({pending_files.read().len()})"
                            }
                            for (idx , file) in pending_files.read().iter().enumerate() {
                                div { class: if file.error.is_some() { "file-preview-item file-preview-item--invalid" } else { "file-preview-item" },
                                    div { class: "file-preview-icon",
                                        IconFileText { size: 16 }
                                    }
                                    div { class: "file-preview-info",
                                        div { class: "file-preview-name", "{file.name}" }
                                        div { class: "file-preview-size", "{format_size(file.size)}" }
                                        if let Some(err) = &file.error {
                                            div { class: "file-preview-error", "{err}" }
                                        }
                                    }
                                    button {
                                        class: "file-preview-remove",
                                        onclick: move |_| remove_file(idx),
                                        IconTrash { size: 14 }
                                    }
                                }
                            }
                        }
                    }

                    // Tags + version fields
                    div { class: "form-group", style: "margin-top: 20px;",
                        label { class: "form-label", "Tags (optional)" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "e.g. api, guide, v2",
                            value: "{tags_input}",
                            oninput: move |evt: Event<FormData>| tags_input.set(evt.value()),
                        }
                        div { class: "form-hint", "Comma-separated tags for document metadata" }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Version (optional)" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "e.g. 2.0",
                            value: "{version_input}",
                            oninput: move |evt: Event<FormData>| version_input.set(evt.value()),
                        }
                    }

                    // Result / error messages
                    if let Some(msg) = &*result_msg.read() {
                        div { class: "alert alert--success", "{msg}" }
                    }
                    if let Some(msg) = &*error_msg.read() {
                        div { class: "alert alert--error", "{msg}" }
                    }
                }

                // Footer
                div { class: "modal-footer",
                    button { class: "btn btn--ghost", onclick: on_close, "Close" }
                    button {
                        class: "btn btn--primary",
                        disabled: *uploading.read() || !has_valid,
                        onclick: on_upload,
                        if *uploading.read() {
                            "Uploading…"
                        } else {
                            "Upload & Index"
                        }
                    }
                }
            }
        }
    }
}
