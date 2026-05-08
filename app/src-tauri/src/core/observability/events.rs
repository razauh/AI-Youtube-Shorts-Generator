use crate::core::errors::ErrorCode;
use serde_json::Value;

pub trait ProgressEmitter: Send + Sync {
    fn emit_status_change(&self, label: &str, status: &str);
}

#[derive(Default)]
pub struct NoopProgressEmitter;

impl ProgressEmitter for NoopProgressEmitter {
    fn emit_status_change(&self, _label: &str, _status: &str) {}
}

pub fn status_change_payload(label: &str, status: &str) -> Value {
    serde_json::json!({
        "event": "progress",
        "stage": "muapi_poll",
        "progress": 0.0,
        "message": format!("[muapi] {label}: {status}"),
        "code": ErrorCode::MuApiStatus.as_str(),
        "retryable": true,
    })
}
