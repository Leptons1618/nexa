//! Model selector dropdown — lets user pick the active Ollama model.
//!
//! Reads the model list from global AppState (no per-component fetch).

use dioxus::prelude::*;

use crate::api;
use crate::components::icons::IconCpu;
use crate::state::AppState;

#[component]
pub fn ModelSelector(
    selected_model: Signal<String>,
) -> Element {
    let state = use_context::<AppState>();
    let mut switching = use_signal(|| false);

    let on_change = move |evt: Event<FormData>| {
        let new_model = evt.value();
        if new_model.is_empty() || new_model == *selected_model.read() {
            return;
        }
        let model_clone = new_model.clone();
        switching.set(true);

        spawn(async move {
            match api::switch_model(&model_clone).await {
                Ok(resp) => {
                    selected_model.set(resp.current_model);
                }
                Err(_err) => {}
            }
            switching.set(false);
        });
    };

    let current_model = selected_model.read().clone();
    let models = state.models.read().clone();
    let is_loaded = *state.loaded.read();

    rsx! {
        div { class: "model-selector",
            IconCpu { size: 14 }
            if is_loaded && !models.is_empty() {
                select {
                    class: "model-select",
                    disabled: *switching.read(),
                    value: "{current_model}",
                    onchange: on_change,
                    for model in models.iter() {
                        option { value: "{model.name}", "{model.name}" }
                    }
                }
            } else if is_loaded {
                span { class: "model-select-fallback", "{current_model}" }
            } else {
                span { class: "model-select-fallback", "Loading…" }
            }
            if *switching.read() {
                span { class: "model-switching",
                    span { class: "spinner" }
                    "switching…"
                }
            }
        }
    }
}
