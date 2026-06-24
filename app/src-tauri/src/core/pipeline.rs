use crate::core::api_mode::{clipper, downloader, muapi::MuApiClient, transcriber};
use crate::core::config::Config;
use crate::core::contracts::{ErrorEnvelope, Highlight, PipelineSuccess, ShortClip, Transcript};
use crate::core::errors::{redact_message, ErrorCode};
use crate::core::highlights::{self, HighlightLlm};
use crate::core::observability::events::{NoopProgressEmitter, ProgressEmitter};
use serde_json::{json, Value};
use std::sync::Arc;

const NO_SEGMENTS_ERROR: &str =
    "Whisper produced no segments. The video may have no detectable speech.";
const NO_HIGHLIGHTS_ERROR: &str = "Highlight generator returned zero clips.";

#[derive(Debug, Clone)]
pub struct GenerateShortsRequest {
    pub youtube_url: String,
    pub num_clips: usize,
    pub aspect_ratio: String,
    pub download_format: String,
    pub language: Option<String>,
    pub mode: String,
}

impl Default for GenerateShortsRequest {
    fn default() -> Self {
        Self {
            youtube_url: String::new(),
            num_clips: 3,
            aspect_ratio: "9:16".to_string(),
            download_format: "720".to_string(),
            language: None,
            mode: "api".to_string(),
        }
    }
}

pub trait PipelineStages {
    fn download_youtube(&mut self, video_url: &str, fmt: &str) -> Result<String, String>;
    fn transcribe(&mut self, media_url: &str, language: Option<&str>) -> Result<Value, String>;
    fn get_highlights(&mut self, transcript: &Value, num_clips: usize) -> Result<Value, String>;
    fn crop_highlights(
        &mut self,
        source_video_url: &str,
        highlights: Vec<Value>,
        aspect_ratio: &str,
    ) -> Result<Vec<Value>, String>;
}

fn emit_progress(
    cb: &mut Option<&mut dyn FnMut(&str, f64, Option<String>)>,
    stage: &str,
    progress: f64,
    message: Option<String>,
) {
    if let Some(f) = cb.as_deref_mut() {
        f(stage, progress, message);
    }
}

fn map_api_error(
    stage: &'static str,
    code: ErrorCode,
    message: String,
    source_video_url: Option<String>,
) -> ErrorEnvelope {
    ErrorEnvelope {
        mode: Some("api".to_string()),
        source_video_url,
        error: redact_message(&message),
        details: Some(json!({"stage": stage, "code": code.as_str(), "retryable": false})),
    }
}

fn score_as_i64(v: &Value) -> i64 {
    if let Some(n) = v.as_i64() {
        n
    } else if let Some(s) = v.as_str() {
        s.parse::<i64>().unwrap_or(0)
    } else {
        0
    }
}

pub fn generate_shorts_with(
    request: &GenerateShortsRequest,
    stages: &mut dyn PipelineStages,
) -> Result<PipelineSuccess, ErrorEnvelope> {
    generate_shorts_with_progress(request, stages, None, None)
}

pub fn generate_shorts_with_progress(
    request: &GenerateShortsRequest,
    stages: &mut dyn PipelineStages,
    mut progress_cb: Option<&mut dyn FnMut(&str, f64, Option<String>)>,
    cancel_cb: Option<&dyn Fn() -> bool>,
) -> Result<PipelineSuccess, ErrorEnvelope> {
    let mode = if request.mode.trim().is_empty() {
        "api".to_string()
    } else {
        request.mode.to_lowercase()
    };

    let is_cancelled = || cancel_cb.map(|cb| cb()).unwrap_or(false);
    let cancelled = || ErrorEnvelope {
        mode: Some(mode.clone()),
        source_video_url: Some(request.youtube_url.clone()),
        error: "Generation cancelled.".to_string(),
        details: Some(json!({"stage":"cancel","code":"E_GENERATION_CANCELLED"})),
    };

    if mode != "api" {
        return Err(ErrorEnvelope {
            mode: None,
            source_video_url: None,
            error: format!("Unknown mode: '{mode}'. Use 'api'."),
            details: None,
        });
    }

    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "download:start",
        0.1,
        Some("downloading source video".to_string()),
    );
    let source_url = stages
        .download_youtube(&request.youtube_url, &request.download_format)
        .map_err(|e| map_api_error("download", ErrorCode::DownloadFailed, e, None))?;
    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "download:end",
        0.25,
        Some("download completed".to_string()),
    );

    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "transcribe:start",
        0.3,
        Some("transcription started".to_string()),
    );
    let transcript_value = stages
        .transcribe(&source_url, request.language.as_deref())
        .map_err(|e| {
            map_api_error(
                "transcribe",
                ErrorCode::TranscribeFailed,
                e,
                Some(source_url.clone()),
            )
        })?;
    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "transcribe:end",
        0.55,
        Some("transcription completed".to_string()),
    );

    let transcript: Transcript = serde_json::from_value(transcript_value.clone()).map_err(|e| {
        map_api_error(
            "transcribe",
            ErrorCode::TranscribeFailed,
            format!("invalid transcript payload: {e}"),
            Some(source_url.clone()),
        )
    })?;

    if transcript.segments.is_empty() {
        return Err(map_api_error(
            "transcribe",
            ErrorCode::NoTranscriptSegments,
            NO_SEGMENTS_ERROR.to_string(),
            Some(source_url),
        ));
    }

    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "highlights:start",
        0.6,
        Some("highlight detection started".to_string()),
    );
    let highlights_result = stages
        .get_highlights(&transcript_value, request.num_clips)
        .map_err(|e| {
            map_api_error(
                "highlights",
                ErrorCode::HighlightsFailed,
                e,
                Some(source_url.clone()),
            )
        })?;
    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "highlights:end",
        0.75,
        Some("highlight detection completed".to_string()),
    );

    let all_highlights: Vec<Value> = highlights_result
        .get("highlights")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if all_highlights.is_empty() {
        return Err(map_api_error(
            "highlights",
            ErrorCode::NoHighlights,
            NO_HIGHLIGHTS_ERROR.to_string(),
            Some(source_url.clone()),
        ));
    }

    let mut top = all_highlights.clone();
    top.sort_by(|a, b| {
        let sa = score_as_i64(a.get("score").unwrap_or(&Value::Null));
        let sb = score_as_i64(b.get("score").unwrap_or(&Value::Null));
        sb.cmp(&sa)
    });
    top.truncate(request.num_clips);

    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "clip:start",
        0.8,
        Some("clip rendering started".to_string()),
    );
    let shorts_values = stages
        .crop_highlights(&source_url, top, &request.aspect_ratio)
        .map_err(|e| map_api_error("clip", ErrorCode::ClipFailed, e, Some(source_url.clone())))?;
    if is_cancelled() {
        return Err(cancelled());
    }
    emit_progress(
        &mut progress_cb,
        "clip:end",
        1.0,
        Some("clip rendering completed".to_string()),
    );

    let highlights: Vec<Highlight> =
        serde_json::from_value(Value::Array(all_highlights)).map_err(|e| {
            map_api_error(
                "highlights",
                ErrorCode::HighlightsFailed,
                format!("invalid highlights payload: {e}"),
                Some(source_url.clone()),
            )
        })?;

    let shorts: Vec<ShortClip> =
        serde_json::from_value(Value::Array(shorts_values)).map_err(|e| {
            map_api_error(
                "clip",
                ErrorCode::ClipFailed,
                format!("invalid shorts payload: {e}"),
                Some(source_url.clone()),
            )
        })?;

    Ok(PipelineSuccess {
        mode: "api".to_string(),
        source_video_url: source_url,
        transcript,
        highlights,
        shorts,
        provider_payload: Default::default(),
    })
}

struct MuApiLlm {
    client: MuApiClient,
    runtime: tokio::runtime::Runtime,
}

impl HighlightLlm for MuApiLlm {
    fn call(&self, prompt: &str) -> Result<String, String> {
        let result = self
            .runtime
            .block_on(self.client.run(
                "gpt-5-mini",
                &json!({"prompt": prompt}),
                Some("gpt-5-mini"),
                2.0,
                highlights::GPT_CALL_TIMEOUT_SECONDS,
            ))
            .map_err(|e| e.to_string())?;

        if let Some(outputs) = result.get("outputs").and_then(Value::as_array) {
            if let Some(first) = outputs.first().and_then(Value::as_str) {
                if !first.trim().is_empty() {
                    return Ok(first.to_string());
                }
            }
        }

        for key in ["output", "text", "response", "result", "content"] {
            if let Some(v) = result.get(key) {
                if let Some(s) = v.as_str() {
                    if !s.trim().is_empty() {
                        return Ok(s.to_string());
                    }
                }
                if let Some(obj) = v.as_object() {
                    for nested in ["text", "content"] {
                        if let Some(s) = obj.get(nested).and_then(Value::as_str) {
                            if !s.trim().is_empty() {
                                return Ok(s.to_string());
                            }
                        }
                    }
                }
                if let Some(arr) = v.as_array() {
                    if let Some(s) = arr.first().and_then(Value::as_str) {
                        if !s.trim().is_empty() {
                            return Ok(s.to_string());
                        }
                    }
                }
            }
        }

        Err(format!(
            "Could not extract gpt-5-mini text from response: {result}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_shorts_with_progress, GenerateShortsRequest, MockPipelineStages};

    #[test]
    fn cancel_cb_short_circuits_pipeline() {
        let mut stages = MockPipelineStages::default()
            .with_download(Ok("https://cdn.example.com/video.mp4".to_string()))
            .with_transcript(Ok(serde_json::json!({
                "duration": 12.0,
                "segments": [{"start": 0.0, "end": 6.0, "text": "hello"}]
            })))
            .with_highlights(Ok(serde_json::json!({
                "highlights": [{
                    "title": "hello",
                    "start_time": 0.0,
                    "end_time": 6.0,
                    "score": 90,
                    "hook_sentence": "hello",
                    "virality_reason": "test"
                }]
            })))
            .with_crop(Ok(vec![]));
        let req = GenerateShortsRequest {
            youtube_url: "https://youtube.com/watch?v=abc".to_string(),
            num_clips: 1,
            aspect_ratio: "9:16".to_string(),
            download_format: "720".to_string(),
            language: None,
            mode: "api".to_string(),
        };
        let cancel = || true;
        let out = generate_shorts_with_progress(&req, &mut stages, None, Some(&cancel));
        assert!(out.is_err());
        let err = out.err().unwrap();
        assert!(err.error.to_lowercase().contains("cancel"));
    }
}

pub struct LivePipelineStages {
    config: Config,
    runtime: tokio::runtime::Runtime,
    emitter: Arc<dyn ProgressEmitter>,
}

impl LivePipelineStages {
    pub fn new(config: Config) -> Result<Self, String> {
        Self::new_with_emitter(config, Arc::new(NoopProgressEmitter))
    }

    pub fn new_with_emitter(
        config: Config,
        emitter: Arc<dyn ProgressEmitter>,
    ) -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            config,
            runtime,
            emitter,
        })
    }

    fn client(&self) -> MuApiClient {
        MuApiClient::with_emitter(self.config.clone(), self.emitter.clone())
    }
}

impl PipelineStages for LivePipelineStages {
    fn download_youtube(&mut self, video_url: &str, fmt: &str) -> Result<String, String> {
        self.runtime
            .block_on(downloader::download_youtube(&self.client(), video_url, fmt))
            .map_err(|e| e.to_string())
    }

    fn transcribe(&mut self, media_url: &str, language: Option<&str>) -> Result<Value, String> {
        let transcript = self
            .runtime
            .block_on(transcriber::transcribe(&self.client(), media_url, language))
            .map_err(|e| e.to_string())?;
        serde_json::to_value(transcript).map_err(|e| e.to_string())
    }

    fn get_highlights(&mut self, transcript: &Value, num_clips: usize) -> Result<Value, String> {
        let llm = MuApiLlm {
            client: self.client(),
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .map_err(|e| e.to_string())?,
        };
        highlights::get_highlights(transcript.clone(), num_clips, &llm)
    }

    fn crop_highlights(
        &mut self,
        source_video_url: &str,
        highlights: Vec<Value>,
        aspect_ratio: &str,
    ) -> Result<Vec<Value>, String> {
        Ok(self.runtime.block_on(clipper::crop_highlights(
            &self.client(),
            source_video_url,
            highlights,
            aspect_ratio,
        )))
    }
}

pub fn generate_shorts(request: &GenerateShortsRequest) -> Result<PipelineSuccess, ErrorEnvelope> {
    generate_shorts_with_progress_live(request, None, None, None)
}

pub fn generate_shorts_with_progress_live(
    request: &GenerateShortsRequest,
    progress_cb: Option<&mut dyn FnMut(&str, f64, Option<String>)>,
    muapi_emitter: Option<Arc<dyn ProgressEmitter>>,
    cancel_cb: Option<&dyn Fn() -> bool>,
) -> Result<PipelineSuccess, ErrorEnvelope> {
    let config = Config::from_env().map_err(|e| ErrorEnvelope {
        mode: Some("api".to_string()),
        source_video_url: None,
        error: e.to_string(),
        details: None,
    })?;

    let emitter: Arc<dyn ProgressEmitter> =
        muapi_emitter.unwrap_or_else(|| Arc::new(NoopProgressEmitter));
    let mut live =
        LivePipelineStages::new_with_emitter(config, emitter).map_err(|e| ErrorEnvelope {
            mode: Some("api".to_string()),
            source_video_url: None,
            error: e,
            details: None,
        })?;

    generate_shorts_with_progress(request, &mut live, progress_cb, cancel_cb)
}

#[derive(Default)]
pub struct MockPipelineStages {
    download: Option<Result<String, String>>,
    transcript: Option<Result<Value, String>>,
    highlights: Option<Result<Value, String>>,
    crop: Option<Result<Vec<Value>, String>>,
    call_log: Vec<String>,
}

impl MockPipelineStages {
    pub fn with_download(mut self, value: Result<String, String>) -> Self {
        self.download = Some(value);
        self
    }

    pub fn with_transcript(mut self, value: Result<Value, String>) -> Self {
        self.transcript = Some(value);
        self
    }

    pub fn with_highlights(mut self, value: Result<Value, String>) -> Self {
        self.highlights = Some(value);
        self
    }

    pub fn with_crop(mut self, value: Result<Vec<Value>, String>) -> Self {
        self.crop = Some(value);
        self
    }

    pub fn call_log(&self) -> Vec<&str> {
        self.call_log.iter().map(String::as_str).collect()
    }
}

impl PipelineStages for MockPipelineStages {
    fn download_youtube(&mut self, _video_url: &str, _fmt: &str) -> Result<String, String> {
        self.call_log.push("download".to_string());
        self.download
            .clone()
            .unwrap_or_else(|| Err("download not stubbed".to_string()))
    }

    fn transcribe(&mut self, _media_url: &str, _language: Option<&str>) -> Result<Value, String> {
        self.call_log.push("transcribe".to_string());
        self.transcript
            .clone()
            .unwrap_or_else(|| Err("transcribe not stubbed".to_string()))
    }

    fn get_highlights(&mut self, _transcript: &Value, _num_clips: usize) -> Result<Value, String> {
        self.call_log.push("highlights".to_string());
        self.highlights
            .clone()
            .unwrap_or_else(|| Err("highlights not stubbed".to_string()))
    }

    fn crop_highlights(
        &mut self,
        _source_video_url: &str,
        _highlights: Vec<Value>,
        _aspect_ratio: &str,
    ) -> Result<Vec<Value>, String> {
        self.call_log.push("crop".to_string());
        self.crop
            .clone()
            .unwrap_or_else(|| Err("crop not stubbed".to_string()))
    }
}
