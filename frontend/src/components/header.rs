//! Header bar with page title and status badge.

use dioxus::prelude::*;

use crate::api;
use crate::components::status_badge::StatusBadge;

#[component]
pub fn Header(title: String) -> Element {
    let health = use_resource(|| async { api::fetch_health().await });

    let (status, connected) = match &*health.read() {
        Some(Ok(h)) => (h.status.clone(), h.llm_connected),
        Some(Err(_)) => ("error".to_string(), false),
        None => ("loading".to_string(), false),
    };

    rsx! {
        header { class: "header",
            h1 { class: "header-title", "{title}" }
            StatusBadge { status, connected }
        }
    }
}
