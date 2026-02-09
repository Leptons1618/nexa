//! Sidebar navigation component.

use dioxus::prelude::*;

use crate::components::icons::{IconFileText, IconMessageSquare, IconSettings};
use crate::Route;

#[component]
pub fn Sidebar() -> Element {
    let route: Route = use_route();

    let chat_active = matches!(route, Route::Chat {});
    let docs_active = matches!(route, Route::Documents {});
    let settings_active = matches!(route, Route::Settings {});

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-brand",
                IconCpuBrand {}
                span { "Nexa Support" }
            }

            nav { class: "sidebar-nav",
                Link {
                    to: Route::Chat {},
                    class: if chat_active { "nav-link active" } else { "nav-link" },
                    IconMessageSquare { size: 18 }
                    span { "Chat" }
                }
                Link {
                    to: Route::Documents {},
                    class: if docs_active { "nav-link active" } else { "nav-link" },
                    IconFileText { size: 18 }
                    span { "Documents" }
                }
                Link {
                    to: Route::Settings {},
                    class: if settings_active { "nav-link active" } else { "nav-link" },
                    IconSettings { size: 18 }
                    span { "Settings" }
                }
            }

            div { class: "sidebar-footer", "v2.0.0" }
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
