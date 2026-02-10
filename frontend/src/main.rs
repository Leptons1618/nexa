//! Nexa Support — Dioxus web frontend

mod api;
mod components;
mod models;
mod pages;
mod state;

use dioxus::prelude::*;

use components::sidebar::Sidebar;
use pages::chat::Chat;
use pages::documents::Documents;
use pages::settings::Settings;
use state::AppState;

/// Bundled stylesheet — processed by the manganis asset pipeline.
const MAIN_CSS: Asset = asset!("/assets/main.css");

/// Application routes
#[derive(Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Chat {},
    #[layout(AppLayout)]
        #[route("/documents")]
        Documents {},
        #[route("/settings")]
        Settings {},
}

/// Root layout — sidebar + main content area
#[component]
fn AppLayout() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "app-shell",
            Sidebar {}
            div { class: "main-area", Outlet::<Route> {} }
        }
    }
}

fn main() {
    dioxus::launch(|| {
        // ── Global state: fetched once, shared across all pages ──
        let mut models = use_signal(Vec::<models::OllamaModelEntry>::new);
        let mut selected_model = use_signal(|| String::new());
        let mut loaded = use_signal(|| false);

        use_context_provider(|| AppState {
            models,
            selected_model,
            loaded,
        });

        // Load initial data once at startup
        use_resource(move || async move {
            if let Ok(status) = api::fetch_ollama_status().await {
                selected_model.set(status.model);
            }
            if let Ok(m) = api::fetch_ollama_models().await {
                models.set(m.models);
            }
            loaded.set(true);
        });

        rsx! {
            document::Link { rel: "stylesheet", href: MAIN_CSS }
            // KaTeX CSS for LaTeX rendering
            document::Link {
                rel: "stylesheet",
                href: "https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css",
            }
            // KaTeX JS
            document::Script { src: "https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js" }
            // highlight.js for code syntax highlighting
            document::Link {
                rel: "stylesheet",
                href: "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.9.0/build/styles/github.min.css",
            }
            document::Script { src: "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.9.0/build/highlight.min.js" }
            Router::<Route> {}
        }
    });
}
