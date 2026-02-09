//! Chat input bar with send button.

use dioxus::prelude::*;

use crate::components::icons::IconSend;

#[component]
pub fn ChatInput(
    on_send: EventHandler<String>,
    disabled: bool,
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

    rsx! {
        div { class: "chat-input-area",
            div { class: "chat-input-wrap",
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
