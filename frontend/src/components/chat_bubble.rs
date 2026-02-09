//! Chat message bubble component.

use dioxus::prelude::*;

use crate::components::icons::{IconBot, IconTag, IconUser};
use crate::models::{ChatMessage, Role};

#[component]
pub fn ChatBubble(msg: ChatMessage) -> Element {
    let is_user = msg.role == Role::User;
    let bubble_class = if is_user { "bubble bubble--user" } else { "bubble bubble--bot" };
    let avatar_class = if is_user {
        "bubble-avatar bubble-avatar--user"
    } else {
        "bubble-avatar bubble-avatar--bot"
    };

    rsx! {
        div { class: "{bubble_class}",
            div { class: "{avatar_class}",
                if is_user {
                    IconUser { size: 18 }
                } else {
                    IconBot { size: 18 }
                }
            }
            div {
                div { class: "bubble-body", "{msg.text}" }
                if !msg.sources.is_empty() {
                    div { class: "bubble-sources",
                        for source in msg.sources.iter() {
                            span { class: "source-tag",
                                IconTag { size: 12 }
                                "{source}"
                            }
                        }
                    }
                }
            }
        }
    }
}
