//! Sidebar navigation component with collapse support.

use dioxus::prelude::*;

use crate::components::icons::{IconChevronLeft, IconChevronRight, IconMessageSquare, IconSettings};
use crate::components::model_selector::ModelSelector;
use crate::components::chat_history::ChatHistory;
use crate::models::ChatSession;
use crate::Route;

#[component]
pub fn Sidebar(
    selected_model: Option<Signal<String>>,
    sessions: Option<Signal<Vec<ChatSession>>>,
    active_session_id: Option<Signal<String>>,
    on_select_session: Option<EventHandler<String>>,
    on_new_chat: Option<EventHandler<()>>,
    on_delete_session: Option<EventHandler<String>>,
) -> Element {
    let route: Route = use_route();
    let mut collapsed = use_signal(|| false);
    let is_collapsed = *collapsed.read();

    let chat_active = matches!(route, Route::Chat {});
    let settings_active = matches!(route, Route::Settings {});

    rsx! {
        aside { class: if is_collapsed { "sidebar sidebar--collapsed" } else { "sidebar" },

            div { class: "sidebar-brand",
                IconCpuBrand {}
                if !is_collapsed {
                    span { "Nexa Support" }
                }
            }

            button {
                class: "sidebar-toggle",
                title: if is_collapsed { "Expand sidebar" } else { "Collapse sidebar" },
                onclick: move |_| collapsed.set(!is_collapsed),
                if is_collapsed {
                    IconChevronRight { size: 16 }
                } else {
                    IconChevronLeft { size: 16 }
                }
            }

            nav { class: "sidebar-nav",
                Link {
                    to: Route::Chat {},
                    class: if chat_active { "nav-link active" } else { "nav-link" },
                    IconMessageSquare { size: 18 }
                    if !is_collapsed {
                        span { "Chat" }
                    }
                }
                Link {
                    to: Route::Settings {},
                    class: if settings_active { "nav-link active" } else { "nav-link" },
                    IconSettings { size: 18 }
                    if !is_collapsed {
                        span { "Settings" }
                    }
                }
            }

            // Model selector (only show when expanded and on chat page)
            if !is_collapsed {
                if let Some(model) = selected_model {
                    div { class: "sidebar-section",
                        div { class: "sidebar-section-title", "Model" }
                        ModelSelector { selected_model: model }
                    }
                }

                // Chat history (only show on chat page)
                if let (Some(sess), Some(active_id), Some(on_sel), Some(on_new), Some(on_del)) = (
                    sessions,
                    active_session_id,
                    on_select_session,
                    on_new_chat,
                    on_delete_session,
                )
                {
                    div { class: "sidebar-section sidebar-section--history",
                        ChatHistory {
                            sessions: sess,
                            active_session_id: active_id,
                            on_select: on_sel,
                            on_new,
                            on_delete: on_del,
                        }
                    }
                }
            }

            div { class: "sidebar-footer",
                if !is_collapsed {
                    "v2.0.0"
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
