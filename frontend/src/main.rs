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

/// Application routes
#[derive(Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Chat {},
        #[route("/documents")]
        Documents {},
        #[route("/settings")]
        Settings {},
}

/// Root layout — sidebar + main content area
#[component]
fn AppLayout() -> Element {
    rsx! {
        div { class: "app-shell",
            Sidebar {}
            div { class: "main-area", Outlet::<Route> {} }
        }
    }
}

fn main() {
    dioxus::launch(|| {
        rsx! {
            Router::<Route> {}
        }
    });
}
