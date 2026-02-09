//! Chat history sidebar panel — shows past chat sessions.

use dioxus::prelude::*;

use crate::components::icons::{IconMessageSquare, IconTrash};
use crate::models::ChatSession;

#[component]
pub fn ChatHistory(
    sessions: Signal<Vec<ChatSession>>,
    active_session_id: Signal<String>,
    on_select: EventHandler<String>,
    on_new: EventHandler<()>,
    on_delete: EventHandler<String>,
) -> Element {
    let sessions_read = sessions.read();

    rsx! {
        div { class: "chat-history",
            div { class: "chat-history-header",
                h4 { "History" }
                button {
                    class: "btn-new-chat",
                    title: "New Chat",
                    onclick: move |_| on_new.call(()),
                    IconPlus { size: 16 }
                }
            }

            div { class: "chat-history-list",
                if sessions_read.is_empty() {
                    div { class: "chat-history-empty",
                        p { "No conversations yet" }
                    }
                } else {
                    for session in sessions_read.iter().rev() {
                        {
                            let sid = session.id.clone();
                            let sid_for_delete = session.id.clone();
                            let is_active = sid == *active_session_id.read();
                            let title = session.title.clone();
                            let msg_count = session.messages.len();
                            let doc_count = session.documents.len();
                            rsx! {
                                div { class: if is_active { "history-item history-item--active" } else { "history-item" },
                                    div {
                                        class: "history-item-content",
                                        onclick: move |_| on_select.call(sid.clone()),
                                        div { class: "history-item-icon",
                                            IconMessageSquare { size: 14 }
                                        }
                                        div { class: "history-item-info",
                                            div { class: "history-item-title", "{title}" }
                                            div { class: "history-item-meta",
                                                span { "{msg_count} messages" }
                                                if doc_count > 0 {
                                                    span { " · {doc_count} docs" }
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        class: "history-item-delete",
                                        title: "Delete conversation",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            on_delete.call(sid_for_delete.clone());
                                        },
                                        IconTrash { size: 12 }
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

/// Plus icon for creating new chats.
#[component]
fn IconPlus(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line {
                x1: "12",
                y1: "5",
                x2: "12",
                y2: "19",
            }
            line {
                x1: "5",
                y1: "12",
                x2: "19",
                y2: "12",
            }
        }
    }
}
