use crate::core::contracts::{ErrorEnvelope, PipelineSuccess, ProgressEvent};
use crate::core::observability::events::ProgressEmitter;
use crate::core::pipeline::{
    generate_shorts_with, generate_shorts_with_progress_live, GenerateShortsRequest,
    MockPipelineStages,
};
use crate::runtime::fs_output::write_result_json_atomic;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tauri::Emitter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenerateShortsCommand {
    pub youtube_url: String,
    #[serde(default = "default_num_clips")]
    pub num_clips: usize,
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,
    #[serde(default = "default_download_format")]
    pub download_format: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub output_json: Option<String>,
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_num_clips() -> usize {
    3
}
fn default_aspect_ratio() -> String {
    "9:16".to_string()
}
fn default_download_format() -> String {
    "720".to_string()
}
fn default_mode() -> String {
    "api".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateCommandArgs {
    pub request: GenerateShortsCommand,
    #[serde(default)]
    pub test_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GenerateEnvelope {
    Success { ok: bool, result: PipelineSuccess },
    Failure { ok: bool, error: ErrorEnvelope },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateWithEventsResponse {
    pub envelope: GenerateEnvelope,
    pub events: Vec<ProgressEvent>,
}

struct TauriMuapiProgressEmitter {
    app_handle: tauri::AppHandle,
}

impl ProgressEmitter for TauriMuapiProgressEmitter {
    fn emit_status_change(&self, label: &str, status: &str) {
        let event = ProgressEvent {
            event: "progress".to_string(),
            stage: format!("muapi_poll:{status}"),
            progress: 0.4,
            message: Some(format!("[muapi] {label}: {status}")),
            mode: Some("api".to_string()),
            source_video_url: None,
            provider_payload: Default::default(),
        };
        let _ = self.app_handle.emit("generate-progress", event);
    }
}

fn push_progress(
    events: &mut Vec<ProgressEvent>,
    stage: &str,
    progress: f64,
    message: Option<String>,
    sink: &mut Option<&mut dyn FnMut(&ProgressEvent)>,
) {
    let event = ProgressEvent {
        event: "progress".to_string(),
        stage: stage.to_string(),
        progress,
        message,
        mode: None,
        source_video_url: None,
        provider_payload: Default::default(),
    };
    if let Some(cb) = sink.as_deref_mut() {
        cb(&event);
    }
    events.push(event);
}

fn as_request(c: &GenerateShortsCommand) -> GenerateShortsRequest {
    GenerateShortsRequest {
        youtube_url: c.youtube_url.clone(),
        num_clips: c.num_clips,
        aspect_ratio: c.aspect_ratio.clone(),
        download_format: c.download_format.clone(),
        language: c.language.clone(),
        mode: c.mode.clone(),
    }
}

fn run_mock(
    events: &mut Vec<ProgressEvent>,
    req: &GenerateShortsRequest,
    mode: &str,
    sink: &mut Option<&mut dyn FnMut(&ProgressEvent)>,
) -> Result<PipelineSuccess, ErrorEnvelope> {
    let mut stages = match mode {
        "fail_transcribe" => MockPipelineStages::default()
            .with_download(Ok("https://cdn.example.com/video.mp4".to_string()))
            .with_transcript(Err("transcribe failed".to_string())),
        _ => MockPipelineStages::default()
            .with_download(Ok("https://cdn.example.com/video.mp4".to_string()))
            .with_transcript(Ok(json!({
                "duration": 12.0,
                "segments": [{"start": 0.0, "end": 6.0, "text": "hello"}]
            })))
            .with_highlights(Ok(json!({
                "highlights": [{
                    "title": "hello",
                    "start_time": 0.0,
                    "end_time": 6.0,
                    "score": 90,
                    "hook_sentence": "hello",
                    "virality_reason": "test"
                }]
            })))
            .with_crop(Ok(vec![json!({
                "title": "hello",
                "start_time": 0.0,
                "end_time": 6.0,
                "score": 90,
                "hook_sentence": "hello",
                "virality_reason": "test",
                "clip_url": "https://cdn.example.com/short_01.mp4"
            })])),
    };

    push_progress(events, "download:start", 0.1, None, sink);
    push_progress(events, "download:end", 0.25, None, sink);
    push_progress(events, "transcribe:start", 0.3, None, sink);
    if mode == "success_with_status" {
        push_progress(
            events,
            "muapi_poll:queued",
            0.35,
            Some("[muapi] transcribe: queued".to_string()),
            sink,
        );
        push_progress(
            events,
            "muapi_poll:completed",
            0.4,
            Some("[muapi] transcribe: completed".to_string()),
            sink,
        );
    }

    let out = generate_shorts_with(req, &mut stages);
    if out.is_ok() {
        push_progress(events, "transcribe:end", 0.55, None, sink);
        push_progress(events, "highlights:start", 0.6, None, sink);
        push_progress(events, "highlights:end", 0.75, None, sink);
        push_progress(events, "clip:start", 0.8, None, sink);
        push_progress(events, "clip:end", 1.0, None, sink);
    }
    out
}

#[tauri::command]
pub fn generate_shorts(args: GenerateCommandArgs) -> Result<GenerateEnvelope, String> {
    Ok(run_generate(args).0)
}

#[tauri::command]
pub fn generate_shorts_with_events(
    args: GenerateCommandArgs,
) -> Result<GenerateWithEventsResponse, String> {
    let (envelope, events) = run_generate(args);
    Ok(GenerateWithEventsResponse { envelope, events })
}

#[tauri::command]
pub fn generate_shorts_stream(
    app_handle: tauri::AppHandle,
    args: GenerateCommandArgs,
) -> Result<GenerateEnvelope, String> {
    let sink_handle = app_handle.clone();
    let mut sink = |event: &ProgressEvent| {
        let _ = sink_handle.emit("generate-progress", event);
    };
    Ok(run_generate_with_sink(args, Some(&mut sink), Some(app_handle)).0)
}

pub fn run_generate(args: GenerateCommandArgs) -> (GenerateEnvelope, Vec<ProgressEvent>) {
    run_generate_with_sink(args, None, None)
}

pub fn run_generate_with_sink(
    args: GenerateCommandArgs,
    mut sink: Option<&mut dyn FnMut(&ProgressEvent)>,
    app_handle: Option<tauri::AppHandle>,
) -> (GenerateEnvelope, Vec<ProgressEvent>) {
    let req = as_request(&args.request);
    let output_json = args.request.output_json.clone();
    let mut events = Vec::new();

    let out = if let Some(test_mode) = args.test_mode.as_deref() {
        run_mock(&mut events, &req, test_mode, &mut sink)
    } else {
        push_progress(
            &mut events,
            "generate:start",
            0.0,
            Some("pipeline started".to_string()),
            &mut sink,
        );
        let result = {
            let mut emit_stage = |stage: &str, progress: f64, message: Option<String>| {
                push_progress(&mut events, stage, progress, message, &mut sink);
            };
            let muapi_emitter = app_handle
                .as_ref()
                .map(|h| Arc::new(TauriMuapiProgressEmitter { app_handle: h.clone() }) as Arc<dyn ProgressEmitter>);
            generate_shorts_with_progress_live(&req, Some(&mut emit_stage), muapi_emitter)
        };
        if result.is_ok() {
            push_progress(
                &mut events,
                "generate:end",
                1.0,
                Some("pipeline completed".to_string()),
                &mut sink,
            );
        }
        result
    };

    let envelope = match out {
        Ok(result) => {
            if let Some(path) = output_json.as_deref() {
                let payload = serde_json::to_value(&result).unwrap_or_default();
                if let Err(e) = write_result_json_atomic(Path::new(path), &payload) {
                    return (
                        GenerateEnvelope::Failure {
                            ok: false,
                            error: ErrorEnvelope {
                                mode: Some(req.mode.clone()),
                                source_video_url: Some(req.youtube_url.clone()),
                                error: format!("failed to write output json: {e}"),
                                details: Some(json!({
                                    "stage": "output_json",
                                    "code": "E_OUTPUT_JSON_WRITE_FAILED"
                                })),
                            },
                        },
                        events,
                    );
                }
            }
            GenerateEnvelope::Success { ok: true, result }
        }
        Err(error) => GenerateEnvelope::Failure { ok: false, error },
    };

    (envelope, events)
}
