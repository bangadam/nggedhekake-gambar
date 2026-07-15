#![allow(non_snake_case)]

use crate::{
    components::{main_content::MainContent, sidebar::Sidebar},
    state::{AppState, invoke_command},
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, prelude::*};

static CSS: Asset = asset!("/assets/styles.css");

#[derive(Serialize)]
struct EmptyArgs {}

#[derive(Deserialize)]
struct EventEnvelope<T> {
    payload: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    percent: f32,
    message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DonePayload {
    output_path: String,
}

#[derive(Deserialize)]
struct ErrorPayload {
    message: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"], js_name = listen)]
    async fn listen(event: &str, handler: &js_sys::Function) -> Result<JsValue, JsValue>;
}

fn install_event_listeners(state: AppState) {
    spawn(async move {
        let mut progress = state.progress;
        let mut progress_message = state.progress_message;
        let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
            if let Ok(event) =
                serde_wasm_bindgen::from_value::<EventEnvelope<ProgressPayload>>(event)
            {
                progress.set(event.payload.percent);
                progress_message.set(event.payload.message);
            }
        });

        if listen(
            "upscale-progress",
            callback.as_ref().unchecked_ref::<js_sys::Function>(),
        )
        .await
        .is_ok()
        {
            callback.forget();
        }
    });

    spawn(async move {
        let mut progress = state.progress;
        let mut progress_message = state.progress_message;
        let mut upscaled_path = state.upscaled_image_path;
        let mut processing = state.is_processing;
        let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
            if let Ok(event) = serde_wasm_bindgen::from_value::<EventEnvelope<DonePayload>>(event) {
                progress.set(100.0);
                progress_message.set("Enhancement complete".into());
                upscaled_path.set(Some(event.payload.output_path));
                processing.set(false);
            }
        });

        if listen(
            "upscale-done",
            callback.as_ref().unchecked_ref::<js_sys::Function>(),
        )
        .await
        .is_ok()
        {
            callback.forget();
        }
    });

    spawn(async move {
        let mut error = state.error;
        let mut processing = state.is_processing;
        let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
            if let Ok(event) = serde_wasm_bindgen::from_value::<EventEnvelope<ErrorPayload>>(event)
            {
                processing.set(false);
                error.set(Some(event.payload.message));
            }
        });

        if listen(
            "upscale-error",
            callback.as_ref().unchecked_ref::<js_sys::Function>(),
        )
        .await
        .is_ok()
        {
            callback.forget();
        }
    });
}

pub fn App() -> Element {
    let state = AppState::new();
    use_context_provider(|| state);

    use_effect(move || {
        install_event_listeners(state);

        spawn(async move {
            let mut models = state.models;
            let mut selected_model = state.selected_model;
            let mut error = state.error;
            match invoke_command("get_models", &EmptyArgs {}).await {
                Ok(value) => match serde_wasm_bindgen::from_value::<Vec<String>>(value) {
                    Ok(available_models) if !available_models.is_empty() => {
                        if !available_models.contains(&selected_model()) {
                            selected_model.set(available_models[0].clone());
                        }
                        models.set(available_models);
                    }
                    Ok(_) => error.set(Some("No bundled upscaling models were found".into())),
                    Err(parse_error) => error.set(Some(parse_error.to_string())),
                },
                Err(message) => error.set(Some(message)),
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "app-shell",
            Sidebar {}
            MainContent {}
        }
    }
}
