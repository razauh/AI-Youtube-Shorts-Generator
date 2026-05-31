use crate::core::contracts::{ErrorEnvelope, PipelineSuccess, ProgressEvent};
use crate::core::observability::events::ProgressEmitter;
use crate::core::pipeline::{
    generate_shorts_with, generate_shorts_with_progress_live, GenerateShortsRequest,
    MockPipelineStages,
};
use crate::runtime::fs_output::write_result_json_atomic;
use license_control_suite::core::SessionState;
use license_control_suite::desktop::tauri::AuthAppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::Emitter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenerateShortsCommand {
    #[serde(default)]
    pub run_id: Option<String>,
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
fn default_language() -> String {
    "English".to_string()
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelGenerateArgs {
    pub run_id: String,
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
    run_id: String,
}

impl ProgressEmitter for TauriMuapiProgressEmitter {
    fn emit_status_change(&self, label: &str, status: &str) {
        let event = ProgressEvent {
            event: "progress".to_string(),
            run_id: Some(self.run_id.clone()),
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

fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn active_runs() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static RUNS: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    RUNS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn normalize_run_id(value: Option<&str>) -> String {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("run-{}-{}", now_epoch_ms(), std::process::id()))
}

fn register_run(run_id: &str) -> Arc<AtomicBool> {
    let mut runs = active_runs().lock().unwrap_or_else(|e| e.into_inner());
    let flag = Arc::new(AtomicBool::new(false));
    runs.insert(run_id.to_string(), flag.clone());
    flag
}

fn unregister_run(run_id: &str) {
    let mut runs = active_runs().lock().unwrap_or_else(|e| e.into_inner());
    runs.remove(run_id);
}

fn cancel_run(run_id: &str) -> bool {
    let runs = active_runs().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(flag) = runs.get(run_id) {
        flag.store(true, Ordering::Relaxed);
        return true;
    }
    false
}

fn cancelled_envelope(args: &GenerateCommandArgs, run_id: &str) -> GenerateEnvelope {
    GenerateEnvelope::Failure {
        ok: false,
        error: ErrorEnvelope {
            mode: Some(args.request.mode.clone()),
            source_video_url: Some(args.request.youtube_url.clone()),
            error: "Generation cancelled.".to_string(),
            details: Some(json!({"stage":"cancel","code":"E_GENERATION_CANCELLED","run_id":run_id})),
        },
    }
}

fn timeout_envelope(args: &GenerateCommandArgs, run_id: &str) -> GenerateEnvelope {
    GenerateEnvelope::Failure {
        ok: false,
        error: ErrorEnvelope {
            mode: Some(args.request.mode.clone()),
            source_video_url: Some(args.request.youtube_url.clone()),
            error: "Generation timed out.".to_string(),
            details: Some(json!({"stage":"timeout","code":"E_GENERATION_TIMEOUT","run_id":run_id})),
        },
    }
}

fn push_progress(
    events: &mut Vec<ProgressEvent>,
    run_id: &str,
    stage: &str,
    progress: f64,
    message: Option<String>,
    sink: &mut Option<&mut dyn FnMut(&ProgressEvent)>,
) {
    let event = ProgressEvent {
        event: "progress".to_string(),
        run_id: Some(run_id.to_string()),
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

fn push_terminal(
    events: &mut Vec<ProgressEvent>,
    run_id: &str,
    stage: &str,
    message: Option<String>,
    sink: &mut Option<&mut dyn FnMut(&ProgressEvent)>,
) {
    let event = ProgressEvent {
        event: "terminal".to_string(),
        run_id: Some(run_id.to_string()),
        stage: stage.to_string(),
        progress: 1.0,
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
        language: c
            .language
            .clone()
            .or_else(|| Some(default_language()))
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()),
        mode: c.mode.clone(),
    }
}

fn run_mock(
    events: &mut Vec<ProgressEvent>,
    run_id: &str,
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

    push_progress(events, run_id, "download:start", 0.1, None, sink);
    push_progress(events, run_id, "download:end", 0.25, None, sink);
    push_progress(events, run_id, "transcribe:start", 0.3, None, sink);
    if mode == "success_with_status" {
        push_progress(
            events,
            run_id,
            "muapi_poll:queued",
            0.35,
            Some("[muapi] transcribe: queued".to_string()),
            sink,
        );
        push_progress(
            events,
            run_id,
            "muapi_poll:completed",
            0.4,
            Some("[muapi] transcribe: completed".to_string()),
            sink,
        );
    }

    let out = generate_shorts_with(req, &mut stages);
    if out.is_ok() {
        push_progress(events, run_id, "transcribe:end", 0.55, None, sink);
        push_progress(events, run_id, "highlights:start", 0.6, None, sink);
        push_progress(events, run_id, "highlights:end", 0.75, None, sink);
        push_progress(events, run_id, "clip:start", 0.8, None, sink);
        push_progress(events, run_id, "clip:end", 1.0, None, sink);
    }
    out
}

#[tauri::command]
pub async fn generate_shorts(
    args: GenerateCommandArgs,
    auth_state: tauri::State<'_, AuthAppState>,
) -> Result<GenerateEnvelope, String> {
    Ok(run_generate_authorized(args, &auth_state).await.0)
}

#[tauri::command]
pub async fn generate_shorts_with_events(
    args: GenerateCommandArgs,
    auth_state: tauri::State<'_, AuthAppState>,
) -> Result<GenerateWithEventsResponse, String> {
    let (envelope, events) = run_generate_authorized(args, &auth_state).await;
    Ok(GenerateWithEventsResponse { envelope, events })
}

#[tauri::command]
pub async fn generate_shorts_stream(
    app_handle: tauri::AppHandle,
    args: GenerateCommandArgs,
    auth_state: tauri::State<'_, AuthAppState>,
) -> Result<GenerateEnvelope, String> {
    if let Some(error) = generation_auth_error(&args, &auth_state).await {
        return Ok(GenerateEnvelope::Failure { ok: false, error });
    }
    let run_id = normalize_run_id(args.request.run_id.as_deref());
    let cancelled = register_run(&run_id);
    let timeout_secs = std::env::var("GENERATE_RUN_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(1800);
    let timed_out = Arc::new(AtomicBool::new(false));
    let timer_cancelled = cancelled.clone();
    let timer_timed_out = timed_out.clone();
    if timeout_secs > 0 {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(timeout_secs));
            timer_timed_out.store(true, Ordering::Relaxed);
            timer_cancelled.store(true, Ordering::Relaxed);
        });
    }

    let _ = app_handle.emit(
        "generate-progress",
        ProgressEvent {
            event: "progress".to_string(),
            run_id: Some(run_id.clone()),
            stage: "generate:start".to_string(),
            progress: 0.0,
            message: Some("pipeline started".to_string()),
            mode: Some(args.request.mode.clone()),
            source_video_url: Some(args.request.youtube_url.clone()),
            provider_payload: Default::default(),
        },
    );

    let started_at = Instant::now();
    let app_for_sink = app_handle.clone();
    let cancelled_for_sink = cancelled.clone();
    let run_id_for_sink = run_id.clone();
    let mut sink = move |event: &ProgressEvent| {
        if event.run_id.as_deref() != Some(run_id_for_sink.as_str()) {
            return;
        }
        if cancelled_for_sink.load(Ordering::Relaxed) && event.event != "terminal" {
            return;
        }
        let _ = app_for_sink.emit("generate-progress", event);
    };

    let mut result = run_generate_with_sink(
        args.clone(),
        Some(&mut sink),
        Some(app_handle.clone()),
        &run_id,
        cancelled.clone(),
    )
    .0;

    if timed_out.load(Ordering::Relaxed)
        || (timeout_secs > 0 && started_at.elapsed() >= Duration::from_secs(timeout_secs))
    {
        let event = ProgressEvent {
            event: "terminal".to_string(),
            run_id: Some(run_id.clone()),
            stage: "generate:timeout".to_string(),
            progress: 1.0,
            message: Some("pipeline timed out".to_string()),
            mode: Some(args.request.mode.clone()),
            source_video_url: Some(args.request.youtube_url.clone()),
            provider_payload: Default::default(),
        };
        let _ = app_handle.emit("generate-progress", event);
        result = timeout_envelope(&args, &run_id);
    }
    unregister_run(&run_id);
    Ok(result)
}

pub async fn run_generate_authorized(
    args: GenerateCommandArgs,
    auth_state: &AuthAppState,
) -> (GenerateEnvelope, Vec<ProgressEvent>) {
    if let Some(error) = generation_auth_error(&args, auth_state).await {
        return (GenerateEnvelope::Failure { ok: false, error }, Vec::new());
    }
    run_generate(args)
}

pub fn run_generate(args: GenerateCommandArgs) -> (GenerateEnvelope, Vec<ProgressEvent>) {
    let run_id = normalize_run_id(args.request.run_id.as_deref());
    run_generate_with_sink(args, None, None, &run_id, Arc::new(AtomicBool::new(false)))
}

async fn generation_auth_error(
    args: &GenerateCommandArgs,
    auth_state: &AuthAppState,
) -> Option<ErrorEnvelope> {
    match auth_state.service.get_auth_state().await {
        Ok(SessionState::Licensed { .. } | SessionState::LicensedOfflineGrace { .. }) => None,
        Ok(_) => Some(auth_error_envelope(args, "license required")),
        Err(err) => Some(auth_error_envelope(args, &err.to_string())),
    }
}

fn auth_error_envelope(args: &GenerateCommandArgs, message: &str) -> ErrorEnvelope {
    ErrorEnvelope {
        mode: Some(args.request.mode.clone()),
        source_video_url: Some(args.request.youtube_url.clone()),
        error: message.to_string(),
        details: Some(json!({
            "code": "E_LICENSE_REQUIRED",
            "stage": "auth"
        })),
    }
}

pub fn run_generate_with_sink(
    args: GenerateCommandArgs,
    mut sink: Option<&mut dyn FnMut(&ProgressEvent)>,
    app_handle: Option<tauri::AppHandle>,
    run_id: &str,
    cancelled: Arc<AtomicBool>,
) -> (GenerateEnvelope, Vec<ProgressEvent>) {
    let req = as_request(&args.request);
    let output_json = args.request.output_json.clone();
    let mut events = Vec::new();

    let out = if let Some(test_mode) = args.test_mode.as_deref() {
        run_mock(&mut events, run_id, &req, test_mode, &mut sink)
    } else {
        let result = {
            let mut emit_stage = |stage: &str, progress: f64, message: Option<String>| {
                if cancelled.load(Ordering::Relaxed) {
                    return;
                }
                push_progress(&mut events, run_id, stage, progress, message, &mut sink);
            };
            let muapi_emitter = app_handle.as_ref().map(|h| {
                Arc::new(TauriMuapiProgressEmitter {
                    app_handle: h.clone(),
                    run_id: run_id.to_string(),
                }) as Arc<dyn ProgressEmitter>
            });
            generate_shorts_with_progress_live(
                &req,
                Some(&mut emit_stage),
                muapi_emitter,
                Some(&|| cancelled.load(Ordering::Relaxed)),
            )
        };
        if result.is_ok() {
            push_terminal(
                &mut events,
                run_id,
                "generate:success",
                Some("pipeline completed".to_string()),
                &mut sink,
            );
        } else if cancelled.load(Ordering::Relaxed) {
            push_terminal(
                &mut events,
                run_id,
                "generate:cancelled",
                Some("pipeline cancelled".to_string()),
                &mut sink,
            );
            return (cancelled_envelope(&args, run_id), events);
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

    if !cancelled.load(Ordering::Relaxed) {
        match &envelope {
            GenerateEnvelope::Success { .. } => {}
            GenerateEnvelope::Failure { error, .. } => {
                push_terminal(
                    &mut events,
                    run_id,
                    "generate:failure",
                    Some(error.error.clone()),
                    &mut sink,
                );
            }
        }
    }

    (envelope, events)
}

#[tauri::command]
pub async fn cancel_generate_run(args: CancelGenerateArgs) -> Result<bool, String> {
    Ok(cancel_run(args.run_id.trim()))
}
