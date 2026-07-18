use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};
use upscale_contract::{
    FailureKind, FailurePayload, RecoveryAction, RunEvent, RunPhase, StartUpscaleRequest,
    UpscaleEngine,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RunSnapshot {
    pub request: StartUpscaleRequest,
    pub engine: UpscaleEngine,
    pub cross_engine_retry_used: bool,
}

#[derive(Clone, Debug)]
pub struct FailedRunRecord {
    pub snapshot: RunSnapshot,
    pub failure: FailurePayload,
}

#[derive(Clone, Debug)]
pub struct ActiveRun {
    pub run_id: Uuid,
    pub snapshot: RunSnapshot,
    pub process_group_id: Option<i32>,
    pub phase: RunPhase,
    pub cancelled: Arc<AtomicBool>,
    pub owned_paths: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct ServiceState {
    pub active: Option<ActiveRun>,
    pub failed: Option<FailedRunRecord>,
    pub latest: Option<RunEvent>,
    pub preference: Option<UpscaleEngine>,
    pub preference_error: Option<String>,
}

impl ServiceState {
    pub fn new(preference: Result<UpscaleEngine, String>) -> Self {
        match preference {
            Ok(engine) => Self {
                active: None,
                failed: None,
                latest: None,
                preference: Some(engine),
                preference_error: None,
            },
            Err(error) => Self {
                active: None,
                failed: None,
                latest: None,
                preference: None,
                preference_error: Some(error),
            },
        }
    }

    pub fn reserve(&mut self, snapshot: RunSnapshot) -> Result<Arc<AtomicBool>, String> {
        if self.active.is_some() {
            return Err("An upscale is already running".into());
        }
        let run_id = Uuid::parse_str(&snapshot.request.run_id)
            .map_err(|_| "Run ID is not a valid UUID".to_string())?;
        let cancelled = Arc::new(AtomicBool::new(false));
        self.active = Some(ActiveRun {
            run_id,
            snapshot,
            process_group_id: None,
            phase: RunPhase::Preflight,
            cancelled: Arc::clone(&cancelled),
            owned_paths: Vec::new(),
        });
        Ok(cancelled)
    }

    pub fn set_started(
        &mut self,
        run_id: Uuid,
        process_group_id: i32,
        owned_paths: Vec<PathBuf>,
    ) -> bool {
        let Some(active) = self
            .active
            .as_mut()
            .filter(|active| active.run_id == run_id)
        else {
            return false;
        };
        debug_assert_eq!(active.snapshot.request.run_id, run_id.to_string());
        active.process_group_id = Some(process_group_id);
        active.phase = RunPhase::StartingEngine;
        active.owned_paths = owned_paths;
        true
    }

    pub fn set_phase(&mut self, run_id: Uuid, phase: RunPhase, event: RunEvent) -> bool {
        let Some(active) = self
            .active
            .as_mut()
            .filter(|active| active.run_id == run_id)
        else {
            return false;
        };
        active.phase = phase;
        self.latest = Some(event);
        true
    }

    pub fn clear_reservation(&mut self, run_id: Uuid) {
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.run_id == run_id)
        {
            self.active = None;
        }
    }

    pub fn store_terminal(
        &mut self,
        run_id: Uuid,
        event: RunEvent,
        failure: Option<FailedRunRecord>,
    ) -> bool {
        if !self
            .active
            .as_ref()
            .is_some_and(|active| active.run_id == run_id)
        {
            return false;
        }
        self.active = None;
        self.failed = failure;
        self.latest = Some(event);
        true
    }

    pub fn store_preflight_failure(&mut self, run_id: Uuid, record: FailedRunRecord) {
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.run_id == run_id)
        {
            self.active = None;
            self.latest = Some(RunEvent::Failed {
                failure: record.failure.clone(),
            });
            self.failed = Some(record);
        }
    }
}

pub fn recovery_actions(
    kind: FailureKind,
    engine: Option<UpscaleEngine>,
    cross_engine_retry_used: bool,
    safe_limit: bool,
) -> Vec<RecoveryAction> {
    match kind {
        FailureKind::EngineIncompatibility => {
            let mut actions = Vec::new();
            if !cross_engine_retry_used {
                if let Some(engine) = engine {
                    actions.push(RecoveryAction::RetryAlternative {
                        engine: engine.alternative(),
                    });
                }
            }
            actions.push(RecoveryAction::CopyTechnicalDetails);
            actions
        }
        FailureKind::EngineProcessing | FailureKind::OutputValidation => {
            let mut actions = vec![RecoveryAction::RetrySameEngine];
            if !cross_engine_retry_used {
                if let Some(engine) = engine {
                    actions.push(RecoveryAction::RetryAlternative {
                        engine: engine.alternative(),
                    });
                }
            }
            actions.push(RecoveryAction::CopyTechnicalDetails);
            actions
        }
        FailureKind::Source if safe_limit => vec![
            RecoveryAction::LowerScale,
            RecoveryAction::ChooseSource,
            RecoveryAction::CopyTechnicalDetails,
        ],
        FailureKind::Source => vec![
            RecoveryAction::ChooseSource,
            RecoveryAction::CopyTechnicalDetails,
        ],
        FailureKind::Destination | FailureKind::DiskSpace => vec![
            RecoveryAction::ChooseDestination,
            RecoveryAction::CopyTechnicalDetails,
        ],
        FailureKind::MissingResource | FailureKind::Internal => {
            vec![RecoveryAction::CopyTechnicalDetails]
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreferenceFile {
    primary_engine: UpscaleEngine,
}

pub fn load_preference(config_dir: &Path) -> Result<UpscaleEngine, String> {
    let path = config_dir.join("engine-preference.json");
    match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<PreferenceFile>(&bytes)
            .map(|file| file.primary_engine)
            .map_err(|error| format!("Engine preference is malformed: {error}")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(UpscaleEngine::Upscayl),
        Err(error) => Err(format!("Engine preference could not be read: {error}")),
    }
}

pub fn persist_preference(config_dir: &Path, engine: UpscaleEngine) -> Result<(), String> {
    fs::create_dir_all(config_dir).map_err(|error| error.to_string())?;
    let final_path = config_dir.join("engine-preference.json");
    let temporary = config_dir.join(format!(".engine-preference.{}.tmp", Uuid::new_v4()));
    let result = (|| -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        serde_json::to_writer(
            &mut file,
            &PreferenceFile {
                primary_engine: engine,
            },
        )
        .map_err(io::Error::other)?;
        file.flush()?;
        file.sync_all()?;
        fs::rename(&temporary, &final_path)?;
        File::open(config_dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| format!("Engine preference could not be saved: {error}"))
}

pub fn diagnostics(
    app_version: &str,
    snapshot: &RunSnapshot,
    kind: FailureKind,
    phase: RunPhase,
    dimensions: Option<[u32; 2]>,
    exit_status: Option<i32>,
    message: &str,
    details: &str,
    sensitive_paths: &[PathBuf],
) -> String {
    let mut sanitized = details.to_owned();
    for (path, replacement) in sensitive_paths.iter().zip(
        [
            "<source>",
            "<destination>",
            "<temporary-output>",
            "<companion-output>",
            "<models>",
        ]
        .into_iter()
        .cycle(),
    ) {
        let full = path.to_string_lossy();
        if !full.is_empty() {
            sanitized = sanitized.replace(full.as_ref(), replacement);
        }
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            if !name.is_empty() {
                sanitized = sanitized.replace(name, replacement);
            }
        }
    }
    let dimensions = dimensions
        .map(|[width, height]| format!("{width}×{height}"))
        .unwrap_or_else(|| "Unknown".into());
    let engine = snapshot.engine.display_name();
    let exit = exit_status
        .map(|status| status.to_string())
        .unwrap_or_else(|| "Not available".into());
    format!(
        "Nggedhekaké Gambar {app_version}\nRun ID: {}\nEngine: {engine}\nFailure: {kind:?}\nPhase: {phase:?}\nModel: {}\nScale: {}×\nFormat: {}\nSource dimensions: {dimensions}\nMessage: {message}\n\nTechnical details:\nOS: {}\nARCH: {}\nExit status: {exit}\n{sanitized}",
        snapshot.request.run_id,
        snapshot.request.model,
        snapshot.request.scale,
        snapshot.request.format.extension().to_uppercase(),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use upscale_contract::OutputFormat;

    fn snapshot(engine: UpscaleEngine, used: bool) -> RunSnapshot {
        RunSnapshot {
            request: StartUpscaleRequest {
                run_id: Uuid::new_v4().to_string(),
                image_path: "/private/photo.png".into(),
                output_folder: "/private/output".into(),
                model: "model".into(),
                scale: 2,
                format: OutputFormat::Png,
                acknowledge_low_disk_space: false,
            },
            engine,
            cross_engine_retry_used: used,
        }
    }

    fn failure(run_id: String) -> FailurePayload {
        FailurePayload {
            run_id,
            kind: FailureKind::EngineProcessing,
            phase: RunPhase::Processing,
            user_message: "failed".into(),
            technical_details: "details".into(),
            engine: Some(UpscaleEngine::Upscayl),
            source_dimensions: Some([10, 10]),
            exit_status: Some(1),
            actions: recovery_actions(
                FailureKind::EngineProcessing,
                Some(UpscaleEngine::Upscayl),
                false,
                false,
            ),
        }
    }

    #[test]
    fn recovery_matrix_is_ordered_and_prevents_ping_pong() {
        assert_eq!(
            recovery_actions(
                FailureKind::EngineProcessing,
                Some(UpscaleEngine::Upscayl),
                false,
                false
            ),
            vec![
                RecoveryAction::RetrySameEngine,
                RecoveryAction::RetryAlternative {
                    engine: UpscaleEngine::RealEsrgan
                },
                RecoveryAction::CopyTechnicalDetails,
            ]
        );
        assert_eq!(
            recovery_actions(
                FailureKind::EngineIncompatibility,
                Some(UpscaleEngine::RealEsrgan),
                true,
                false
            ),
            vec![RecoveryAction::CopyTechnicalDetails]
        );
        assert_eq!(
            recovery_actions(
                FailureKind::DiskSpace,
                Some(UpscaleEngine::Upscayl),
                false,
                false
            ),
            vec![
                RecoveryAction::ChooseDestination,
                RecoveryAction::CopyTechnicalDetails
            ]
        );
    }

    #[test]
    fn active_slot_is_reserved_before_a_second_start() {
        let mut state = ServiceState::new(Ok(UpscaleEngine::Upscayl));
        let first = snapshot(UpscaleEngine::Upscayl, false);
        let first_id = Uuid::parse_str(&first.request.run_id).unwrap();
        state.reserve(first.clone()).unwrap();
        assert!(state
            .reserve(snapshot(UpscaleEngine::Upscayl, false))
            .is_err());
        assert_eq!(state.active.as_ref().unwrap().run_id, first_id);
        assert_eq!(
            state.active.as_ref().unwrap().snapshot.request,
            first.request
        );
    }

    #[test]
    fn failed_record_preserves_the_exact_retry_snapshot() {
        let mut state = ServiceState::new(Ok(UpscaleEngine::Upscayl));
        let snapshot = snapshot(UpscaleEngine::Upscayl, false);
        let run_id = Uuid::parse_str(&snapshot.request.run_id).unwrap();
        state.reserve(snapshot.clone()).unwrap();
        let failure = failure(snapshot.request.run_id.clone());
        state.store_preflight_failure(
            run_id,
            FailedRunRecord {
                snapshot: snapshot.clone(),
                failure,
            },
        );
        assert_eq!(
            state.failed.as_ref().unwrap().snapshot.request,
            snapshot.request
        );
        assert_eq!(
            state.failed.as_ref().unwrap().snapshot.engine,
            UpscaleEngine::Upscayl
        );
        assert!(
            !state
                .failed
                .as_ref()
                .unwrap()
                .snapshot
                .cross_engine_retry_used
        );
    }

    #[test]
    fn preference_defaults_only_when_missing_and_persists() {
        let dir = tempdir().unwrap();
        assert_eq!(load_preference(dir.path()).unwrap(), UpscaleEngine::Upscayl);
        persist_preference(dir.path(), UpscaleEngine::RealEsrgan).unwrap();
        assert_eq!(
            load_preference(dir.path()).unwrap(),
            UpscaleEngine::RealEsrgan
        );
        fs::write(dir.path().join("engine-preference.json"), b"bad").unwrap();
        assert!(load_preference(dir.path()).is_err());
        fs::remove_file(dir.path().join("engine-preference.json")).unwrap();
        fs::create_dir(dir.path().join("engine-preference.json")).unwrap();
        assert!(load_preference(dir.path()).is_err());
    }

    #[test]
    fn diagnostics_redact_paths_and_basenames() {
        let snapshot = snapshot(UpscaleEngine::Upscayl, false);
        let output = PathBuf::from("/private/output/.nggedhekake-gambar.id.tmp.png");
        let details = format!("failed reading {} and photo.png", output.display());
        let text = diagnostics(
            "0.1.0",
            &snapshot,
            FailureKind::OutputValidation,
            RunPhase::Validating,
            Some([10, 10]),
            Some(1),
            "invalid",
            &details,
            &[PathBuf::from(&snapshot.request.image_path), output],
        );
        assert!(!text.contains("/private"));
        assert!(!text.contains("photo.png"));
        assert!(text.contains("Engine: Upscayl NCNN"));
        assert!(text.contains("Source dimensions: 10×10"));
    }
}
