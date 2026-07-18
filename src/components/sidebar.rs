use crate::state::{AppState, EmptyArgs, RunState, invoke_command};
use dioxus::prelude::*;
use std::path::Path;
use upscale_contract::{OutputFormat, UpscaleEngine};

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
    let selected_model = state.selected_model.read().clone();
    let selected_scale = *state.scale.read();
    let selected_format = *state.format.read();
    let selected_format_value = selected_format.extension();
    let run_state = state.run_state.read().clone();
    let controls_disabled = run_state.is_active();
    let can_upscale = state.can_start();

    let select_image = move |_| async move {
        match invoke_command("select_image", &EmptyArgs {}).await {
            Ok(value) => {
                if let Some(path) = value.as_string() {
                    state.set_image_path(path);
                    state.ui_error.set(None);
                }
            }
            Err(message) => state.ui_error.set(Some(message)),
        }
    };

    let select_folder = move |_| async move {
        match invoke_command("select_folder", &EmptyArgs {}).await {
            Ok(value) => {
                if let Some(path) = value.as_string() {
                    state.set_output_folder(path);
                    state.ui_error.set(None);
                }
            }
            Err(message) => state.ui_error.set(Some(message)),
        }
    };

    let primary_engine = *state.primary_engine.read();

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
                        disabled: controls_disabled,
                        onclick: select_image,
                        span { class: "picker-icon", "+" }
                        span { class: "picker-copy", "{image_label}" }
                    }
                }

                section { class: "step",
                    div { class: "step-heading",
                        span { class: "step-number", "02" }
                        div {
                            h2 { "Model" }
                            p { "Choose the upscaling model" }
                        }
                    }
                    select {
                        class: "select-control",
                        value: "{selected_model}",
                        disabled: controls_disabled || state.models.read().is_empty(),
                        oninput: move |event| state.set_model(event.value()),
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
                        disabled: controls_disabled,
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
                                disabled: controls_disabled,
                                onclick: move |_| state.set_scale(scale),
                                "{scale}×"
                            }
                        }
                    }
                    div { class: "format-row",
                        label { r#for: "format", "FORMAT" }
                        select {
                            id: "format",
                            value: "{selected_format_value}",
                            disabled: controls_disabled,
                            oninput: move |event| {
                                let format = match event.value().as_str() {
                                    "jpg" => OutputFormat::Jpg,
                                    "webp" => OutputFormat::Webp,
                                    _ => OutputFormat::Png,
                                };
                                state.set_format(format);
                            },
                            option { value: "png", "PNG" }
                            option { value: "jpg", "JPG" }
                            option { value: "webp", "WEBP" }
                        }
                    }
                }
            }

            match run_state {
                RunState::Running(_) => rsx! {
                    button {
                        class: "primary-action stop",
                        onclick: move |_| async move { state.cancel_active_run().await },
                        "Stop upscaling"
                    }
                },
                RunState::Cancelling(_) => rsx! {
                    button { class: "primary-action stop", disabled: true, "Stopping…" }
                },
                RunState::DiskWarning(_) => rsx! {
                    div { class: "sidebar-actions",
                        button {
                            class: "primary-action",
                            onclick: move |_| async move { state.continue_after_disk_warning().await },
                            "Continue anyway"
                        }
                        button { class: "secondary-action full-width", onclick: select_folder, "Choose another destination" }
                    }
                },
                _ => rsx! {
                    button {
                        class: "primary-action",
                        disabled: !can_upscale,
                        onclick: move |_| async move { state.start_normal_run().await },
                        span { "Upscale image" }
                        span { class: "action-arrow", "↗" }
                    }
                },
            }

            details { class: "settings-disclosure",
                summary { "Device settings" }
                div { class: "settings-row",
                    label { r#for: "preferred-engine", "Preferred engine" }
                    select {
                        id: "preferred-engine",
                        disabled: controls_disabled || !*state.preference_initialized.read(),
                        value: match primary_engine {
                            Some(UpscaleEngine::RealEsrgan) => "real-esrgan",
                            _ => "upscayl",
                        },
                        oninput: move |event| {
                            let engine = if event.value() == "real-esrgan" {
                                UpscaleEngine::RealEsrgan
                            } else {
                                UpscaleEngine::Upscayl
                            };
                            spawn(async move { state.persist_engine_preference(engine).await });
                        },
                        option { value: "upscayl", "Upscayl NCNN" }
                        option { value: "real-esrgan", "Official Real-ESRGAN" }
                    }
                }
            }
            p { class: "privacy-note", "Runs entirely on your Mac. No uploads." }
        }
    }
}
