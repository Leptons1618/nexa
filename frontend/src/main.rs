//! Nexa Support — Dioxus web frontend

mod api;
mod components;
mod models;
mod pages;

use dioxus::prelude::*;

use components::sidebar::Sidebar;
use pages::chat::Chat;
use pages::documents::Documents;
use pages::settings::Settings;

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
