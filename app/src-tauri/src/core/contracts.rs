use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub duration: f64,
    pub segments: Vec<TranscriptSegment>,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
    pub score: i64,
    pub hook_sentence: String,
    pub virality_reason: String,
    #[serde(flatten)]
    pub provider_payload: std::collections::BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortClip {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
    pub score: i64,
    pub hook_sentence: String,
    pub virality_reason: String,
    pub clip_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(flatten)]
    pub provider_payload: std::collections::BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSuccess {
    pub mode: String,
    pub source_video_url: String,
    pub transcript: Transcript,
    pub highlights: Vec<Highlight>,
    pub shorts: Vec<ShortClip>,
    #[serde(flatten)]
    pub provider_payload: std::collections::BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub mode: Option<String>,
    pub source_video_url: Option<String>,
    pub error: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub stage: String,
    pub progress: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_video_url: Option<String>,
    #[serde(flatten)]
    pub provider_payload: std::collections::BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::{ErrorEnvelope, PipelineSuccess, ProgressEvent};

    const SUCCESS: &str = include_str!("../../../../tests/fixtures/contracts/success.json");
    const PARTIAL_FAILURE: &str =
        include_str!("../../../../tests/fixtures/contracts/partial_failure.json");
    const HARD_FAILURE: &str =
        include_str!("../../../../tests/fixtures/contracts/hard_failure.json");

    #[test]
    fn deserialize_success_fixture() {
        let parsed: PipelineSuccess =
            serde_json::from_str(SUCCESS).expect("success fixture should deserialize");
        assert!(parsed.source_video_url.len() > 1);
        assert!(!parsed.transcript.segments.is_empty());
        assert!(!parsed.highlights.is_empty());
        assert!(!parsed.shorts.is_empty());
    }

    #[test]
    fn deserialize_partial_failure_fixture() {
        let parsed: PipelineSuccess = serde_json::from_str(PARTIAL_FAILURE)
            .expect("partial failure fixture should deserialize");
        assert!(parsed.shorts.iter().any(|s| s.clip_url.is_none()));
        assert!(parsed.shorts.iter().any(|s| s.error.is_some()));
    }

    #[test]
    fn deserialize_hard_failure_fixture() {
        let parsed: ErrorEnvelope =
            serde_json::from_str(HARD_FAILURE).expect("hard failure should deserialize");
        assert!(!parsed.error.is_empty());
    }

    #[test]
    fn deserialize_progress_event_shape() {
        let raw = r#"{"event":"progress","stage":"transcribe","progress":0.5,"message":"halfway"}"#;
        let parsed: ProgressEvent =
            serde_json::from_str(raw).expect("progress event should deserialize");
        assert_eq!(parsed.stage, "transcribe");
    }
}
