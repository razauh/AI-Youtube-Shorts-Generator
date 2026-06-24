use super::muapi::{MuApiClient, MuApiError};
use crate::core::contracts::{Transcript, TranscriptSegment};
use serde_json::{json, Value};

fn coerce_verbose(raw: &Value) -> Value {
    if let Some(s) = raw.as_str() {
        return serde_json::from_str::<Value>(s).unwrap_or_else(|_| json!({}));
    }
    if raw.is_object() {
        return raw.clone();
    }
    json!({})
}

pub fn extract_verbose_payload(result: &Value) -> Result<Value, String> {
    for key in ["output", "result", "outputs"] {
        if let Some(v) = result.get(key) {
            if v.is_object() && v.get("segments").is_some() {
                return Ok(v.clone());
            }

            if let Some(first) = v.as_array().and_then(|arr| arr.first()) {
                let decoded = coerce_verbose(first);
                if decoded.get("segments").is_some() {
                    return Ok(decoded);
                }
            }

            if v.is_string() {
                let decoded = coerce_verbose(v);
                if decoded.get("segments").is_some() {
                    return Ok(decoded);
                }
            }
        }
    }

    if result.get("segments").is_some() {
        return Ok(result.clone());
    }

    Err(format!(
        "Could not find Whisper segments in MuAPI response: {result}"
    ))
}

fn as_f64_or_default(value: Option<&Value>, default: f64) -> f64 {
    match value {
        Some(v) => {
            if let Some(n) = v.as_f64() {
                n
            } else if let Some(s) = v.as_str() {
                s.parse::<f64>().unwrap_or(default)
            } else {
                default
            }
        }
        None => default,
    }
}

pub fn normalize_transcript(verbose: &Value) -> Result<Transcript, String> {
    let mut segments = Vec::new();

    for s in verbose
        .get("segments")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
    {
        let start = as_f64_or_default(s.get("start"), 0.0);
        let end = as_f64_or_default(s.get("end"), 0.0);
        let text = s
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        segments.push(TranscriptSegment { start, end, text });
    }

    let duration = as_f64_or_default(
        verbose.get("duration"),
        segments.last().map(|s| s.end).unwrap_or(0.0),
    );

    Ok(Transcript {
        duration,
        segments,
        extra: Default::default(),
    })
}

pub async fn transcribe(
    client: &MuApiClient,
    media_url: &str,
    language: Option<&str>,
) -> Result<Transcript, MuApiError> {
    let mut payload = json!({
        "audio_url": media_url,
        "response_format": "verbose_json"
    });
    if let Some(lang) = language {
        payload["language"] = Value::String(lang.to_string());
    }

    let result = client
        .run(
            "openai-whisper",
            &payload,
            Some("openai-whisper"),
            2.0,
            600.0,
        )
        .await?;

    let verbose = extract_verbose_payload(&result).map_err(|message| MuApiError::Api {
        stage: "transcribe",
        message,
    })?;

    normalize_transcript(&verbose).map_err(|message| MuApiError::Api {
        stage: "transcribe",
        message,
    })
}
