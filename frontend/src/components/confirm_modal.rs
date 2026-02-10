//! Reusable confirmation dialog modal component.

use dioxus::prelude::*;

use crate::components::icons::IconAlertCircle;

/// A confirmation dialog that overlays the page.
///
/// Renders nothing when `show` is false; overlays the page when true.
/// Calls `on_confirm` when the user clicks the confirm button, then auto-hides.
#[component]
pub fn ConfirmModal(
    show: Signal<bool>,
    title: String,
    message: String,
    confirm_text: Option<String>,
    danger: Option<bool>,
    on_confirm: EventHandler<()>,
) -> Element {
    if !*show.read() {
        return rsx! {};
    }

    let btn_label = confirm_text.unwrap_or_else(|| "Confirm".to_string());
    let is_danger = danger.unwrap_or(false);
    let btn_class = if is_danger {
        "btn btn--danger btn--sm"
    } else {
        "btn btn--primary btn--sm"
    };

    rsx! {
        div {
            class: "modal-backdrop confirm-backdrop",
            onclick: move |_| show.set(false),
            div {
                class: "modal modal--confirm",
                onclick: move |e: MouseEvent| e.stop_propagation(),
                if is_danger {
                    div { class: "confirm-icon-row",
                        div { class: "confirm-icon confirm-icon--danger",
                            IconAlertCircle { size: 28 }
                        }
                    }
                }
                h3 { class: "confirm-title", "{title}" }
                p { class: "confirm-message", "{message}" }
                div { class: "confirm-actions",
                    button {
                        class: "btn btn--ghost btn--sm",
                        onclick: move |_| show.set(false),
                        "Cancel"
                    }
                    button {
                        class: "{btn_class}",
                        onclick: move |_| {
                            on_confirm.call(());
                            show.set(false);
                        },
                        "{btn_label}"
                    }
                }
            }
        }
    }
}
