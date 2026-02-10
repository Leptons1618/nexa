//! Chat page — main conversation interface with collapsible sidebar, SSE streaming,
//! server-side history persistence, and documents flyout above input.

use dioxus::prelude::*;

use crate::api;
use crate::api::StreamEvent;
use crate::components::chat_bubble::ChatBubble;
use crate::components::chat_history::ChatHistory;
use crate::components::chat_input::ChatInput;
use crate::components::confirm_modal::ConfirmModal;
use crate::components::icons::{
    IconChevronRight, IconFileCode, IconFileText, IconMessageSquare,
    IconSettings, IconUpload, IconX,
};
use crate::components::model_selector::ModelSelector;
use crate::components::upload_modal::UploadModal;
use crate::models::{ChatMessage, ChatSession, Role, UploadedDoc};
use crate::state::{AppState, ChatState};
use crate::Route;

/// Generate a simple unique id using a static counter.
fn new_id() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("s-{id}")
}

/// Format bytes into human-readable size.
fn format_doc_size(bytes: u64) -> String {
    if bytes == 0 {
        return String::new();
    }
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

/// Derive a short title from the first user message.
fn title_from_messages(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .find(|m| m.role == Role::User)
        .map(|m| {
            let t: String = m.text.chars().take(40).collect();
            if m.text.len() > 40 {
                format!("{t}...")
            } else {
                t
            }
        })
        .unwrap_or_else(|| "New Chat".to_string())
}

#[component]
pub fn Chat() -> Element {
    // ── Pull chat state from global context (survives page navigation) ──
    let chat = use_context::<ChatState>();
    let mut messages = chat.messages;
    let mut sessions = chat.sessions;
    let mut active_session_id = chat.active_session_id;
    let mut uploaded_docs = chat.uploaded_docs;
    let chat_loaded = chat.chat_loaded;

    // Active conversation loading indicator
    let mut loading = use_signal(|| false);

    // Model selector — use global state
    let state = use_context::<AppState>();
    let selected_model = state.selected_model;

    // Upload modal
    let show_upload = use_signal(|| false);

    // Sidebar collapse state
    let mut sidebar_collapsed = use_signal(|| false);

    // Documents flyout state (above input)
    let mut show_docs_panel = use_signal(|| false);

    // Delete session confirmation
    let mut show_confirm_delete_session = use_signal(|| false);
    let mut pending_delete_session_id = use_signal(|| String::new());

    // Load sessions from server ONCE (not on every mount)
    {
        let sessions = sessions.clone();
        let active_session_id = active_session_id.clone();
        let messages = messages.clone();
        let uploaded_docs = uploaded_docs.clone();
        let chat_loaded = chat_loaded.clone();
        use_resource(move || {
            let mut sessions = sessions.clone();
            let mut active_session_id = active_session_id.clone();
            let mut messages = messages.clone();
            let mut uploaded_docs = uploaded_docs.clone();
            let mut chat_loaded = chat_loaded.clone();
            async move {
                // Skip if already loaded
                if *chat_loaded.read() {
                    return;
                }
                if let Ok(list) = api::fetch_sessions().await {
                    let mut loaded: Vec<ChatSession> = Vec::new();
                    for s in &list.sessions {
                        loaded.push(ChatSession {
                            id: s.id.clone(),
                            title: s.title.clone(),
                            messages: vec![],
                            documents: vec![],
                            created_at: s.created_at.clone(),
                        });
                    }
                    if !loaded.is_empty() {
                        let first_id = loaded[0].id.clone();
                        if let Ok(detail) = api::fetch_session(&first_id).await {
                            let msgs: Vec<ChatMessage> = detail
                                .messages
                                .iter()
                                .map(|m| ChatMessage {
                                    role: if m.role == "user" {
                                        Role::User
                                    } else {
                                        Role::Assistant
                                    },
                                    text: m.text.clone(),
                                    sources: m.sources.clone(),
                                    source_contexts: vec![],
                                    streaming: false,
                                })
                                .collect();
                            loaded[0].messages = msgs.clone();
                            loaded[0].documents = detail.documents.clone();
                            messages.set(msgs);
                            active_session_id.set(first_id);
                            // Restore uploaded docs from the session
                            let restored_docs: Vec<UploadedDoc> = detail
                                .documents
                                .iter()
                                .map(|p| {
                                    let name = p.rsplit(['/', '\\']).next().unwrap_or(p).to_string();
                                    UploadedDoc {
                                        name,
                                        path: p.clone(),
                                        chunks: 0,
                                        size: 0,
                                        tags: vec![],
                                    }
                                })
                                .collect();
                            uploaded_docs.set(restored_docs);
                        }
                    }
                    sessions.set(loaded);
                }
                // Ensure there's an active session ID even if no server sessions exist
                if active_session_id.read().is_empty() {
                    active_session_id.set(new_id());
                }
                chat_loaded.set(true);
            }
        });
    }

    let on_send = move |text: String| {
        messages.write().push(ChatMessage {
            role: Role::User,
            text: text.clone(),
            sources: vec![],
            source_contexts: vec![],
            streaming: false,
        });
        loading.set(true);

        messages.write().push(ChatMessage {
            role: Role::Assistant,
            text: String::new(),
            sources: vec![],
            source_contexts: vec![],
            streaming: true,
        });

        spawn(async move {
            let result = api::stream_chat(&text, |event| match event {
                StreamEvent::Token(token) => {
                    let mut msgs = messages.write();
                    if let Some(last) = msgs.last_mut() {
                        last.text.push_str(&token);
                    }
                }
                StreamEvent::Sources(sources) => {
                    let mut msgs = messages.write();
                    if let Some(last) = msgs.last_mut() {
                        last.sources = sources;
                    }
                }
                StreamEvent::Contexts(contexts) => {
                    let mut msgs = messages.write();
                    if let Some(last) = msgs.last_mut() {
                        last.source_contexts = contexts;
                    }
                }
                StreamEvent::Done => {
                    let mut msgs = messages.write();
                    if let Some(last) = msgs.last_mut() {
                        last.streaming = false;
                    }
                }
                StreamEvent::Error(err) => {
                    let mut msgs = messages.write();
                    if let Some(last) = msgs.last_mut() {
                        last.text = format!("Error: {err}");
                        last.streaming = false;
                    }
                }
            })
            .await;

            if let Err(err) = result {
                let mut msgs = messages.write();
                if let Some(last) = msgs.last_mut() {
                    if last.streaming {
                        last.text = format!("Error: {err}");
                        last.streaming = false;
                    }
                }
            }

            loading.set(false);

            // Update session in history and persist to server
            let current_id = active_session_id.read().clone();
            let current_messages = messages.read().clone();
            let current_docs: Vec<String> =
                uploaded_docs.read().iter().map(|d| d.path.clone()).collect();
            let title = title_from_messages(&current_messages);

            {
                let mut sess = sessions.write();
                if let Some(s) = sess.iter_mut().find(|s| s.id == current_id) {
                    s.messages = current_messages.clone();
                    s.documents = current_docs.clone();
                    s.title = title.clone();
                } else {
                    sess.push(ChatSession {
                        id: current_id.clone(),
                        title: title.clone(),
                        messages: current_messages.clone(),
                        documents: current_docs.clone(),
                        created_at: String::new(),
                    });
                }
            }

            // Persist to server
            let _ =
                api::save_session(&current_id, &title, &current_messages, &current_docs).await;
        });
    };

    let on_select_session = move |id: String| {
        // Save current session first
        let current_id = active_session_id.read().clone();
        let current_messages = messages.read().clone();
        if !current_messages.is_empty() {
            let current_docs: Vec<String> =
                uploaded_docs.read().iter().map(|d| d.path.clone()).collect();
            let title = title_from_messages(&current_messages);
            let mut sess = sessions.write();
            if let Some(s) = sess.iter_mut().find(|s| s.id == current_id) {
                s.messages = current_messages.clone();
                s.documents = current_docs.clone();
                s.title = title.clone();
            }
            let cid = current_id.clone();
            spawn(async move {
                let _ = api::save_session(&cid, &title, &current_messages, &current_docs).await;
            });
        }

        let selected_id = id.clone();
        active_session_id.set(id.clone());

        // Try local first
        let sess = sessions.read();
        let local_session = sess.iter().find(|s| s.id == id);

        if let Some(s) = local_session {
            if !s.messages.is_empty() {
                messages.set(s.messages.clone());
                let restored_docs: Vec<UploadedDoc> = s
                    .documents
                    .iter()
                    .map(|p| {
                        let name = p.rsplit(['/', '\\']).next().unwrap_or(p).to_string();
                        UploadedDoc {
                            name,
                            path: p.clone(),
                            chunks: 0,
                            size: 0,
                            tags: vec![],
                        }
                    })
                    .collect();
                uploaded_docs.set(restored_docs);
                return;
            }
        }
        drop(sess);

        // Fetch from server
        spawn(async move {
            if let Ok(detail) = api::fetch_session(&selected_id).await {
                let msgs: Vec<ChatMessage> = detail
                    .messages
                    .iter()
                    .map(|m| ChatMessage {
                        role: if m.role == "user" {
                            Role::User
                        } else {
                            Role::Assistant
                        },
                        text: m.text.clone(),
                        sources: m.sources.clone(),
                        source_contexts: vec![],
                        streaming: false,
                    })
                    .collect();

                let mut sess = sessions.write();
                if let Some(s) = sess.iter_mut().find(|s| s.id == selected_id) {
                    s.messages = msgs.clone();
                    s.documents = detail.documents.clone();
                }
                drop(sess);

                messages.set(msgs);
                let restored_docs: Vec<UploadedDoc> = detail
                    .documents
                    .iter()
                    .map(|p| {
                        let name = p.rsplit(['/', '\\']).next().unwrap_or(p).to_string();
                        UploadedDoc {
                            name,
                            path: p.clone(),
                            chunks: 0,
                            size: 0,
                            tags: vec![],
                        }
                    })
                    .collect();
                uploaded_docs.set(restored_docs);
            }
        });
    };

    let on_new_chat = move |_: ()| {
        // Save current session
        let current_id = active_session_id.read().clone();
        let current_messages = messages.read().clone();
        if !current_messages.is_empty() {
            let current_docs: Vec<String> =
                uploaded_docs.read().iter().map(|d| d.path.clone()).collect();
            let title = title_from_messages(&current_messages);
            let mut sess = sessions.write();
            if let Some(s) = sess.iter_mut().find(|s| s.id == current_id) {
                s.messages = current_messages.clone();
                s.documents = current_docs.clone();
                s.title = title.clone();
            } else {
                sess.push(ChatSession {
                    id: current_id.clone(),
                    title: title.clone(),
                    messages: current_messages.clone(),
                    documents: current_docs.clone(),
                    created_at: String::new(),
                });
            }
            let cid = current_id.clone();
            spawn(async move {
                let _ = api::save_session(&cid, &title, &current_messages, &current_docs).await;
            });
        }

        let new_session_id = new_id();
        active_session_id.set(new_session_id);
        messages.set(vec![]);
        uploaded_docs.set(vec![]);
    };

    let on_delete_session = move |id: String| {
        pending_delete_session_id.set(id);
        show_confirm_delete_session.set(true);
    };

    let do_delete_session = move |_: ()| {
        let id = pending_delete_session_id.read().clone();
        if id.is_empty() {
            return;
        }
        let mut sess = sessions.write();
        sess.retain(|s| s.id != id);

        if id == *active_session_id.read() {
            let new_session_id = new_id();
            active_session_id.set(new_session_id);
            messages.set(vec![]);
            uploaded_docs.set(vec![]);
        }

        let delete_id = id.clone();
        spawn(async move {
            let _ = api::delete_session(&delete_id).await;
        });
    };

    // Regenerate: remove last bot message and re-send last user message
    let on_regenerate = move |_: ()| {
        let msgs = messages.read().clone();
        // Find the last user message
        let last_user_text = msgs
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.text.clone());

        if let Some(user_text) = last_user_text {
            // Remove last bot message
            let mut new_msgs: Vec<ChatMessage> = msgs.clone();
            if let Some(last) = new_msgs.last() {
                if last.role == Role::Assistant {
                    new_msgs.pop();
                }
            }
            messages.set(new_msgs);

            // Re-send the same user question
            loading.set(true);
            messages.write().push(ChatMessage {
                role: Role::Assistant,
                text: String::new(),
                sources: vec![],
                source_contexts: vec![],
                streaming: true,
            });

            spawn(async move {
                let result = api::stream_chat(&user_text, |event| match event {
                    StreamEvent::Token(token) => {
                        let mut msgs = messages.write();
                        if let Some(last) = msgs.last_mut() {
                            last.text.push_str(&token);
                        }
                    }
                    StreamEvent::Sources(sources) => {
                        let mut msgs = messages.write();
                        if let Some(last) = msgs.last_mut() {
                            last.sources = sources;
                        }
                    }
                    StreamEvent::Contexts(contexts) => {
                        let mut msgs = messages.write();
                        if let Some(last) = msgs.last_mut() {
                            last.source_contexts = contexts;
                        }
                    }
                    StreamEvent::Done => {
                        let mut msgs = messages.write();
                        if let Some(last) = msgs.last_mut() {
                            last.streaming = false;
                        }
                    }
                    StreamEvent::Error(err) => {
                        let mut msgs = messages.write();
                        if let Some(last) = msgs.last_mut() {
                            last.text = format!("Error: {err}");
                            last.streaming = false;
                        }
                    }
                })
                .await;

                if let Err(err) = result {
                    let mut msgs = messages.write();
                    if let Some(last) = msgs.last_mut() {
                        if last.streaming {
                            last.text = format!("Error: {err}");
                            last.streaming = false;
                        }
                    }
                }

                loading.set(false);

                // Persist updated session
                let current_id = active_session_id.read().clone();
                let current_messages = messages.read().clone();
                let current_docs: Vec<String> =
                    uploaded_docs.read().iter().map(|d| d.path.clone()).collect();
                let title = title_from_messages(&current_messages);
                {
                    let mut sess = sessions.write();
                    if let Some(s) = sess.iter_mut().find(|s| s.id == current_id) {
                        s.messages = current_messages.clone();
                        s.documents = current_docs.clone();
                        s.title = title.clone();
                    }
                }
                let _ = api::save_session(&current_id, &title, &current_messages, &current_docs)
                    .await;
            });
        }
    };

    let is_empty = messages.read().is_empty();
    let is_collapsed = *sidebar_collapsed.read();
    let doc_count = uploaded_docs.read().len();

    rsx! {
        div { class: "app-shell",
            // ── Collapsible Sidebar ──────────────────────────
            aside { class: if is_collapsed { "sidebar sidebar--collapsed" } else { "sidebar" },

                div { class: "sidebar-brand",
                    div { class: "sidebar-brand-left",
                        IconCpuBrand {}
                        if !is_collapsed {
                            span { "Nexa Support" }
                        }
                    }
                    button {
                        class: "sidebar-toggle",
                        title: if is_collapsed { "Expand sidebar" } else { "Collapse sidebar" },
                        onclick: move |_| sidebar_collapsed.set(!is_collapsed),
                        if is_collapsed {
                            IconChevronRight { size: 14 }
                        } else {
                            IconMenu { size: 16 }
                        }
                    }
                }

                nav { class: "sidebar-nav",
                    Link { to: Route::Chat {}, class: "nav-link active",
                        IconMessageSquare { size: 18 }
                        if !is_collapsed {
                            span { "Chat" }
                        }
                    }
                    Link { to: Route::Settings {}, class: "nav-link",
                        IconSettings { size: 18 }
                        if !is_collapsed {
                            span { "Settings" }
                        }
                    }
                }

                if !is_collapsed {
                    div { class: "sidebar-section sidebar-section--history",
                        ChatHistory {
                            sessions,
                            active_session_id,
                            on_select: on_select_session,
                            on_new: on_new_chat,
                            on_delete: on_delete_session,
                        }
                    }
                }

                div { class: "sidebar-footer",
                    if !is_collapsed {
                        "v2.0.0"
                    }
                }
            }

            // ── Main Area ────────────────────────────────────
            div { class: "main-area",
                // Topbar with model selector always visible
                div { class: "chat-header",
                    h1 { class: "chat-header-title", "Chat" }
                    div { class: "chat-header-model",
                        ModelSelector { selected_model }
                    }
                }

                // Chat content
                div { class: "chat-container",
                    if is_empty {
                        div { class: "chat-empty",
                            IconMessageSquare { size: 48 }
                            h2 { "Start a conversation" }
                            p { "Ask questions about your documents and get AI-powered answers." }
                            if uploaded_docs.read().is_empty() {
                                p { class: "chat-empty-hint",
                                    "Tip: Click the document icon next to the input to upload files."
                                }
                            }
                        }
                    } else {
                        div { class: "chat-messages",
                            {
                                let msgs = messages.read();
                                let last_bot_idx = msgs
                                    .iter()
                                    .enumerate()
                                    .rev()
                                    .find(|(_, m)| m.role == Role::Assistant)
                                    .map(|(idx, _)| idx);
                                rsx! {
                                    for (idx , msg) in msgs.iter().enumerate() {
                                        ChatBubble {
                                            msg: msg.clone(),
                                            on_regenerate,
                                            is_last_bot: Some(last_bot_idx == Some(idx)),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ── Documents flyout (above input) ──────────
                    if *show_docs_panel.read() && doc_count > 0 {
                        DocsFlyout {
                            uploaded_docs,
                            show_upload,
                            on_close: move |_| show_docs_panel.set(false),
                        }
                    }

                    // ── Input area with docs button ─────────────
                    ChatInput {
                        on_send,
                        disabled: *loading.read(),
                        show_upload: Some(show_upload),
                        doc_count: Some(doc_count),
                        on_toggle_docs: move |_| {
                            let current = *show_docs_panel.read();
                            show_docs_panel.set(!current);
                        },
                    }
                }

                UploadModal { show: show_upload, uploaded_docs }
            }

            // Delete session confirmation modal
            ConfirmModal {
                show: show_confirm_delete_session,
                title: "Delete Conversation".to_string(),
                message: "This conversation will be permanently deleted. This cannot be undone.".to_string(),
                confirm_text: Some("Delete".to_string()),
                danger: Some(true),
                on_confirm: do_delete_session,
            }
        }
    }
}

// ── Documents Flyout ─────────────────────────────────────

/// Flyout panel showing uploaded documents — opens above the input.
#[component]
fn DocsFlyout(
    uploaded_docs: Signal<Vec<UploadedDoc>>,
    show_upload: Signal<bool>,
    on_close: EventHandler<()>,
) -> Element {
    let docs = uploaded_docs.read();
    let total_size: u64 = docs.iter().map(|d| d.size).sum();
    let total_chunks: u64 = docs.iter().map(|d| d.chunks).sum();
    let doc_count = docs.len();

    rsx! {
        div { class: "docs-flyout",
            div { class: "docs-flyout-header",
                div { class: "docs-flyout-title",
                    IconFileText { size: 16 }
                    span { "Documents ({doc_count})" }
                }
                div { class: "docs-flyout-actions",
                    button {
                        class: "btn-add-doc",
                        onclick: move |_| show_upload.set(true),
                        IconUpload { size: 12 }
                        "Add"
                    }
                    button {
                        class: "docs-flyout-close",
                        onclick: move |_| on_close.call(()),
                        IconX { size: 14 }
                    }
                }
            }

            if total_size > 0 || total_chunks > 0 {
                div { class: "docs-flyout-stats",
                    {
                        let suffix = if doc_count != 1 { "s" } else { "" };
                        rsx! {
                            span { class: "doc-stat", "{doc_count} file{suffix}" }
                        }
                    }
                    if total_size > 0 {
                        span { class: "doc-stat", "{format_doc_size(total_size)}" }
                    }
                    if total_chunks > 0 {
                        span { class: "doc-stat", "{total_chunks} chunks" }
                    }
                }
            }

            div { class: "docs-flyout-list",
                for doc in docs.iter() {
                    DocFileItem { doc: doc.clone() }
                }
            }
        }
    }
}

/// Get an appropriate icon class for a file extension.
fn file_type_class(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => "doc-icon--pdf",
        "md" | "rst" => "doc-icon--md",
        "py" | "rs" | "js" | "ts" | "html" | "css" | "json" | "xml" | "yaml" | "yml" => {
            "doc-icon--code"
        }
        "csv" | "xlsx" | "xls" => "doc-icon--data",
        _ => "doc-icon--text",
    }
}

/// Single document file item in the flyout.
#[component]
fn DocFileItem(doc: UploadedDoc) -> Element {
    let icon_class = file_type_class(&doc.name);
    let ext = doc.name.rsplit('.').next().unwrap_or("").to_uppercase();

    rsx! {
        div { class: "doc-file-item",
            div { class: "doc-file-icon {icon_class}",
                if icon_class == "doc-icon--code" {
                    IconFileCode { size: 16 }
                } else {
                    IconFileText { size: 16 }
                }
            }
            div { class: "doc-file-info",
                div { class: "doc-file-name", "{doc.name}" }
                div { class: "doc-file-meta",
                    span { class: "doc-file-ext", "{ext}" }
                    if doc.size > 0 {
                        span { "{format_doc_size(doc.size)}" }
                    }
                    if doc.chunks > 0 {
                        span { "{doc.chunks} chunks" }
                    }
                }
                if !doc.tags.is_empty() {
                    div { class: "doc-file-tags",
                        for tag in &doc.tags {
                            span { class: "doc-tag", "{tag}" }
                        }
                    }
                }
            }
        }
    }
}

/// Small brand icon for the sidebar header.
#[component]
fn IconCpuBrand() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "22",
            height: "22",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect {
                x: "4",
                y: "4",
                width: "16",
                height: "16",
                rx: "2",
            }
            rect {
                x: "9",
                y: "9",
                width: "6",
                height: "6",
            }
            line {
                x1: "9",
                y1: "1",
                x2: "9",
                y2: "4",
            }
            line {
                x1: "15",
                y1: "1",
                x2: "15",
                y2: "4",
            }
            line {
                x1: "9",
                y1: "20",
                x2: "9",
                y2: "23",
            }
            line {
                x1: "15",
                y1: "20",
                x2: "15",
                y2: "23",
            }
            line {
                x1: "20",
                y1: "9",
                x2: "23",
                y2: "9",
            }
            line {
                x1: "20",
                y1: "14",
                x2: "23",
                y2: "14",
            }
            line {
                x1: "1",
                y1: "9",
                x2: "4",
                y2: "9",
            }
            line {
                x1: "1",
                y1: "14",
                x2: "4",
                y2: "14",
            }
        }
    }
}

/// Simple hamburger icon for sidebar toggle.
#[component]
fn IconMenu(size: u32) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line {
                x1: "4",
                y1: "6",
                x2: "20",
                y2: "6",
            }
            line {
                x1: "4",
                y1: "12",
                x2: "20",
                y2: "12",
            }
            line {
                x1: "4",
                y1: "18",
                x2: "20",
                y2: "18",
            }
        }
    }
}
