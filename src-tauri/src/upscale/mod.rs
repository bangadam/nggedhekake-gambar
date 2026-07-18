mod engine;
mod files;
mod state;

use self::{
    engine::{
        accept_progress, drain_stream, kill_process_group, CommandRunner, EngineAdapter,
        EngineRunner,
    },
    files::{
        cleanup_startup, disk_warning, preflight, publish_exclusive, temporary_path,
        validate_output, OwnedJournal, Preflight, PreflightError,
    },
    state::{
        diagnostics, load_preference, persist_preference, recovery_actions, FailedRunRecord,
        RunSnapshot, ServiceState,
    },
};
use std::{
    path::PathBuf,
    sync::{atomic::Ordering, Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use upscale_contract::{
    FailureKind, FailurePayload, RetryKind, RetryUpscaleRequest, RunEvent, RunPhase, StartOutcome,
    StartUpscaleRequest, StartupStatus, UpscaleEngine,
};
use uuid::Uuid;

pub use files::paired_models;

#[derive(Clone)]
pub struct UpscaleService(Arc<ServiceInner>);

struct ServiceInner {
    app: AppHandle,
    state: Mutex<ServiceState>,
    runner: Arc<dyn EngineRunner>,
    resources: PathBuf,
    runs_dir: PathBuf,
    config_dir: PathBuf,
    startup_status: StartupStatus,
    app_version: String,
}

impl UpscaleService {
    pub fn new(
        app: AppHandle,
        resources: PathBuf,
        app_data_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Self {
        let runs_dir = app_data_dir.join("runs");
        let startup_status = cleanup_startup(&runs_dir);
        let preference = load_preference(&config_dir);
        Self(Arc::new(ServiceInner {
            app,
            state: Mutex::new(ServiceState::new(preference)),
            runner: Arc::new(CommandRunner),
            resources,
            runs_dir,
            config_dir,
            startup_status,
            app_version: env!("CARGO_PKG_VERSION").into(),
        }))
    }

    #[cfg(test)]
    fn with_runner(
        app: AppHandle,
        resources: PathBuf,
        app_data_dir: PathBuf,
        config_dir: PathBuf,
        runner: Arc<dyn EngineRunner>,
    ) -> Self {
        let service = Self::new(app, resources, app_data_dir, config_dir);
        let inner = Arc::try_unwrap(service.0)
            .ok()
            .expect("new service has one owner");
        Self(Arc::new(ServiceInner { runner, ..inner }))
    }

    pub fn start(&self, request: StartUpscaleRequest) -> Result<StartOutcome, String> {
        let engine = {
            let mut state = self.lock_state()?;
            state.failed = None;
            state.preference.ok_or_else(|| {
                state
                    .preference_error
                    .clone()
                    .unwrap_or_else(|| "Engine preference is unavailable".into())
            })?
        };
        self.start_snapshot(RunSnapshot {
            request,
            engine,
            cross_engine_retry_used: false,
        })
    }

    pub fn retry(&self, request: RetryUpscaleRequest) -> Result<StartOutcome, String> {
        let failed = {
            let state = self.lock_state()?;
            state
                .failed
                .as_ref()
                .filter(|record| record.failure.run_id == request.failed_run_id)
                .cloned()
                .ok_or_else(|| "That failed run is no longer available for retry".to_string())?
        };
        let eligible =
            match request.kind {
                RetryKind::SameEngine => failed.failure.actions.iter().any(|action| {
                    matches!(action, upscale_contract::RecoveryAction::RetrySameEngine)
                }),
                RetryKind::AlternativeEngine => failed.failure.actions.iter().any(|action| {
                    matches!(
                        action,
                        upscale_contract::RecoveryAction::RetryAlternative { .. }
                    )
                }),
            };
        if !eligible {
            return Err("That retry is not available for this failure".into());
        }
        Uuid::parse_str(&request.run_id).map_err(|_| "Run ID is not a valid UUID".to_string())?;
        if request.run_id == request.failed_run_id {
            return Err("A retry requires a fresh run ID".into());
        }
        let mut snapshot = failed.snapshot;
        snapshot.request.run_id = request.run_id;
        snapshot.request.acknowledge_low_disk_space = request.acknowledge_low_disk_space;
        if request.kind == RetryKind::AlternativeEngine {
            snapshot.engine = snapshot.engine.alternative();
            snapshot.cross_engine_retry_used = true;
        }
        self.start_snapshot(snapshot)
    }

    fn start_snapshot(&self, snapshot: RunSnapshot) -> Result<StartOutcome, String> {
        let run_id = Uuid::parse_str(&snapshot.request.run_id)
            .map_err(|_| "Run ID is not a valid UUID".to_string())?;
        {
            let mut state = self.lock_state()?;
            state.reserve(snapshot.clone())?;
        }

        let prepared = match preflight(
            &snapshot.request.run_id,
            &snapshot.request.image_path,
            &snapshot.request.output_folder,
            &snapshot.request.model,
            snapshot.request.scale,
            snapshot.engine,
            &self.0.resources,
        ) {
            Ok(prepared) => prepared,
            Err(error) => return Ok(self.reject_preflight(snapshot, run_id, error)),
        };
        if !snapshot.request.acknowledge_low_disk_space {
            if let Some(warning) = disk_warning(&prepared) {
                self.lock_state()?.clear_reservation(run_id);
                return Ok(StartOutcome::NeedsDiskConfirmation { warning });
            }
        }

        let adapter = EngineAdapter::new(snapshot.engine);
        let temporary = temporary_path(
            &prepared.destination,
            prepared.run_id,
            snapshot.request.format,
        );
        let mut owned_paths = vec![temporary.clone()];
        owned_paths.extend(adapter.owned_companion_paths(&temporary, snapshot.request.format));
        let journal = match OwnedJournal::create(
            &self.0.runs_dir,
            run_id,
            snapshot.engine,
            owned_paths.clone(),
        ) {
            Ok(journal) => journal,
            Err(error) => {
                let failure = self.make_failure(
                    &snapshot,
                    FailureKind::Internal,
                    RunPhase::Preflight,
                    Some(prepared.source_dimensions),
                    None,
                    "The upscale run couldn't be completed because the app encountered an internal problem.",
                    &format!("Could not create the ownership journal: {error}"),
                    &owned_paths,
                    false,
                );
                return Ok(self.reject_failure(snapshot, run_id, failure));
            }
        };
        let invocation = adapter.build_invocation(
            &prepared.binary,
            &prepared.source,
            &temporary,
            &prepared.models,
            &snapshot.request.model,
            snapshot.request.scale,
            snapshot.request.format,
        );

        if let Err(error) = journal.set_inherited(true) {
            let failure = self.make_failure(
                &snapshot,
                FailureKind::Internal,
                RunPhase::StartingEngine,
                Some(prepared.source_dimensions),
                None,
                "The upscale run couldn't be completed because the app encountered an internal problem.",
                &format!("Could not prepare the ownership lock for inheritance: {error}"),
                &owned_paths,
                false,
            );
            let _ = journal.cleanup();
            return Ok(self.reject_failure(snapshot, run_id, failure));
        }
        let process_result = self.0.runner.spawn(&invocation);
        let restore_result = journal.set_inherited(false);
        let mut process = match process_result {
            Ok(process) => process,
            Err(error) => {
                let failure = self.make_failure(
                    &snapshot,
                    FailureKind::EngineProcessing,
                    RunPhase::StartingEngine,
                    Some(prepared.source_dimensions),
                    None,
                    &format!(
                        "{} stopped before producing a valid image.",
                        snapshot.engine.display_name()
                    ),
                    &format!("Could not launch the engine: {error}"),
                    &owned_paths,
                    false,
                );
                let _ = journal.cleanup();
                return Ok(self.reject_failure(snapshot, run_id, failure));
            }
        };
        if let Err(error) = restore_result {
            let _ = process.kill_process_group();
            let _ = process.wait();
            let failure = self.make_failure(
                &snapshot,
                FailureKind::Internal,
                RunPhase::StartingEngine,
                Some(prepared.source_dimensions),
                None,
                "The upscale run couldn't be completed because the app encountered an internal problem.",
                &format!("Could not restore ownership descriptor flags: {error}"),
                &owned_paths,
                false,
            );
            let _ = journal.cleanup();
            return Ok(self.reject_failure(snapshot, run_id, failure));
        }
        let process_group_id = process.process_group_id();
        let cancelled_during_spawn = {
            let mut state = self.lock_state()?;
            if !state.set_started(run_id, process_group_id, owned_paths.clone()) {
                let _ = process.kill_process_group();
                let _ = process.wait();
                let _ = journal.cleanup();
                return Err("Run reservation was lost before engine launch".into());
            }
            state
                .active
                .as_ref()
                .is_some_and(|active| active.cancelled.load(Ordering::SeqCst))
        };
        if cancelled_during_spawn {
            let _ = process.kill_process_group();
        } else {
            self.progress(run_id, snapshot.engine, RunPhase::StartingEngine, None);
        }

        let service = self.clone();
        thread::spawn(move || service.monitor(process, journal, prepared, snapshot, temporary));
        Ok(StartOutcome::Started {
            run_id: run_id.to_string(),
            engine: invocation.engine,
        })
    }

    fn monitor(
        &self,
        mut process: Box<dyn engine::EngineProcess>,
        journal: OwnedJournal,
        prepared: Preflight,
        snapshot: RunSnapshot,
        temporary: PathBuf,
    ) {
        let run_id = prepared.run_id;
        let stdout = process.take_stdout();
        let stderr = process.take_stderr();
        let (stdout, stderr) = match (stdout, stderr) {
            (Ok(stdout), Ok(stderr)) => (stdout, stderr),
            (stdout, stderr) => {
                let _ = process.kill_process_group();
                let _ = process.wait();
                self.finish_failure(
                    snapshot,
                    prepared,
                    Some(journal),
                    FailureKind::Internal,
                    RunPhase::StartingEngine,
                    None,
                    "The upscale run couldn't be completed because the app encountered an internal problem.",
                    &format!("Could not capture engine logs: stdout={:?}, stderr={:?}", stdout.err(), stderr.err()),
                    vec![temporary],
                    false,
                );
                return;
            }
        };

        self.progress(run_id, snapshot.engine, RunPhase::Processing, None);
        let stdout_thread = thread::spawn(move || drain_stream(stdout, |_| {}));
        let progress_service = self.clone();
        let mut last_percent = None::<f32>;
        let stderr_thread = thread::spawn(move || {
            drain_stream(stderr, |line| {
                if let Some(percent) = accept_progress(&mut last_percent, line) {
                    progress_service.progress(
                        run_id,
                        snapshot.engine,
                        RunPhase::Processing,
                        Some(percent),
                    );
                }
            })
        });
        let status = process.wait();
        let stdout_tail = stdout_thread
            .join()
            .ok()
            .and_then(Result::ok)
            .map(|tail| tail.to_lossy_string())
            .unwrap_or_default();
        let stderr_tail = stderr_thread
            .join()
            .ok()
            .and_then(Result::ok)
            .map(|tail| tail.to_lossy_string())
            .unwrap_or_default();
        let details = format!("stdout:\n{stdout_tail}\nstderr:\n{stderr_tail}");

        if self.is_cancelled(run_id) {
            self.finish_cancelled(snapshot, Some(journal));
            return;
        }
        let status = match status {
            Ok(status) => status,
            Err(error) => {
                self.finish_failure(
                    snapshot,
                    prepared,
                    Some(journal),
                    FailureKind::Internal,
                    RunPhase::Processing,
                    None,
                    "The upscale run couldn't be completed because the app encountered an internal problem.",
                    &format!("Engine wait failed: {error}\n{details}"),
                    vec![temporary],
                    false,
                );
                return;
            }
        };
        if !status.success() {
            let kind = EngineAdapter::new(snapshot.engine).classify_diagnostics(
                &status,
                &stdout_tail,
                &stderr_tail,
            );
            let message = user_message(kind, snapshot.engine, snapshot.request.scale);
            self.finish_failure(
                snapshot,
                prepared,
                Some(journal),
                kind,
                RunPhase::Processing,
                status.code(),
                &message,
                &details,
                vec![temporary],
                false,
            );
            return;
        }

        self.progress(run_id, snapshot.engine, RunPhase::Validating, None);
        if let Err(error) = validate_output(
            &temporary,
            snapshot.request.format,
            prepared.output_dimensions,
        ) {
            let message = format!(
                "{} produced an image that did not pass validation.",
                snapshot.engine.display_name()
            );
            self.finish_failure(
                snapshot,
                prepared,
                Some(journal),
                FailureKind::OutputValidation,
                RunPhase::Validating,
                status.code(),
                &message,
                &format!("Validation failed: {error}\n{details}"),
                vec![temporary],
                false,
            );
            return;
        }
        if self.is_cancelled(run_id) {
            self.finish_cancelled(snapshot, Some(journal));
            return;
        }

        self.progress(run_id, snapshot.engine, RunPhase::Finalizing, None);
        let publication = {
            let mut state = match self.lock_state() {
                Ok(state) => state,
                Err(_) => {
                    let _ = journal.cleanup();
                    return;
                }
            };
            if !state.active.as_ref().is_some_and(|active| {
                active.run_id == run_id && !active.cancelled.load(Ordering::SeqCst)
            }) {
                drop(state);
                self.finish_cancelled(snapshot, Some(journal));
                return;
            }
            match publish_exclusive(
                &temporary,
                &prepared.source,
                &prepared.destination,
                snapshot.request.scale,
                snapshot.request.format,
            ) {
                Ok(output) => {
                    let event = RunEvent::Completed {
                        run_id: run_id.to_string(),
                        engine: snapshot.engine,
                        output_path: output.to_string_lossy().into_owned(),
                        was_cross_engine_retry: snapshot.cross_engine_retry_used,
                    };
                    let _ = journal.cleanup();
                    state.store_terminal(run_id, event.clone(), None);
                    Some(event)
                }
                Err(error) => {
                    drop(state);
                    let kind = if error.raw_os_error() == Some(libc::ENOSPC) {
                        FailureKind::DiskSpace
                    } else {
                        FailureKind::Destination
                    };
                    let message = user_message(kind, snapshot.engine, snapshot.request.scale);
                    self.finish_failure(
                        snapshot,
                        prepared,
                        Some(journal),
                        kind,
                        RunPhase::Finalizing,
                        status.code(),
                        &message,
                        &format!("Exclusive publication failed: {error}\n{details}"),
                        vec![temporary],
                        false,
                    );
                    None
                }
            }
        };
        if let Some(event) = publication {
            self.emit(event);
        }
    }

    pub fn cancel(&self, run_id: &str) -> Result<(), String> {
        let run_id =
            Uuid::parse_str(run_id).map_err(|_| "Run ID is not a valid UUID".to_string())?;
        let process_group_id = {
            let state = self.lock_state()?;
            let active = state
                .active
                .as_ref()
                .filter(|active| active.run_id == run_id)
                .ok_or_else(|| "That upscale run is already finished".to_string())?;
            active.cancelled.store(true, Ordering::SeqCst);
            active.process_group_id
        };
        if let Some(group_id) = process_group_id {
            kill_process_group(group_id, libc::SIGTERM)
                .map_err(|error| format!("Failed to stop upscaler: {error}"))?;
            let service = self.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(2));
                if service.is_cancelled(run_id) {
                    let _ = kill_process_group(group_id, libc::SIGKILL);
                }
            });
        }
        Ok(())
    }

    pub fn cancel_active(&self) {
        let active = self
            .lock_state()
            .ok()
            .and_then(|state| state.active.as_ref().map(|run| run.run_id.to_string()));
        if let Some(run_id) = active {
            let _ = self.cancel(&run_id);
        }
    }

    pub fn status(&self) -> Option<RunEvent> {
        self.lock_state()
            .ok()
            .and_then(|state| state.latest.clone())
    }

    pub fn engine_preference(&self) -> Result<UpscaleEngine, String> {
        let state = self.lock_state()?;
        state.preference.ok_or_else(|| {
            state
                .preference_error
                .clone()
                .unwrap_or_else(|| "Engine preference is unavailable".into())
        })
    }

    pub fn set_engine_preference(&self, engine: UpscaleEngine) -> Result<UpscaleEngine, String> {
        {
            let state = self.lock_state()?;
            if state.active.is_some() {
                return Err("Engine preference cannot change during an active run".into());
            }
        }
        persist_preference(&self.0.config_dir, engine)?;
        let mut state = self.lock_state()?;
        state.preference = Some(engine);
        state.preference_error = None;
        Ok(engine)
    }

    pub fn models(&self) -> Result<Vec<String>, String> {
        paired_models(&self.0.resources)
    }

    pub fn startup_status(&self) -> StartupStatus {
        self.0.startup_status.clone()
    }

    fn reject_preflight(
        &self,
        snapshot: RunSnapshot,
        run_id: Uuid,
        error: PreflightError,
    ) -> StartOutcome {
        let (kind, details, dimensions, safe_limit, message) = match error {
            PreflightError::Source(details) => (
                FailureKind::Source,
                details,
                None,
                false,
                "The source image could not be read.".to_string(),
            ),
            PreflightError::SafeLimit { message, dimensions } => (
                FailureKind::Source,
                message,
                dimensions,
                true,
                format!(
                    "This image would exceed the safe output size at {}×.",
                    snapshot.request.scale
                ),
            ),
            PreflightError::Destination(details) => (
                FailureKind::Destination,
                details,
                None,
                false,
                "Nggedhekaké Gambar can't write to this destination.".to_string(),
            ),
            PreflightError::MissingResource(details) => (
                FailureKind::MissingResource,
                details,
                None,
                false,
                "The app's bundled upscaling files are incomplete.".to_string(),
            ),
            PreflightError::Internal(details) => (
                FailureKind::Internal,
                details,
                None,
                false,
                "The upscale run couldn't be completed because the app encountered an internal problem.".to_string(),
            ),
        };
        let failure = self.make_failure(
            &snapshot,
            kind,
            RunPhase::Preflight,
            dimensions,
            None,
            &message,
            &details,
            &[],
            safe_limit,
        );
        self.reject_failure(snapshot, run_id, failure)
    }

    fn reject_failure(
        &self,
        snapshot: RunSnapshot,
        run_id: Uuid,
        failure: FailurePayload,
    ) -> StartOutcome {
        if let Ok(mut state) = self.lock_state() {
            state.store_preflight_failure(
                run_id,
                FailedRunRecord {
                    snapshot,
                    failure: failure.clone(),
                },
            );
        }
        self.emit(RunEvent::Failed {
            failure: failure.clone(),
        });
        StartOutcome::Rejected { failure }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_failure(
        &self,
        snapshot: RunSnapshot,
        prepared: Preflight,
        journal: Option<OwnedJournal>,
        kind: FailureKind,
        phase: RunPhase,
        exit_status: Option<i32>,
        message: &str,
        details: &str,
        mut sensitive: Vec<PathBuf>,
        safe_limit: bool,
    ) {
        let mut cleanup_error = None;
        if let Some(journal) = journal {
            if let Err(error) = journal.cleanup() {
                cleanup_error = Some(error.to_string());
            }
        }
        let details = match cleanup_error {
            Some(error) => format!("{details}\nCleanup also failed: {error}"),
            None => details.to_owned(),
        };
        sensitive.extend([
            prepared.source.clone(),
            prepared.destination.clone(),
            prepared.models.clone(),
            prepared.binary.clone(),
        ]);
        let failure = self.make_failure(
            &snapshot,
            kind,
            phase,
            Some(prepared.source_dimensions),
            exit_status,
            message,
            &details,
            &sensitive,
            safe_limit,
        );
        let event = RunEvent::Failed {
            failure: failure.clone(),
        };
        if let Ok(mut state) = self.lock_state() {
            if !state.store_terminal(
                prepared.run_id,
                event.clone(),
                Some(FailedRunRecord { snapshot, failure }),
            ) {
                return;
            }
        } else {
            return;
        }
        self.emit(event);
    }

    fn finish_cancelled(&self, snapshot: RunSnapshot, journal: Option<OwnedJournal>) {
        if let Some(journal) = journal {
            let _ = journal.cleanup();
        }
        let Ok(run_id) = Uuid::parse_str(&snapshot.request.run_id) else {
            return;
        };
        let event = RunEvent::Cancelled {
            run_id: run_id.to_string(),
            engine: snapshot.engine,
        };
        if let Ok(mut state) = self.lock_state() {
            if !state.store_terminal(run_id, event.clone(), None) {
                return;
            }
        } else {
            return;
        }
        self.emit(event);
    }

    #[allow(clippy::too_many_arguments)]
    fn make_failure(
        &self,
        snapshot: &RunSnapshot,
        kind: FailureKind,
        phase: RunPhase,
        dimensions: Option<[u32; 2]>,
        exit_status: Option<i32>,
        message: &str,
        details: &str,
        sensitive: &[PathBuf],
        safe_limit: bool,
    ) -> FailurePayload {
        FailurePayload {
            run_id: snapshot.request.run_id.clone(),
            kind,
            phase,
            user_message: message.into(),
            technical_details: diagnostics(
                &self.0.app_version,
                snapshot,
                kind,
                phase,
                dimensions,
                exit_status,
                message,
                details,
                sensitive,
            ),
            engine: (!matches!(phase, RunPhase::Preflight)).then_some(snapshot.engine),
            source_dimensions: dimensions,
            exit_status,
            actions: recovery_actions(
                kind,
                Some(snapshot.engine),
                snapshot.cross_engine_retry_used,
                safe_limit,
            ),
        }
    }

    fn progress(&self, run_id: Uuid, engine: UpscaleEngine, phase: RunPhase, percent: Option<f32>) {
        let event = RunEvent::Progress {
            run_id: run_id.to_string(),
            engine,
            phase,
            percent,
        };
        if let Ok(mut state) = self.lock_state() {
            if !state.set_phase(run_id, phase, event.clone()) {
                return;
            }
        } else {
            return;
        }
        self.emit(event);
    }

    fn is_cancelled(&self, run_id: Uuid) -> bool {
        self.lock_state()
            .ok()
            .and_then(|state| {
                state
                    .active
                    .as_ref()
                    .filter(|active| active.run_id == run_id)
                    .map(|active| active.cancelled.load(Ordering::SeqCst))
            })
            .unwrap_or(true)
    }

    fn emit(&self, event: RunEvent) {
        let _ = self.0.app.emit("upscale-run-event", event);
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, ServiceState>, String> {
        self.0
            .state
            .lock()
            .map_err(|_| "Upscale service state is unavailable".into())
    }
}

fn user_message(kind: FailureKind, engine: UpscaleEngine, scale: u32) -> String {
    match kind {
        FailureKind::Source => format!("This image would exceed the safe output size at {scale}×."),
        FailureKind::Destination => "Nggedhekaké Gambar can't write to this destination.".into(),
        FailureKind::DiskSpace => "This destination may not have enough free space.".into(),
        FailureKind::MissingResource => "The app's bundled upscaling files are incomplete.".into(),
        FailureKind::EngineIncompatibility => format!(
            "{} is not compatible with this Mac's current graphics setup.",
            engine.display_name()
        ),
        FailureKind::EngineProcessing => format!(
            "{} stopped before producing a valid image.",
            engine.display_name()
        ),
        FailureKind::OutputValidation => format!(
            "{} produced an image that did not pass validation.",
            engine.display_name()
        ),
        FailureKind::Internal => {
            "The upscale run couldn't be completed because the app encountered an internal problem."
                .into()
        }
    }
}
