use crate::{
    components::image_slider::ImageSlider,
    state::{AppState, EmptyArgs, RunAttempt, RunState, image_url, invoke_command},
};
use dioxus::prelude::*;
use serde::Serialize;
use std::path::Path;
use upscale_contract::{
    FailurePayload, OutputFormat, RecoveryAction, RetryKind, RunPhase, UpscaleEngine,
};

#[derive(Serialize)]
struct OpenFolderArgs<'a> {
    path: &'a str,
}

fn phase_copy(phase: RunPhase, engine: Option<UpscaleEngine>) -> String {
    match phase {
        RunPhase::Preflight => "Checking image and destination…".into(),
        RunPhase::StartingEngine => format!(
            "Starting {}…",
            engine
                .map(UpscaleEngine::display_name)
                .unwrap_or("upscaler")
        ),
        RunPhase::Processing => "Upscaling image…".into(),
        RunPhase::Validating => "Checking the upscaled image…".into(),
        RunPhase::Finalizing => "Saving upscaled image…".into(),
    }
}

fn format_label(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Png => "PNG",
        OutputFormat::Jpg => "JPG",
        OutputFormat::Webp => "WEBP",
    }
}

#[component]
fn FailurePanel(attempt: RunAttempt, failure: FailurePayload) -> Element {
    let mut state = use_context::<AppState>();
    let mut copied = use_signal(|| false);
    let has_alternative = failure
        .actions
        .iter()
        .any(|action| matches!(action, RecoveryAction::RetryAlternative { .. }));

    rsx! {
        section { class: "lifecycle-panel failure-panel", role: "alert",
            div { class: "lifecycle-copy",
                p { class: "eyebrow failure", "UPSCALE FAILED" }
                h3 { "{failure.user_message}" }
                if let Some(engine) = failure.engine {
                    p { class: "supporting-copy", "Engine: {engine.display_name()}" }
                }
                if has_alternative {
                    p { class: "supporting-copy", "This may take longer and produce a different output." }
                }
            }
            div { class: "action-group",
                for action in failure.actions.clone() {
                    match action {
                        RecoveryAction::ChooseSource => rsx! {
                            button {
                                class: "secondary-action",
                                onclick: move |_| async move {
                                    match invoke_command("select_image", &EmptyArgs {}).await {
                                        Ok(value) => if let Some(path) = value.as_string() { state.set_image_path(path); },
                                        Err(message) => state.ui_error.set(Some(message)),
                                    }
                                },
                                "Choose another image"
                            }
                        },
                        RecoveryAction::ChooseDestination => rsx! {
                            button {
                                class: "secondary-action",
                                onclick: move |_| async move {
                                    match invoke_command("select_folder", &EmptyArgs {}).await {
                                        Ok(value) => if let Some(path) = value.as_string() { state.set_output_folder(path); },
                                        Err(message) => state.ui_error.set(Some(message)),
                                    }
                                },
                                "Choose another destination"
                            }
                        },
                        RecoveryAction::LowerScale => rsx! {
                            button {
                                class: "secondary-action",
                                onclick: move |_| {
                                    let next = attempt.settings.scale.saturating_sub(1).max(2);
                                    state.set_scale(next);
                                },
                                "Lower scale"
                            }
                        },
                        RecoveryAction::RetrySameEngine => rsx! {
                            button {
                                class: "secondary-action",
                                onclick: move |_| async move { state.retry_failed(RetryKind::SameEngine).await },
                                "Try again"
                            }
                        },
                        RecoveryAction::RetryAlternative { engine } => rsx! {
                            button {
                                class: "secondary-action emphasis",
                                onclick: move |_| async move { state.retry_failed(RetryKind::AlternativeEngine).await },
                                "Try with {engine.display_name()}"
                            }
                        },
                        RecoveryAction::CopyTechnicalDetails => {
                            let details = failure.technical_details.clone();
                            rsx! {
                                button {
                                    class: "secondary-action quiet",
                                    onclick: move |_| {
                                        let details = details.clone();
                                        spawn(async move {
                                            if state.copy_technical_details(details).await {
                                                copied.set(true);
                                            }
                                        });
                                    },
                                    if copied() { "Copied" } else { "Copy technical details" }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn CompletedPanel(
    attempt: RunAttempt,
    output_path: String,
    was_cross_engine_retry: bool,
) -> Element {
    let mut state = use_context::<AppState>();
    let engine = attempt.engine;
    let parent = Path::new(&output_path)
        .parent()
        .map(|path| path.to_string_lossy().into_owned());
    let offer_engine = (*state.primary_engine.read()).and_then(|primary| {
        RunState::Completed {
            attempt: attempt.clone(),
            output_path: output_path.clone(),
            was_cross_engine_retry,
        }
        .successful_alternative_offer(primary)
    });
    let should_offer = offer_engine.is_some();

    rsx! {
        section { class: "lifecycle-panel result-panel",
            div {
                p { class: "eyebrow success", "UPSCALE COMPLETE" }
                h3 { "Your upscaled image is ready." }
                if was_cross_engine_retry {
                    if let Some(engine) = engine {
                        p { class: "supporting-copy", "Completed with {engine.display_name()}." }
                    }
                }
            }
            div { class: "action-group horizontal",
                if let Some(folder) = parent {
                    button {
                        class: "secondary-action",
                        onclick: move |_| {
                            let folder = folder.clone();
                            spawn(async move {
                                if let Err(message) = invoke_command("open_folder", &OpenFolderArgs { path: &folder }).await {
                                    state.ui_error.set(Some(message));
                                }
                            });
                        },
                        "Show in folder ↗"
                    }
                }
                if should_offer {
                    if let Some(engine) = engine {
                        button {
                            class: "secondary-action emphasis",
                            onclick: move |_| async move { state.persist_engine_preference(engine).await },
                            "Use {engine.display_name()} for future runs on this Mac"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MainContent() -> Element {
    let mut state = use_context::<AppState>();
    let run_state = state.run_state.read().clone();
    let attempt = run_state.attempt().cloned();
    let image_path = state.image_path.read().clone();
    let result_path = match &run_state {
        RunState::Completed { output_path, .. } => Some(output_path.clone()),
        _ => None,
    };
    let active = run_state.is_active();
    let settings = attempt.as_ref().map(|attempt| &attempt.settings);
    let scale_label = settings
        .map(|settings| settings.scale)
        .unwrap_or(*state.scale.read());
    let selected_format = settings
        .map(|settings| settings.format)
        .unwrap_or(*state.format.read());
    let engine = attempt.as_ref().and_then(|attempt| attempt.engine);
    let original_url = image_path.as_deref().map(image_url);
    let result_url = result_path.as_deref().map(image_url);

    rsx! {
        main { class: "main-content",
            header { class: "workspace-header",
                div {
                    p { class: "eyebrow", "IMAGE WORKSPACE" }
                    h2 {
                        match run_state {
                            RunState::Completed { .. } => "Compare the result",
                            RunState::Failed { .. } => "The run needs attention",
                            RunState::Cancelled { .. } => "Ready when you are",
                            _ if image_path.is_some() => "Ready to upscale",
                            _ => "Bring small details back",
                        }
                    }
                }
                div { class: "workspace-meta",
                    span { "{scale_label}×" }
                    span { "{format_label(selected_format)}" }
                    if matches!(run_state, RunState::Running(_) | RunState::Cancelling(_) | RunState::Failed { .. } | RunState::Completed { .. }) {
                        if let Some(engine) = engine {
                            span { "{engine.display_name()}" }
                        }
                    }
                    span { class: "status-dot", "LOCAL" }
                }
            }

            if let Some(message) = state.startup_warning.read().as_ref() {
                div { class: "startup-warning", role: "status", "{message}" }
            }

            section { class: "preview-stage",
                div { class: "stage-grid" }
                match (original_url, result_url) {
                    (Some(before), Some(after)) => rsx! { ImageSlider { before, after } },
                    (Some(image), None) => rsx! {
                        div { class: "single-preview",
                            img { src: "{image}", alt: "Selected image preview" }
                            if active {
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
                    },
                }
            }

            match run_state {
                RunState::Running(active) | RunState::Cancelling(active) => {
                    let label = phase_copy(active.phase, active.attempt.engine);
                    let percent = active.percent.map(|value| value.clamp(0.0, 99.9));
                    rsx! {
                        section { class: "progress-panel",
                            div { class: "progress-copy",
                                div {
                                    p { class: "eyebrow", "UPSCALING IN PROGRESS" }
                                    h3 { "{label}" }
                                }
                                strong {
                                    if let Some(percent) = percent { "{percent:.0}%" } else { "Working" }
                                }
                            }
                            if let Some(percent) = percent {
                                div {
                                    class: "progress-track",
                                    role: "progressbar",
                                    aria_valuemin: "0",
                                    aria_valuemax: "100",
                                    aria_valuenow: "{percent:.0}",
                                    div { class: "progress-fill", style: "width: {percent}%" }
                                }
                            } else {
                                div {
                                    class: "progress-track indeterminate",
                                    role: "progressbar",
                                    aria_valuemin: "0",
                                    aria_valuemax: "100",
                                    div { class: "progress-fill" }
                                }
                            }
                        }
                    }
                },
                RunState::DiskWarning(warning) => rsx! {
                    section { class: "lifecycle-panel warning-panel", role: "status",
                        div {
                            p { class: "eyebrow warning", "CHECK DESTINATION" }
                            h3 { "{warning.warning.message}" }
                            p { class: "supporting-copy", "No upscaler is running. Continue only if you expect more space to become available." }
                        }
                    }
                },
                RunState::Cancelled { .. } => rsx! {
                    section { class: "lifecycle-panel cancelled-panel", role: "status",
                        div {
                            p { class: "eyebrow", "UPSCALE CANCELLED" }
                            h3 { "Upscaling was cancelled." }
                            p { class: "supporting-copy", "Your source image and settings are unchanged." }
                        }
                    }
                },
                RunState::Failed { attempt, failure } => rsx! {
                    FailurePanel { attempt, failure }
                },
                RunState::Completed { attempt, output_path, was_cross_engine_retry } => rsx! {
                    CompletedPanel { attempt, output_path, was_cross_engine_retry }
                },
                RunState::Ready => rsx! {
                    footer { class: "workspace-footer",
                        span { "01" }
                        p { "Use the controls on the left, then start the local enhancement pass." }
                        div { class: "footer-line" }
                    }
                },
            }

            if let Some(message) = state.ui_error.read().as_ref() {
                div { class: "error-toast", role: "alert",
                    span { "{message}" }
                    button { onclick: move |_| state.ui_error.set(None), aria_label: "Dismiss error", "×" }
                }
            }
        }
    }
}
