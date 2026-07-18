use dioxus::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use upscale_contract::{
    DiskSpaceWarning, FailurePayload, OutputFormat, RetryKind, RetryUpscaleRequest, RunEvent,
    RunPhase, StartOutcome, StartUpscaleRequest, StartupStatus, UpscaleEngine,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RunSettings {
    pub image_path: String,
    pub output_folder: String,
    pub model: String,
    pub scale: u32,
    pub format: OutputFormat,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PendingStart {
    Normal(StartUpscaleRequest),
    Retry(RetryUpscaleRequest),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunAttempt {
    pub run_id: String,
    pub settings: RunSettings,
    pub engine: Option<UpscaleEngine>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiskWarningState {
    pub pending: PendingStart,
    pub attempt: RunAttempt,
    pub warning: DiskSpaceWarning,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveRunState {
    pub attempt: RunAttempt,
    pub phase: RunPhase,
    pub percent: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RunState {
    Ready,
    DiskWarning(DiskWarningState),
    Running(ActiveRunState),
    Cancelling(ActiveRunState),
    Cancelled {
        attempt: RunAttempt,
    },
    Failed {
        attempt: RunAttempt,
        failure: FailurePayload,
    },
    Completed {
        attempt: RunAttempt,
        output_path: String,
        was_cross_engine_retry: bool,
    },
}

impl Default for RunState {
    fn default() -> Self {
        Self::Ready
    }
}

impl RunState {
    pub fn attempt(&self) -> Option<&RunAttempt> {
        match self {
            Self::Ready => None,
            Self::DiskWarning(state) => Some(&state.attempt),
            Self::Running(state) | Self::Cancelling(state) => Some(&state.attempt),
            Self::Cancelled { attempt }
            | Self::Failed { attempt, .. }
            | Self::Completed { attempt, .. } => Some(attempt),
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running(_) | Self::Cancelling(_))
    }

    pub fn transition(self, event: RunEvent) -> Self {
        let Some(current_id) = self.attempt().map(|attempt| attempt.run_id.clone()) else {
            return self;
        };
        if event.run_id() != current_id {
            return self;
        }
        match (self, event) {
            (
                Self::Running(mut active),
                RunEvent::Progress {
                    engine,
                    phase,
                    percent,
                    ..
                },
            ) => {
                active.attempt.engine = Some(engine);
                active.phase = phase;
                active.percent = percent.map(|value| value.clamp(0.0, 99.9));
                Self::Running(active)
            }
            (Self::Cancelling(mut active), RunEvent::Progress { engine, .. }) => {
                active.attempt.engine = Some(engine);
                Self::Cancelling(active)
            }
            (
                Self::Running(mut active) | Self::Cancelling(mut active),
                RunEvent::Cancelled { engine, .. },
            ) => {
                active.attempt.engine = Some(engine);
                Self::Cancelled {
                    attempt: active.attempt,
                }
            }
            (Self::Running(mut active), RunEvent::Failed { failure }) => {
                active.attempt.engine = failure.engine;
                Self::Failed {
                    attempt: active.attempt,
                    failure,
                }
            }
            (
                Self::Running(mut active),
                RunEvent::Completed {
                    engine,
                    output_path,
                    was_cross_engine_retry,
                    ..
                },
            ) => {
                active.attempt.engine = Some(engine);
                Self::Completed {
                    attempt: active.attempt,
                    output_path,
                    was_cross_engine_retry,
                }
            }
            (current, _) => current,
        }
    }

    pub fn invalidated_by_settings_change(self, settings: &RunSettings) -> Self {
        let changed = self
            .attempt()
            .is_some_and(|attempt| attempt.settings != *settings);
        if changed
            && matches!(
                self,
                Self::DiskWarning(_)
                    | Self::Cancelled { .. }
                    | Self::Failed { .. }
                    | Self::Completed { .. }
            )
        {
            Self::Ready
        } else {
            self
        }
    }

    pub fn successful_alternative_offer(&self, primary: UpscaleEngine) -> Option<UpscaleEngine> {
        match self {
            Self::Completed {
                attempt,
                was_cross_engine_retry: true,
                ..
            } => attempt.engine.filter(|engine| *engine != primary),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub image_path: Signal<Option<String>>,
    pub output_folder: Signal<Option<String>>,
    pub selected_model: Signal<String>,
    pub scale: Signal<u32>,
    pub format: Signal<OutputFormat>,
    pub models: Signal<Vec<String>>,
    pub primary_engine: Signal<Option<UpscaleEngine>>,
    pub run_state: Signal<RunState>,
    pub ui_error: Signal<Option<String>>,
    pub startup_warning: Signal<Option<String>>,
    pub models_initialized: Signal<bool>,
    pub preference_initialized: Signal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            image_path: use_signal(|| None),
            output_folder: use_signal(|| None),
            selected_model: use_signal(|| "realesrgan-x4plus".into()),
            scale: use_signal(|| 4),
            format: use_signal(|| OutputFormat::Png),
            models: use_signal(Vec::new),
            primary_engine: use_signal(|| None),
            run_state: use_signal(RunState::default),
            ui_error: use_signal(|| None),
            startup_warning: use_signal(|| None),
            models_initialized: use_signal(|| false),
            preference_initialized: use_signal(|| false),
        }
    }

    pub fn settings(&self) -> Option<RunSettings> {
        Some(RunSettings {
            image_path: self.image_path.read().clone()?,
            output_folder: self.output_folder.read().clone()?,
            model: self.selected_model.read().clone(),
            scale: *self.scale.read(),
            format: *self.format.read(),
        })
    }

    pub fn controls_disabled(&self) -> bool {
        self.run_state.read().is_active()
    }

    pub fn can_start(&self) -> bool {
        self.settings().is_some()
            && !self.models.read().is_empty()
            && *self.models_initialized.read()
            && *self.preference_initialized.read()
            && self.primary_engine.read().is_some()
            && !self.controls_disabled()
    }

    fn invalidate_bound_state(&mut self) {
        let current = self.run_state.read().clone();
        let next = match self.settings() {
            Some(settings) => current.invalidated_by_settings_change(&settings),
            None if matches!(
                &current,
                RunState::DiskWarning(_)
                    | RunState::Cancelled { .. }
                    | RunState::Failed { .. }
                    | RunState::Completed { .. }
            ) =>
            {
                RunState::Ready
            }
            None => current,
        };
        self.run_state.set(next);
    }

    pub fn set_image_path(&mut self, value: String) {
        if self.image_path.read().as_ref() != Some(&value) {
            self.image_path.set(Some(value));
            self.invalidate_bound_state();
        }
    }

    pub fn set_output_folder(&mut self, value: String) {
        if self.output_folder.read().as_ref() != Some(&value) {
            self.output_folder.set(Some(value));
            self.invalidate_bound_state();
        }
    }

    pub fn set_model(&mut self, value: String) {
        if *self.selected_model.read() != value {
            self.selected_model.set(value);
            self.invalidate_bound_state();
        }
    }

    pub fn set_scale(&mut self, value: u32) {
        if *self.scale.read() != value {
            self.scale.set(value);
            self.invalidate_bound_state();
        }
    }

    pub fn set_format(&mut self, value: OutputFormat) {
        if *self.format.read() != value {
            self.format.set(value);
            self.invalidate_bound_state();
        }
    }

    pub fn apply_event(&mut self, event: RunEvent) {
        let current = self.run_state.read().clone();
        self.run_state.set(current.transition(event));
    }

    pub async fn start_normal_run(mut self) {
        let Some(settings) = self.settings() else {
            return;
        };
        let run_id = Uuid::new_v4().to_string();
        let request = StartUpscaleRequest {
            run_id: run_id.clone(),
            image_path: settings.image_path.clone(),
            output_folder: settings.output_folder.clone(),
            model: settings.model.clone(),
            scale: settings.scale,
            format: settings.format,
            acknowledge_low_disk_space: false,
        };
        let attempt = RunAttempt {
            run_id,
            settings,
            engine: *self.primary_engine.read(),
        };
        self.ui_error.set(None);
        self.run_state.set(RunState::Running(ActiveRunState {
            attempt: attempt.clone(),
            phase: RunPhase::Preflight,
            percent: None,
        }));
        match invoke_decode::<_, StartOutcome>("start_upscale", &RequestArg { request: &request })
            .await
        {
            Ok(outcome) => {
                self.apply_start_outcome(PendingStart::Normal(request), attempt, outcome)
            }
            Err(message) => {
                self.run_state.set(RunState::Ready);
                self.ui_error.set(Some(message));
            }
        }
    }

    pub async fn continue_after_disk_warning(mut self) {
        let RunState::DiskWarning(warning_state) = self.run_state.read().clone() else {
            return;
        };
        let pending = match warning_state.pending {
            PendingStart::Normal(mut request) => {
                request.acknowledge_low_disk_space = true;
                PendingStart::Normal(request)
            }
            PendingStart::Retry(mut request) => {
                request.acknowledge_low_disk_space = true;
                PendingStart::Retry(request)
            }
        };
        let attempt = warning_state.attempt;
        self.run_state.set(RunState::Running(ActiveRunState {
            attempt: attempt.clone(),
            phase: RunPhase::Preflight,
            percent: None,
        }));
        let result = match &pending {
            PendingStart::Normal(request) => {
                invoke_decode::<_, StartOutcome>("start_upscale", &RequestArg { request }).await
            }
            PendingStart::Retry(request) => {
                invoke_decode::<_, StartOutcome>("retry_upscale", &RequestArg { request }).await
            }
        };
        match result {
            Ok(outcome) => self.apply_start_outcome(pending, attempt, outcome),
            Err(message) => {
                self.run_state.set(RunState::Ready);
                self.ui_error.set(Some(message));
            }
        }
    }

    pub async fn retry_failed(mut self, kind: RetryKind) {
        let RunState::Failed {
            attempt: failed, ..
        } = self.run_state.read().clone()
        else {
            return;
        };
        let (request, attempt) = Self::retry_attempt(failed, kind);
        self.run_state.set(RunState::Running(ActiveRunState {
            attempt: attempt.clone(),
            phase: RunPhase::Preflight,
            percent: None,
        }));
        match invoke_decode::<_, StartOutcome>("retry_upscale", &RequestArg { request: &request })
            .await
        {
            Ok(outcome) => self.apply_start_outcome(PendingStart::Retry(request), attempt, outcome),
            Err(message) => {
                self.run_state.set(RunState::Ready);
                self.ui_error.set(Some(message));
            }
        }
    }

    fn retry_attempt(failed: RunAttempt, kind: RetryKind) -> (RetryUpscaleRequest, RunAttempt) {
        let run_id = Uuid::new_v4().to_string();
        let engine = match kind {
            RetryKind::SameEngine => failed.engine,
            RetryKind::AlternativeEngine => failed.engine.map(UpscaleEngine::alternative),
        };
        let request = RetryUpscaleRequest {
            run_id: run_id.clone(),
            failed_run_id: failed.run_id,
            kind,
            acknowledge_low_disk_space: false,
        };
        let attempt = RunAttempt {
            run_id,
            settings: failed.settings,
            engine,
        };
        (request, attempt)
    }

    fn apply_start_outcome(
        &mut self,
        pending: PendingStart,
        mut attempt: RunAttempt,
        outcome: StartOutcome,
    ) {
        match outcome {
            StartOutcome::Started { engine, .. } => {
                let current = self.run_state.read().clone();
                if let RunState::Running(mut active) = current {
                    if active.attempt.run_id == attempt.run_id {
                        active.attempt.engine = Some(engine);
                        if active.phase == RunPhase::Preflight {
                            active.phase = RunPhase::StartingEngine;
                        }
                        self.run_state.set(RunState::Running(active));
                    }
                }
            }
            StartOutcome::NeedsDiskConfirmation { warning } => {
                self.run_state.set(RunState::DiskWarning(DiskWarningState {
                    pending,
                    attempt,
                    warning,
                }));
            }
            StartOutcome::Rejected { failure } => {
                attempt.engine = failure.engine;
                self.run_state.set(RunState::Failed { attempt, failure });
            }
        }
    }

    pub async fn cancel_active_run(mut self) {
        let RunState::Running(active) = self.run_state.read().clone() else {
            return;
        };
        let run_id = active.attempt.run_id.clone();
        self.run_state.set(RunState::Cancelling(active.clone()));
        if let Err(message) = invoke_command("cancel_upscale", &RunIdArg { run_id: &run_id }).await
        {
            if matches!(&*self.run_state.read(), RunState::Cancelling(current) if current.attempt.run_id == run_id)
            {
                self.run_state.set(RunState::Running(active));
            }
            self.ui_error.set(Some(message));
        }
    }

    pub async fn persist_engine_preference(mut self, engine: UpscaleEngine) {
        match invoke_decode::<_, UpscaleEngine>("set_engine_preference", &EngineArg { engine })
            .await
        {
            Ok(saved) => {
                self.primary_engine.set(Some(saved));
                self.preference_initialized.set(true);
            }
            Err(message) => self.ui_error.set(Some(message)),
        }
    }

    pub async fn copy_technical_details(mut self, text: String) -> bool {
        match invoke_command("copy_technical_details", &TextArg { text: &text }).await {
            Ok(_) => true,
            Err(message) => {
                self.ui_error.set(Some(message));
                false
            }
        }
    }
}

#[derive(Serialize)]
struct RequestArg<'a, T> {
    request: &'a T,
}

#[derive(Serialize)]
struct RunIdArg<'a> {
    #[serde(rename = "runId")]
    run_id: &'a str,
}

#[derive(Serialize)]
struct EngineArg {
    engine: UpscaleEngine,
}

#[derive(Serialize)]
struct TextArg<'a> {
    text: &'a str,
}

#[derive(Serialize)]
pub struct EmptyArgs {}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = convertFileSrc)]
    fn convert_file_src(path: &str) -> String;
}

pub async fn invoke_command<T: Serialize>(command: &str, args: &T) -> Result<JsValue, String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    tauri_invoke(command, args)
        .await
        .map_err(|error| error.as_string().unwrap_or_else(|| format!("{error:?}")))
}

pub async fn invoke_decode<T: Serialize, R: DeserializeOwned>(
    command: &str,
    args: &T,
) -> Result<R, String> {
    let value = invoke_command(command, args).await?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub fn image_url(path: &str) -> String {
    convert_file_src(path)
}

pub fn startup_warning(status: StartupStatus) -> Option<String> {
    (!status.cleanup_failures.is_empty()).then(|| {
        "Some unfinished output files could not be cleaned. Reconnect the destination or restore access, then restart the app.".into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use upscale_contract::{FailureKind, RecoveryAction};

    fn attempt(id: &str) -> RunAttempt {
        RunAttempt {
            run_id: id.into(),
            settings: RunSettings {
                image_path: "/source.png".into(),
                output_folder: "/output".into(),
                model: "model".into(),
                scale: 2,
                format: OutputFormat::Png,
            },
            engine: Some(UpscaleEngine::Upscayl),
        }
    }

    fn running(id: &str) -> RunState {
        RunState::Running(ActiveRunState {
            attempt: attempt(id),
            phase: RunPhase::Preflight,
            percent: None,
        })
    }

    fn failure(id: &str) -> FailurePayload {
        FailurePayload {
            run_id: id.into(),
            kind: FailureKind::EngineProcessing,
            phase: RunPhase::Processing,
            user_message: "failed".into(),
            technical_details: "details".into(),
            engine: Some(UpscaleEngine::Upscayl),
            source_dimensions: Some([10, 10]),
            exit_status: Some(1),
            actions: vec![RecoveryAction::RetrySameEngine],
        }
    }

    #[test]
    fn terminal_states_retain_the_exact_attempt_snapshot() {
        let terminal = RunState::Completed {
            attempt: attempt("run"),
            output_path: "/output/result.png".into(),
            was_cross_engine_retry: false,
        };
        assert_eq!(terminal.attempt().unwrap().settings.scale, 2);
        assert_eq!(
            terminal.attempt().unwrap().settings.format,
            OutputFormat::Png
        );
    }

    #[test]
    fn active_and_terminal_classification_is_explicit() {
        let active = ActiveRunState {
            attempt: attempt("run"),
            phase: RunPhase::Processing,
            percent: Some(50.0),
        };
        assert!(RunState::Running(active.clone()).is_active());
        assert!(RunState::Cancelling(active).is_active());
        assert!(
            !RunState::Cancelled {
                attempt: attempt("run")
            }
            .is_active()
        );
    }

    #[test]
    fn stale_and_duplicate_terminal_events_are_ignored() {
        let current = running("current");
        let stale = RunEvent::Progress {
            run_id: "stale".into(),
            engine: UpscaleEngine::RealEsrgan,
            phase: RunPhase::Processing,
            percent: Some(50.0),
        };
        assert_eq!(current.clone().transition(stale), current);

        let completed = running("current").transition(RunEvent::Completed {
            run_id: "current".into(),
            engine: UpscaleEngine::Upscayl,
            output_path: "/output/result.png".into(),
            was_cross_engine_retry: false,
        });
        assert_eq!(
            completed.clone().transition(RunEvent::Failed {
                failure: failure("current"),
            }),
            completed
        );
    }

    #[test]
    fn progress_is_clamped_and_cancelling_does_not_roll_back() {
        let progressed = running("run").transition(RunEvent::Progress {
            run_id: "run".into(),
            engine: UpscaleEngine::Upscayl,
            phase: RunPhase::Processing,
            percent: Some(100.0),
        });
        assert!(matches!(
            progressed,
            RunState::Running(ActiveRunState {
                percent: Some(value),
                ..
            }) if value == 99.9
        ));

        let cancelling = RunState::Cancelling(ActiveRunState {
            attempt: attempt("run"),
            phase: RunPhase::Processing,
            percent: Some(20.0),
        });
        let transitioned = cancelling.transition(RunEvent::Progress {
            run_id: "run".into(),
            engine: UpscaleEngine::Upscayl,
            phase: RunPhase::Finalizing,
            percent: Some(90.0),
        });
        assert!(matches!(
            transitioned,
            RunState::Cancelling(ActiveRunState {
                phase: RunPhase::Processing,
                percent: Some(20.0),
                ..
            })
        ));
    }

    #[test]
    fn terminal_state_invalidation_is_bound_to_all_five_settings() {
        let terminal = RunState::Completed {
            attempt: attempt("run"),
            output_path: "/output/result.png".into(),
            was_cross_engine_retry: false,
        };
        let original = terminal.attempt().unwrap().settings.clone();
        assert_eq!(
            terminal.clone().invalidated_by_settings_change(&original),
            terminal
        );

        let mut variants = Vec::new();
        let mut source = original.clone();
        source.image_path.push('2');
        variants.push(source);
        let mut destination = original.clone();
        destination.output_folder.push('2');
        variants.push(destination);
        let mut model = original.clone();
        model.model.push('2');
        variants.push(model);
        let mut scale = original.clone();
        scale.scale = 3;
        variants.push(scale);
        let mut format = original;
        format.format = OutputFormat::Webp;
        variants.push(format);

        for settings in variants {
            assert_eq!(
                terminal.clone().invalidated_by_settings_change(&settings),
                RunState::Ready
            );
        }
    }

    #[test]
    fn retries_use_fresh_ids_and_symmetric_engine_roles() {
        let failed = attempt("failed");
        let (same_request, same_attempt) =
            AppState::retry_attempt(failed.clone(), RetryKind::SameEngine);
        assert_ne!(same_request.run_id, "failed");
        assert_eq!(same_request.failed_run_id, "failed");
        assert_eq!(same_attempt.settings, failed.settings);
        assert_eq!(same_attempt.engine, Some(UpscaleEngine::Upscayl));

        let (_, alternative) = AppState::retry_attempt(failed, RetryKind::AlternativeEngine);
        assert_eq!(alternative.engine, Some(UpscaleEngine::RealEsrgan));
    }

    #[test]
    fn preference_offer_requires_an_opt_in_cross_engine_success() {
        let state = RunState::Completed {
            attempt: RunAttempt {
                engine: Some(UpscaleEngine::RealEsrgan),
                ..attempt("run")
            },
            output_path: "/output/result.png".into(),
            was_cross_engine_retry: true,
        };
        assert_eq!(
            state.successful_alternative_offer(UpscaleEngine::Upscayl),
            Some(UpscaleEngine::RealEsrgan)
        );
        assert_eq!(
            state.successful_alternative_offer(UpscaleEngine::RealEsrgan),
            None
        );
    }
}
