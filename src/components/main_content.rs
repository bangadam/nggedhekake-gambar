use crate::{
    components::image_slider::ImageSlider,
    state::{AppState, image_url, invoke_command},
};
use dioxus::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct OpenFolderArgs<'a> {
    path: &'a str,
}

#[component]
pub fn MainContent() -> Element {
    let mut state = use_context::<AppState>();
    let image_path = state.image_path.read().clone();
    let result_path = state.upscaled_image_path.read().clone();
    let is_processing = (state.is_processing)();
    let progress = (state.progress)().clamp(0.0, 100.0);
    let scale_label = (state.scale)();
    let format_label = (state.format)().to_uppercase();
    let progress_message = (state.progress_message)();
    let progress_label = if progress > 0.0 {
        format!("{progress:.0}%")
    } else {
        "Starting".into()
    };

    let original_url = image_path.as_deref().map(image_url);
    let result_url = result_path.as_deref().map(image_url);

    let open_output = move |_| async move {
        let Some(folder) = state.output_folder.read().clone() else {
            return;
        };
        let mut error = state.error;
        if let Err(message) = invoke_command("open_folder", &OpenFolderArgs { path: &folder }).await
        {
            error.set(Some(message));
        }
    };

    rsx! {
        main { class: "main-content",
            header { class: "workspace-header",
                div {
                    p { class: "eyebrow", "IMAGE WORKSPACE" }
                    h2 {
                        if result_path.is_some() { "Compare the result" }
                        else if image_path.is_some() { "Ready to upscale" }
                        else { "Bring small details back" }
                    }
                }
                div { class: "workspace-meta",
                    span { "{scale_label}×" }
                    span { "{format_label}" }
                    span { class: "status-dot", "LOCAL" }
                }
            }

            section { class: "preview-stage",
                div { class: "stage-grid" }
                match (original_url, result_url) {
                    (Some(before), Some(after)) => rsx! {
                        ImageSlider { before, after }
                    },
                    (Some(image), None) => rsx! {
                        div { class: "single-preview",
                            img { src: "{image}", alt: "Selected image preview" }
                            if is_processing {
                                div { class: "processing-scrim",
                                    div { class: "processing-orbit" }
                                    span { "ENHANCING" }
                                }
                            }
                        }
                    },
                    _ => rsx! {
                        div { class: "empty-state",
                            div { class: "empty-art",
                                div { class: "empty-frame outer" }
                                div { class: "empty-frame inner" }
                                span { "4×" }
                            }
                            p { class: "eyebrow", "AI-POWERED DETAIL RECOVERY" }
                            h3 { "A sharper version is one image away." }
                            p { class: "empty-copy", "Select a photo or illustration to preview it here. Processing stays on-device." }
                        }
                    }
                }
            }

            if is_processing {
                section { class: "progress-panel",
                    div { class: "progress-copy",
                        div {
                            p { class: "eyebrow", "UPSCALING IN PROGRESS" }
                            h3 { "{progress_message}" }
                        }
                        strong { "{progress_label}" }
                    }
                    div { class: "progress-track",
                        div { class: "progress-fill", style: "width: {progress}%" }
                    }
                }
            } else if result_path.is_some() {
                section { class: "result-panel",
                    div {
                        p { class: "eyebrow success", "UPSCALE COMPLETE" }
                        h3 { "Your enhanced image is ready." }
                    }
                    button { class: "secondary-action", onclick: open_output, "Show in folder ↗" }
                }
            } else {
                footer { class: "workspace-footer",
                    span { "01" }
                    p { "Use the controls on the left, then start the local enhancement pass." }
                    div { class: "footer-line" }
                }
            }

            if let Some(message) = state.error.read().as_ref() {
                div { class: "error-toast", role: "alert",
                    span { "{message}" }
                    button { onclick: move |_| state.error.set(None), aria_label: "Dismiss error", "×" }
                }
            }
        }
    }
}
