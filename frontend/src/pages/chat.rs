//! Chat page — main conversation interface.

use dioxus::prelude::*;

use crate::api;
use crate::components::chat_bubble::ChatBubble;
use crate::components::chat_input::ChatInput;
use crate::components::header::Header;
use crate::components::icons::IconMessageSquare;
use crate::models::{ChatMessage, Role};

#[component]
pub fn Chat() -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut loading = use_signal(|| false);

    let on_send = move |text: String| {
        // Push user message
        messages.write().push(ChatMessage {
            role: Role::User,
            text: text.clone(),
            sources: vec![],
        });
        loading.set(true);

        spawn(async move {
            match api::send_chat(&text).await {
                Ok(resp) => {
                    messages.write().push(ChatMessage {
                        role: Role::Assistant,
                        text: resp.answer,
                        sources: resp.sources,
                    });
                }
                Err(err) => {
                    messages.write().push(ChatMessage {
                        role: Role::Assistant,
                        text: format!("Error: {err}"),
                        sources: vec![],
                    });
                }
            }
            loading.set(false);
        });
    };

    let is_empty = messages.read().is_empty();

    rsx! {
        Header { title: "Chat".to_string() }
        div { class: "page-content",
            div { class: "chat-container",
                if is_empty {
                    div { class: "chat-empty",
                        IconMessageSquare { size: 48 }
                        h2 { "Start a conversation" }
                        p { "Ask questions about Nexa and get answers from the documentation." }
                    }
                } else {
                    div { class: "chat-messages",
                        for msg in messages.read().iter() {
                            ChatBubble { msg: msg.clone() }
                        }
                        if *loading.read() {
                            div { class: "bubble bubble--bot",
                                div { class: "bubble-avatar bubble-avatar--bot",
                                    crate::components::icons::IconBot { size: 18 }
                                }
                                div { class: "bubble-body",
                                    div { class: "typing-indicator",
                                        span {}
                                        span {}
                                        span {}
                                    }
                                }
                            }
                        }
                    }
                }
                ChatInput { on_send, disabled: *loading.read() }
            }
        }
    }
}
