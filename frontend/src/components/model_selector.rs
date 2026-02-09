//! Model selector dropdown — lets user pick the active Ollama model.

use dioxus::prelude::*;

use crate::api;
use crate::components::icons::IconCpu;

#[component]
pub fn ModelSelector(
    selected_model: Signal<String>,
) -> Element {
    let models_res = use_resource(|| async { api::fetch_ollama_models().await });
    let mut switching = use_signal(|| false);

    let on_change = move |evt: Event<FormData>| {
        let new_model = evt.value();
        tracing::info!("🔍 DEBUG: Model selector changed to: {}", new_model);
        if new_model.is_empty() || new_model == *selected_model.read() {
            return;
        }
        let model_clone = new_model.clone();
        switching.set(true);

        spawn(async move {
            match api::switch_model(&model_clone).await {
                Ok(resp) => {
                    tracing::info!("✅ Model switched successfully to: {}", resp.current_model);
                    selected_model.set(resp.current_model);
                }
                Err(err) => {
                    tracing::error!("❌ Failed to switch model: {}", err);
                    // Revert silently on failure
                }
            }
            switching.set(false);
        });
    };

    let current_model = selected_model.read().clone();
    tracing::info!("🔍 DEBUG: Current selected model: {}", current_model);

    rsx! {
        div { class: "model-selector",
            IconCpu { size: 14 }
            match &*models_res.read() {
                Some(Ok(m)) => {
                    tracing::info!("🔍 DEBUG: Available models count: {}", m.models.len());
                    rsx! {
                        select {
                            class: "model-select",
                            disabled: *switching.read(),
                            value: "{current_model}",
                            onchange: on_change,
                            for model in m.models.iter() {
                                option { value: "{model.name}", "{model.name}" }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    tracing::error!("🔍 DEBUG: Error loading models: {:?}", e);
                    rsx! {
                        span { class: "model-select-fallback", "{selected_model}" }
                    }
                }
                None => rsx! {
                    span { class: "model-select-fallback", "Loading..." }
                },
            }
            if *switching.read() {
                span { class: "model-switching", "switching..." }
            }
        }
    }
}
