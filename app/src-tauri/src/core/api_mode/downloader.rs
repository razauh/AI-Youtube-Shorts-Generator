use super::muapi::{MuApiClient, MuApiError};
use serde_json::{json, Value};

pub fn extract_video_url(result: &Value) -> Result<String, String> {
    for key in ["video_url", "url", "output_url", "result_url"] {
        if let Some(v) = result.get(key).and_then(|v| v.as_str()) {
            if v.starts_with("http") {
                return Ok(v.to_string());
            }
        }
    }

    let output = result
        .get("outputs")
        .or_else(|| result.get("output"))
        .or_else(|| result.get("result"));

    if let Some(obj) = output.and_then(|v| v.as_object()) {
        for key in ["video_url", "url", "output_url"] {
            if let Some(v) = obj.get(key).and_then(|v| v.as_str()) {
                if v.starts_with("http") {
                    return Ok(v.to_string());
                }
            }
        }
    }

    if let Some(first) = output
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
    {
        if first.starts_with("http") {
            return Ok(first.to_string());
        }
    }

    Err(format!(
        "Could not find downloaded video URL in MuAPI response: {result}"
    ))
}

pub async fn download_youtube(
    client: &MuApiClient,
    video_url: &str,
    fmt: &str,
) -> Result<String, MuApiError> {
    let result = client
        .run(
            "youtube-download",
            &json!({"video_url": video_url, "format": fmt}),
            Some("youtube-download"),
            2.0,
            600.0,
        )
        .await?;

    extract_video_url(&result).map_err(|message| MuApiError::Api {
        stage: "download",
        message,
    })
}
