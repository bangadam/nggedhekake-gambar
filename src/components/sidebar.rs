use crate::state::{AppState, invoke_command};
use dioxus::prelude::*;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct EmptyArgs {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpscaleArgs {
    image_path: String,
    output_folder: String,
    model: String,
    scale: u32,
    format: String,
    gpu_id: Option<i32>,
}

fn display_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_owned()
}

#[component]
pub fn Sidebar() -> Element {
    let mut state = use_context::<AppState>();
    let image_label = state
        .image_path
        .read()
        .as_deref()
        .map(display_name)
        .unwrap_or_else(|| "Choose an image".into());
    let folder_label = state
        .output_folder
        .read()
        .as_deref()
        .map(display_name)
        .unwrap_or_else(|| "Choose destination".into());
    let selected_model = (state.selected_model)();
    let selected_scale = (state.scale)();
    let selected_format = (state.format)();
    let is_processing = (state.is_processing)();
    let can_upscale = state.image_path.read().is_some()
        && state.output_folder.read().is_some()
        && !state.models.read().is_empty()
        && !is_processing;

    let select_image = move |_| async move {
        let mut image_path = state.image_path;
        let mut upscaled_path = state.upscaled_image_path;
        let mut error = state.error;
        match invoke_command("select_image", &EmptyArgs {}).await {
            Ok(value) => {
                if let Some(path) = value.as_string() {
                    image_path.set(Some(path));
                    upscaled_path.set(None);
                    error.set(None);
                }
            }
            Err(message) => error.set(Some(message)),
        }
    };

    let select_folder = move |_| async move {
        let mut output_folder = state.output_folder;
        let mut error = state.error;
        match invoke_command("select_folder", &EmptyArgs {}).await {
            Ok(value) => {
                if let Some(path) = value.as_string() {
                    output_folder.set(Some(path));
                    error.set(None);
                }
            }
            Err(message) => error.set(Some(message)),
        }
    };

    let start_upscale = move |_| async move {
        let Some(image_path) = state.image_path.read().clone() else {
            return;
        };
        let Some(output_folder) = state.output_folder.read().clone() else {
            return;
        };

        let args = UpscaleArgs {
            image_path,
            output_folder,
            model: (state.selected_model)(),
            scale: (state.scale)(),
            format: (state.format)(),
            gpu_id: None,
        };

        let mut processing = state.is_processing;
        let mut progress = state.progress;
        let mut progress_message = state.progress_message;
        let mut upscaled_path = state.upscaled_image_path;
        let mut error = state.error;
        processing.set(true);
        progress.set(0.0);
        progress_message.set("Preparing model…".into());
        upscaled_path.set(None);
        error.set(None);

        if let Err(message) = invoke_command("upscale_image", &args).await {
            processing.set(false);
            error.set(Some(message));
        }
    };

    let stop_upscale = move |_| async move {
        let mut error = state.error;
        if let Err(message) = invoke_command("stop_upscale", &EmptyArgs {}).await {
            error.set(Some(message));
        }
    };

    rsx! {
        aside { class: "sidebar",
            header { class: "brand",
                div { class: "brand-mark", "NG" }
                div {
                    p { class: "eyebrow", "LOCAL AI UPSCALER" }
                    h1 { "Nggedhekaké Gambar" }
                }
            }

            div { class: "steps",
                section { class: "step",
                    div { class: "step-heading",
                        span { class: "step-number", "01" }
                        div {
                            h2 { "Select image" }
                            p { "PNG, JPG, or WebP" }
                        }
                    }
                    button {
                        class: "file-picker",
                        disabled: is_processing,
                        onclick: select_image,
                        span { class: "picker-icon", "+" }
                        span { class: "picker-copy", "{image_label}" }
                    }
                }

                section { class: "step",
                    div { class: "step-heading",
                        span { class: "step-number", "02" }
                        div {
                            h2 { "Choose model" }
                            p { "Detail recovery profile" }
                        }
                    }
                    select {
                        class: "select-control",
                        value: "{selected_model}",
                        disabled: is_processing || state.models.read().is_empty(),
                        oninput: move |event| state.selected_model.set(event.value()),
                        if state.models.read().is_empty() {
                            option { "Loading models…" }
                        } else {
                            for model in state.models.read().iter() {
                                option { value: "{model}", "{model}" }
                            }
                        }
                    }
                }

                section { class: "step",
                    div { class: "step-heading",
                        span { class: "step-number", "03" }
                        div {
                            h2 { "Output folder" }
                            p { "Keep the original untouched" }
                        }
                    }
                    button {
                        class: "file-picker",
                        disabled: is_processing,
                        onclick: select_folder,
                        span { class: "picker-icon folder", "↗" }
                        span { class: "picker-copy", "{folder_label}" }
                    }
                }

                section { class: "step final-step",
                    div { class: "step-heading",
                        span { class: "step-number", "04" }
                        div {
                            h2 { "Size & format" }
                            p { "Choose final dimensions" }
                        }
                    }
                    div { class: "segmented scale-options",
                        for scale in [2, 3, 4] {
                            button {
                                class: if selected_scale == scale { "active" } else { "" },
                                disabled: is_processing,
                                onclick: move |_| state.scale.set(scale),
                                "{scale}×"
                            }
                        }
                    }
                    div { class: "format-row",
                        label { r#for: "format", "FORMAT" }
                        select {
                            id: "format",
                            value: "{selected_format}",
                            disabled: is_processing,
                            oninput: move |event| state.format.set(event.value()),
                            option { value: "png", "PNG" }
                            option { value: "jpg", "JPG" }
                            option { value: "webp", "WEBP" }
                        }
                    }
                }
            }

            if is_processing {
                button { class: "primary-action stop", onclick: stop_upscale, "Stop upscaling" }
            } else {
                button {
                    class: "primary-action",
                    disabled: !can_upscale,
                    onclick: start_upscale,
                    span { "Upscale image" }
                    span { class: "action-arrow", "↗" }
                }
            }
            p { class: "privacy-note", "Runs entirely on your Mac. No uploads." }
        }
    }
}
