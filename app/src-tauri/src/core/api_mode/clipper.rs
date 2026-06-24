use super::downloader::extract_video_url;
use super::muapi::{MuApiClient, MuApiError};
use serde_json::{json, Value};

pub async fn crop_clip(
    client: &MuApiClient,
    source_video_url: &str,
    start_time: f64,
    end_time: f64,
    aspect_ratio: &str,
) -> Result<String, MuApiError> {
    let payload = json!({
        "video_url": source_video_url,
        "start_time": start_time,
        "end_time": end_time,
        "aspect_ratio": aspect_ratio,
    });
    let label = format!("autocrop({:.0}-{:.0})", start_time, end_time);
    let result = client
        .run("autocrop", &payload, Some(&label), 2.0, 600.0)
        .await?;

    extract_video_url(&result).map_err(|message| MuApiError::Api {
        stage: "clip",
        message,
    })
}

pub fn crop_highlights_with_mapper<F>(highlights: Vec<Value>, mut map: F) -> Vec<Value>
where
    F: FnMut(&Value) -> Result<String, String>,
{
    let mut out = Vec::with_capacity(highlights.len());

    for h in highlights {
        match map(&h) {
            Ok(url) => {
                let mut item = h;
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("clip_url".to_string(), Value::String(url));
                }
                out.push(item);
            }
            Err(err) => {
                let mut item = h;
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("clip_url".to_string(), Value::Null);
                    obj.insert("error".to_string(), Value::String(err));
                }
                out.push(item);
            }
        }
    }

    out
}

pub async fn crop_highlights(
    client: &MuApiClient,
    source_video_url: &str,
    highlights: Vec<Value>,
    aspect_ratio: &str,
) -> Vec<Value> {
    let mut out = Vec::with_capacity(highlights.len());

    for h in highlights {
        let start_time = h.get("start_time").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let end_time = h.get("end_time").and_then(|v| v.as_f64()).unwrap_or(0.0);
        match crop_clip(client, source_video_url, start_time, end_time, aspect_ratio).await {
            Ok(url) => {
                let mut item = h;
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("clip_url".to_string(), Value::String(url));
                }
                out.push(item);
            }
            Err(err) => {
                let mut item = h;
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("clip_url".to_string(), Value::Null);
                    obj.insert("error".to_string(), Value::String(err.to_string()));
                }
                out.push(item);
            }
        }
    }

    out
}
