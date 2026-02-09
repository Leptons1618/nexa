//! Chat message bubble component with rich content rendering and source hover.

use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::icons::{IconBot, IconCopy, IconRefreshCw, IconTag, IconUser};
use crate::components::rich_content::RichContent;
use crate::models::{ChatMessage, Role};

#[component]
pub fn ChatBubble(
    msg: ChatMessage,
    on_regenerate: Option<EventHandler<()>>,
    is_last_bot: Option<bool>,
) -> Element {
    let is_user = msg.role == Role::User;
    let bubble_class = if is_user {
        "bubble bubble--user"
    } else {
        "bubble bubble--bot"
    };
    let avatar_class = if is_user {
        "bubble-avatar bubble-avatar--user"
    } else {
        "bubble-avatar bubble-avatar--bot"
    };
    let streaming = msg.streaming;
    let text_for_copy = msg.text.clone();

    rsx! {
        div { class: "{bubble_class}",
            div { class: "{avatar_class}",
                if is_user {
                    IconUser { size: 18 }
                } else {
                    IconBot { size: 18 }
                }
            }
            div { class: "bubble-content",
                div { class: "bubble-body",
                    if is_user {
                        // User messages rendered as plain text
                        "{msg.text}"
                    } else {
                        // Bot messages rendered with full rich content
                        RichContent { text: msg.text.clone(), streaming }
                    }
                    // Streaming cursor
                    if streaming {
                        span { class: "streaming-cursor" }
                    }
                }

                // Bubble actions (copy, regenerate, etc.) — only for bot messages that are done
                if !is_user && !streaming && !msg.text.is_empty() {
                    div { class: "bubble-actions",
                        button {
                            class: "bubble-action-btn",
                            title: "Copy response",
                            onclick: {
                                let text = text_for_copy.clone();
                                move |_| {
                                    let t = text.clone();
                                    spawn(async move {
                                        copy_to_clipboard(&t).await;
                                    });
                                }
                            },
                            IconCopy { size: 14 }
                        }
                        if is_last_bot.unwrap_or(false) {
                            if let Some(handler) = &on_regenerate {
                                button {
                                    class: "bubble-action-btn",
                                    title: "Regenerate response",
                                    onclick: {
                                        let handler = handler.clone();
                                        move |_| handler.call(())
                                    },
                                    IconRefreshCw { size: 14 }
                                }
                            }
                        }
                    }
                }

                // Source references with hover previews
                if !msg.sources.is_empty() {
                    div { class: "bubble-sources",
                        for (idx , source) in msg.sources.iter().enumerate() {
                            SourceTag {
                                source: source.clone(),
                                context_text: msg.source_contexts.get(idx)
                                                                                                    .map(|c| c.text.clone())
                                                                                                    .unwrap_or_default(),
                                score: msg.source_contexts.get(idx)
                                                                                                    .map(|c| c.score)
                                                                                                    .unwrap_or(0.0),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A source tag with hover tooltip showing the retrieved chunk text.
#[component]
fn SourceTag(source: String, context_text: String, score: f64) -> Element {
    let has_context = !context_text.is_empty();
    let score_pct = format!("{:.0}%", score * 100.0);

    rsx! {
        div { class: "source-tag-wrapper",
            span { class: "source-tag",
                IconTag { size: 12 }
                "{source}"
                if has_context {
                    span { class: "source-score", "{score_pct}" }
                }
            }
            // Hover tooltip with the actual retrieved text
            if has_context {
                div { class: "source-tooltip",
                    div { class: "source-tooltip-header",
                        span { class: "source-tooltip-file", "{source}" }
                        span { class: "source-tooltip-score", "Relevance: {score_pct}" }
                    }
                    div { class: "source-tooltip-body", "{context_text}" }
                }
            }
        }
    }
}

/// Copy text to clipboard via the Clipboard API.
async fn copy_to_clipboard(text: &str) {
    use wasm_bindgen::JsValue;
    if let Some(window) = web_sys::window() {
        let nav: JsValue = js_sys::Reflect::get(&window, &JsValue::from_str("navigator"))
            .unwrap_or(JsValue::UNDEFINED);
        if !nav.is_undefined() {
            let clipboard: JsValue = js_sys::Reflect::get(&nav, &JsValue::from_str("clipboard"))
                .unwrap_or(JsValue::UNDEFINED);
            if !clipboard.is_undefined() {
                if let Ok(write_fn) = js_sys::Reflect::get(&clipboard, &JsValue::from_str("writeText")) {
                    if let Ok(func) = write_fn.dyn_into::<js_sys::Function>() {
                        let _: Result<JsValue, JsValue> = func.call1(&clipboard, &JsValue::from_str(text));
                    }
                }
            }
        }
    }
}
