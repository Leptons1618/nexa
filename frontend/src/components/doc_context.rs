//! Document context panel — shows documents uploaded in the current chat session.

use dioxus::prelude::*;

use crate::components::icons::{IconFileText, IconTag, IconUpload};
use crate::models::UploadedDoc;

#[component]
pub fn DocContext(
    documents: Signal<Vec<UploadedDoc>>,
    on_add_click: EventHandler<()>,
) -> Element {
    let docs = documents.read();

    rsx! {
        div { class: "doc-context",
            div { class: "doc-context-header",
                div { class: "doc-context-title",
                    IconFileText { size: 14 }
                    span { "Documents ({docs.len()})" }
                }
                button {
                    class: "btn-add-doc",
                    title: "Add documents",
                    onclick: move |_| on_add_click.call(()),
                    IconUpload { size: 14 }
                    span { "Add" }
                }
            }

            if docs.is_empty() {
                div { class: "doc-context-empty",
                    p { "No documents added to this session yet." }
                    button {
                        class: "btn btn--ghost btn--sm",
                        onclick: move |_| on_add_click.call(()),
                        IconUpload { size: 14 }
                        "Add Documents"
                    }
                }
            } else {
                div { class: "doc-context-list",
                    for doc in docs.iter() {
                        div { class: "doc-item",
                            div { class: "doc-item-icon",
                                IconFileText { size: 14 }
                            }
                            div { class: "doc-item-info",
                                div { class: "doc-item-path", "{doc.path}" }
                                div { class: "doc-item-meta",
                                    span { "{doc.chunks} chunks" }
                                    if !doc.tags.is_empty() {
                                        for tag in doc.tags.iter() {
                                            span { class: "doc-tag",
                                                IconTag { size: 10 }
                                                "{tag}"
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
    }
}
