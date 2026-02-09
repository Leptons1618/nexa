//! Status badge showing backend health.

use dioxus::prelude::*;

#[component]
pub fn StatusBadge(status: String, connected: bool) -> Element {
    let (badge_class, dot_class, label) = if status == "loading" {
        ("status-badge status-badge--warn", "status-dot status-dot--warn", "Connecting...")
    } else if connected {
        ("status-badge status-badge--ok", "status-dot status-dot--ok", "Connected")
    } else {
        ("status-badge status-badge--error", "status-dot status-dot--error", "Disconnected")
    };

    rsx! {
        span { class: "{badge_class}",
            span { class: "{dot_class}" }
            "{label}"
        }
    }
}
