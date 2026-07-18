use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpscaleEngine {
    Upscayl,
    RealEsrgan,
}

impl UpscaleEngine {
    pub const fn alternative(self) -> Self {
        match self {
            Self::Upscayl => Self::RealEsrgan,
            Self::RealEsrgan => Self::Upscayl,
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Upscayl => "Upscayl NCNN",
            Self::RealEsrgan => "Official Real-ESRGAN",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Png,
    Jpg,
    Webp,
}

impl OutputFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Webp => "webp",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartUpscaleRequest {
    pub run_id: String,
    pub image_path: String,
    pub output_folder: String,
    pub model: String,
    pub scale: u32,
    pub format: OutputFormat,
    pub acknowledge_low_disk_space: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryUpscaleRequest {
    pub run_id: String,
    pub failed_run_id: String,
    pub kind: RetryKind,
    pub acknowledge_low_disk_space: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RetryKind {
    SameEngine,
    AlternativeEngine,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpaceWarning {
    pub message: String,
    pub required_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum StartOutcome {
    Started {
        run_id: String,
        engine: UpscaleEngine,
    },
    NeedsDiskConfirmation {
        warning: DiskSpaceWarning,
    },
    Rejected {
        failure: FailurePayload,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RunPhase {
    Preflight,
    StartingEngine,
    Processing,
    Validating,
    Finalizing,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureKind {
    Source,
    Destination,
    DiskSpace,
    MissingResource,
    EngineIncompatibility,
    EngineProcessing,
    OutputValidation,
    Internal,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailurePayload {
    pub run_id: String,
    pub kind: FailureKind,
    pub phase: RunPhase,
    pub user_message: String,
    pub technical_details: String,
    pub engine: Option<UpscaleEngine>,
    pub source_dimensions: Option<[u32; 2]>,
    pub exit_status: Option<i32>,
    pub actions: Vec<RecoveryAction>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RecoveryAction {
    ChooseSource,
    ChooseDestination,
    LowerScale,
    RetrySameEngine,
    RetryAlternative { engine: UpscaleEngine },
    CopyTechnicalDetails,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RunEvent {
    Progress {
        run_id: String,
        engine: UpscaleEngine,
        phase: RunPhase,
        percent: Option<f32>,
    },
    Completed {
        run_id: String,
        engine: UpscaleEngine,
        output_path: String,
        was_cross_engine_retry: bool,
    },
    Cancelled {
        run_id: String,
        engine: UpscaleEngine,
    },
    Failed {
        failure: FailurePayload,
    },
}

impl RunEvent {
    pub fn run_id(&self) -> &str {
        match self {
            Self::Progress { run_id, .. }
            | Self::Completed { run_id, .. }
            | Self::Cancelled { run_id, .. } => run_id,
            Self::Failed { failure } => &failure.run_id,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    pub cleanup_failures: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};

    fn round_trip<T>(value: &T) -> Value
    where
        T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug,
    {
        let serialized = serde_json::to_value(value).unwrap();
        let decoded = serde_json::from_value(serialized.clone()).unwrap();
        assert_eq!(*value, decoded);
        serialized
    }

    #[test]
    fn engine_format_and_retry_spellings_are_stable() {
        assert_eq!(round_trip(&UpscaleEngine::Upscayl), json!("upscayl"));
        assert_eq!(round_trip(&UpscaleEngine::RealEsrgan), json!("real-esrgan"));
        assert_eq!(round_trip(&OutputFormat::Png), json!("png"));
        assert_eq!(round_trip(&OutputFormat::Jpg), json!("jpg"));
        assert_eq!(round_trip(&OutputFormat::Webp), json!("webp"));
        assert_eq!(round_trip(&RetryKind::SameEngine), json!("sameEngine"));
        assert_eq!(
            round_trip(&RetryKind::AlternativeEngine),
            json!("alternativeEngine")
        );

        for unknown in ["unknown", "official", "jpeg"] {
            assert!(serde_json::from_value::<UpscaleEngine>(json!(unknown)).is_err());
            assert!(serde_json::from_value::<OutputFormat>(json!(unknown)).is_err());
            assert!(serde_json::from_value::<RetryKind>(json!(unknown)).is_err());
        }
    }

    #[test]
    fn wire_fields_and_event_tags_are_stable() {
        let failure = FailurePayload {
            run_id: "run-1".into(),
            kind: FailureKind::OutputValidation,
            phase: RunPhase::Validating,
            user_message: "invalid".into(),
            technical_details: "details".into(),
            engine: Some(UpscaleEngine::Upscayl),
            source_dimensions: Some([8, 9]),
            exit_status: Some(2),
            actions: vec![RecoveryAction::RetryAlternative {
                engine: UpscaleEngine::RealEsrgan,
            }],
        };
        let value = round_trip(&RunEvent::Failed { failure });
        assert_eq!(value["type"], "failed");
        assert_eq!(value["failure"]["runId"], "run-1");
        assert_eq!(value["failure"]["technicalDetails"], "details");
        assert_eq!(value["failure"]["sourceDimensions"], json!([8, 9]));
        assert_eq!(value["failure"]["actions"][0]["type"], "retryAlternative");

        let completed = round_trip(&RunEvent::Completed {
            run_id: "run-2".into(),
            engine: UpscaleEngine::RealEsrgan,
            output_path: "/output.png".into(),
            was_cross_engine_retry: true,
        });
        assert_eq!(completed["outputPath"], "/output.png");
        assert_eq!(completed["wasCrossEngineRetry"], true);

        let warning = round_trip(&DiskSpaceWarning {
            message: "low".into(),
            required_bytes: 10,
            available_bytes: 5,
        });
        assert_eq!(warning["requiredBytes"], 10);
        assert_eq!(warning["availableBytes"], 5);

        let retry = round_trip(&RetryUpscaleRequest {
            run_id: "new".into(),
            failed_run_id: "old".into(),
            kind: RetryKind::SameEngine,
            acknowledge_low_disk_space: false,
        });
        assert_eq!(retry["runId"], "new");
        assert_eq!(retry["failedRunId"], "old");
    }
}
