#![allow(non_snake_case)]

use crate::{
    components::{main_content::MainContent, sidebar::Sidebar},
    state::{AppState, EmptyArgs, invoke_decode, startup_warning},
};
use dioxus::prelude::*;
use serde::Deserialize;
use upscale_contract::{RunEvent, StartupStatus, UpscaleEngine};
use wasm_bindgen::{JsCast, prelude::*};

static CSS: Asset = asset!("/assets/styles.css");

#[derive(Deserialize)]
struct EventEnvelope<T> {
    payload: T,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"], js_name = listen)]
    async fn listen(event: &str, handler: &js_sys::Function) -> Result<JsValue, JsValue>;
}

async fn install_event_listener(mut state: AppState) -> Result<(), String> {
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        if let Ok(event) = serde_wasm_bindgen::from_value::<EventEnvelope<RunEvent>>(event) {
            state.apply_event(event.payload);
        }
    });
    listen(
        "upscale-run-event",
        callback.as_ref().unchecked_ref::<js_sys::Function>(),
    )
    .await
    .map_err(|error| error.as_string().unwrap_or_else(|| format!("{error:?}")))?;
    callback.forget();
    Ok(())
}

async fn initialize(mut state: AppState) {
    match invoke_decode::<_, Vec<String>>("get_models", &EmptyArgs {}).await {
        Ok(models) if !models.is_empty() => {
            if !models.contains(&state.selected_model.read()) {
                state.selected_model.set(models[0].clone());
            }
            state.models.set(models);
            state.models_initialized.set(true);
        }
        Ok(_) => state
            .ui_error
            .set(Some("No bundled upscaling models were found".into())),
        Err(message) => state.ui_error.set(Some(message)),
    }

    match invoke_decode::<_, UpscaleEngine>("get_engine_preference", &EmptyArgs {}).await {
        Ok(engine) => {
            state.primary_engine.set(Some(engine));
            state.preference_initialized.set(true);
        }
        Err(message) => state.ui_error.set(Some(message)),
    }

    match invoke_decode::<_, StartupStatus>("get_startup_status", &EmptyArgs {}).await {
        Ok(status) => state.startup_warning.set(startup_warning(status)),
        Err(message) => state.ui_error.set(Some(message)),
    }

    match invoke_decode::<_, Option<RunEvent>>("get_upscale_status", &EmptyArgs {}).await {
        Ok(Some(event)) => state.apply_event(event),
        Ok(None) => {}
        Err(message) => state.ui_error.set(Some(message)),
    }
}

pub fn App() -> Element {
    let state = AppState::new();
    use_context_provider(|| state);

    use_effect(move || {
        spawn(async move {
            let mut state = state;
            if let Err(message) = install_event_listener(state).await {
                state.ui_error.set(Some(message));
                return;
            }
            initialize(state).await;
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
