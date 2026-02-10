//! Chat input bar with send button, documents indicator, and upload button.

use dioxus::prelude::*;

use crate::components::icons::{IconFileText, IconSend};

#[component]
pub fn ChatInput(
    on_send: EventHandler<String>,
    disabled: bool,
    show_upload: Option<Signal<bool>>,
    /// Number of uploaded docs in current session (shows indicator badge).
    doc_count: Option<usize>,
    /// Callback when docs button is clicked (toggle docs flyout).
    on_toggle_docs: Option<EventHandler<()>>,
) -> Element {
    let mut input_text = use_signal(|| String::new());

    let submit = move |_| {
        let text = input_text.read().trim().to_string();
        if !text.is_empty() && !disabled {
            on_send.call(text);
            input_text.set(String::new());
        }
    };

    let on_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Enter && !evt.modifiers().shift() {
            let text = input_text.read().trim().to_string();
            if !text.is_empty() && !disabled {
                on_send.call(text);
                input_text.set(String::new());
            }
        }
    };

    let count = doc_count.unwrap_or(0);

    rsx! {
        div { class: "chat-input-area",
            div { class: "chat-input-wrap",
                // Documents indicator button
                if let Some(on_docs) = on_toggle_docs {
                    button {
                        class: if count > 0 { "btn-docs btn-docs--has-files" } else { "btn-docs" },
                        title: if count > 0 { "View uploaded documents" } else { "Upload documents" },
                        onclick: move |_| {
                            if count > 0 {
                                on_docs.call(());
                            } else if let Some(mut upload) = show_upload {
                                upload.set(true);
                            }
                        },
                        IconFileText { size: 16 }
                        if count > 0 {
                            span { class: "btn-docs-badge", "{count}" }
                        }
                    }
                }

                // // Upload button
                // if let Some(mut upload) = show_upload {
                //     button {
                //         class: "btn-upload",
                //         onclick: move |_| upload.set(true),
                //         title: "Upload Documents",
                //         IconUpload { size: 16 }
                //     }
                // }
                input {
                    class: "chat-input",
                    r#type: "text",
                    placeholder: "Type your question...",
                    value: "{input_text}",
                    disabled,
                    oninput: move |evt| input_text.set(evt.value()),
                    onkeydown: on_keydown,
                }
                button {
                    class: "btn-send",
                    disabled: disabled || input_text.read().trim().is_empty(),
                    onclick: submit,
                    IconSend { size: 18 }
                }
            }
        }
    }
}
